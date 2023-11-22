use crate::{
    core::{
        assets::Handle,
        camera::Camera,
        mesh::{GpuMesh, MeshBundle, VertexAttribute},
        Color, FxHashSet, GpuMaterial, Light, MaterialBundle, TextureBundle,
    },
    render::{
        rpass::{Globals, RenderingPass},
        RenderTarget, Renderer,
    },
    scene::{NodeIdx, Scene},
};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use legion::IntoQuery;
use std::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Locals {
    model: [f32; 16],
    model_view_inv: [f32; 16],
}

impl Locals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    pub const fn identity() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array(),
            model_view_inv: Mat4::IDENTITY.to_cols_array(),
        }
    }
}

/// Initial number of meshes of which the buffer can hold.
pub const INITIAL_MESHES_COUNT: usize = 512;

/// Push constants for the shading pipeline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PConsts {
    /// Index of the first instance in the instance buffer.
    instance_base_index: u32,
    /// Material index.
    material_index: u32,
}

impl PConsts {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

pub const MAX_DIRECTIONAL_LIGHTS: usize = 256;
pub const MAX_POINT_LIGHTS: usize = 256;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirectionalLight {
    pub direction: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirectionalLightArray {
    pub len: [u32; 4], // make sure the array is 16-byte aligned.
    pub lights: [DirectionalLight; MAX_DIRECTIONAL_LIGHTS],
}

impl DirectionalLightArray {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn empty() -> Self {
        Self {
            len: [0u32; 4],
            lights: [DirectionalLight::empty(); MAX_DIRECTIONAL_LIGHTS],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PointLight {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PointLightArray {
    pub len: [u32; 4],
    pub lights: [PointLight; MAX_POINT_LIGHTS],
}

impl PointLightArray {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn empty() -> Self {
        Self {
            len: [0u32; 4], // make sure the array is 16-byte aligned.
            lights: [PointLight::empty(); MAX_POINT_LIGHTS],
        }
    }
}

impl DirectionalLight {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn empty() -> Self {
        Self {
            direction: [0.0; 4],
            color: [0.0; 4],
        }
    }
}

impl PointLight {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn empty() -> Self {
        Self {
            position: [0.0; 4],
            color: [0.0; 4],
        }
    }
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub struct BlinnPhongShading {
    pub depth_texture: Option<(wgpu::Texture, wgpu::TextureView)>,

    pub globals_bind_group: wgpu::BindGroup,
    pub globals_uniform_buffer: wgpu::Buffer,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,

    pub locals_bind_group: wgpu::BindGroup,
    pub locals_storage_buffer: wgpu::Buffer,
    pub locals_bind_group_layout: wgpu::BindGroupLayout,

    pub materials_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,

    pub lights_bind_group_layout: wgpu::BindGroupLayout,
    pub lights_bind_group: wgpu::BindGroup,
    pub directional_lights_storage_buffer: wgpu::Buffer,
    pub point_lights_storage_buffer: wgpu::Buffer,

    /// The render pipeline for rendering entities.
    pub entity_pipeline: wgpu::RenderPipeline,
}

impl BlinnPhongShading {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shading_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blinn_phong.wgsl").into()),
        });

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shading_globals_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(Globals::SIZE),
                    },
                    count: None,
                }],
            });
        let globals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shading_globals_uniform_buffer"),
            size: Globals::SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shading_globals_bind_group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_uniform_buffer.as_entire_binding(),
            }],
        });

        let locals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shading_locals_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(Locals::SIZE),
                    },
                    count: None,
                }],
            });

        let locals_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shading_locals_storage_buffer"),
            size: Locals::SIZE * INITIAL_MESHES_COUNT as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let locals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shading_locals_bind_group"),
            layout: &locals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: locals_storage_buffer.as_entire_binding(),
            }],
        });

        let materials_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shading_materials_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(GpuMaterial::SIZE),
                    },
                    count: None,
                }],
            });

        let textures_bind_group_layout = texture_bundle_bind_group_layout(device);

        let lights_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shading_lights_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                DirectionalLightArray::SIZE as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(PointLightArray::SIZE as u64),
                        },
                        count: None,
                    },
                ],
            });

        // Preallocate a buffer for directional lights.
        let directional_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shading_directional_lights_buffer"),
            size: std::mem::size_of::<DirectionalLightArray>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Preallocate a buffer for point lights.
        let point_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shading_point_lights_buffer"),
            size: std::mem::size_of::<PointLightArray>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shading_lights_bind_group"),
            layout: &lights_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: directional_lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: point_lights_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blinn_phong_shading_pipeline_layout"),
            bind_group_layouts: &[
                &globals_bind_group_layout,
                &locals_bind_group_layout,
                &materials_bind_group_layout,
                &textures_bind_group_layout,
                &lights_bind_group_layout,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..PConsts::SIZE as u32,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blinn_phong_shading_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Position.
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Normal.
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // UV0.
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                // unclipped_depth: false,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            depth_texture: None,
            globals_bind_group,
            globals_uniform_buffer,
            globals_bind_group_layout,
            locals_bind_group,
            locals_storage_buffer,
            locals_bind_group_layout,
            materials_bind_group_layout,
            textures_bind_group_layout,
            lights_bind_group_layout,
            lights_bind_group,
            directional_lights_storage_buffer: directional_lights_buffer,
            point_lights_storage_buffer: point_lights_buffer,
            entity_pipeline: pipeline,
        }
    }
}

/// Maximum number of textures in a texture array.
pub const MAX_TEXTURE_ARRAY_LEN: u32 = 128;
pub const MAX_SAMPLER_ARRAY_LEN: u32 = 16;

pub fn texture_bundle_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blinn_phong_textures_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: NonZeroU32::new(MAX_TEXTURE_ARRAY_LEN),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(
                        std::mem::size_of::<u32>() as u64 * MAX_TEXTURE_ARRAY_LEN as u64,
                    ),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: NonZeroU32::new(MAX_SAMPLER_ARRAY_LEN),
            },
        ],
    })
}

impl RenderingPass for BlinnPhongShading {
    fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        renderer: &Renderer,
        scene: &Scene,
    ) {
        profiling::scope!("BlinnPhongShading::record");

        let view_mat = {
            // Update camera globals.
            let mut camera_query = <(&Camera, &NodeIdx)>::query();
            let num_cameras = camera_query.iter(&scene.world).count();
            if num_cameras == 0 {
                log::error!("No camera found in the scene! Skip rendering!");
                return;
            }

            let main_camera = camera_query
                .iter(&scene.world)
                .find(|(camera, _)| camera.is_main);

            let (camera, node_idx) = match main_camera {
                None => {
                    // If there is no main camera, use the first camera.
                    let camera = camera_query.iter(&scene.world).next().unwrap();
                    log::warn!("No main camera found, use the first camera #{:?}", camera.1);
                    camera
                }
                Some(camera) => {
                    // If there is a main camera, use it.
                    log::debug!("Use main camera {:?}", camera.1);
                    camera
                }
            };

            let view_mat = scene.nodes.inverse_world(*node_idx).to_mat4();
            let proj = camera.proj_matrix(target.aspect_ratio());
            let globals = Globals {
                view: view_mat.to_cols_array(),
                proj: proj.to_cols_array(),
            };
            queue.write_buffer(
                &self.globals_uniform_buffer,
                0,
                bytemuck::bytes_of(&globals),
            );
            view_mat
        };

        {
            // (Re-)create depth texture if necessary.
            let need_recreate = match &self.depth_texture {
                None => true,
                Some(depth) => target.size != depth.0.size(),
            };

            if need_recreate {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("wireframe_depth_texture"),
                    size: target.size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.depth_texture = Some((texture, view));
            }
        }

        // Create render pass.
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blinn_phong_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(*Color::DARK_GREY),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.as_ref().unwrap().1,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.entity_pipeline);
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);

        // Update light buffers.
        let mut light_query = <(&Light, &NodeIdx)>::query();
        let mut pl_arr = PointLightArray::empty();
        let mut dl_arr = DirectionalLightArray::empty();

        let active_lights = light_query
            .iter(&scene.world)
            .filter(|(_, node_idx)| scene.nodes[**node_idx].is_active());
        for (light, node_idx) in active_lights {
            match light {
                Light::Directional { direction, color } => {
                    dl_arr.lights[dl_arr.len[0] as usize] = DirectionalLight {
                        direction: [-direction.x, -direction.y, -direction.z, 0.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                    };
                    dl_arr.len[0] += 1;
                }
                Light::Point { color } => {
                    let transform = scene.nodes.world(*node_idx);
                    let position = transform.translation;
                    pl_arr.lights[pl_arr.len[0] as usize] = PointLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                    };
                    pl_arr.len[0] += 1;
                }
            }
        }
        // Update light buffers.
        queue.write_buffer(
            &self.directional_lights_storage_buffer,
            0,
            bytemuck::bytes_of(&dl_arr),
        );
        queue.write_buffer(
            &self.point_lights_storage_buffer,
            0,
            bytemuck::bytes_of(&pl_arr),
        );

        // Bind lights. TODO: maybe move lights to renderer?
        render_pass.set_bind_group(4, &self.lights_bind_group, &[]);

        let mut mesh_query = <(&NodeIdx, &MeshBundle)>::query();
        let mut mesh_bundles = FxHashSet::default();
        let mut total_inst_count = 0;
        for (_, mesh) in mesh_query
            .iter(&scene.world)
            .filter(|(node_idx, _)| scene.nodes[**node_idx].is_visible())
        {
            mesh_bundles.insert(mesh);
            total_inst_count += 1;
        }
        log::debug!(
            "Draw {} meshes of {} instances",
            mesh_bundles.len(),
            total_inst_count
        );

        if total_inst_count == 0 {
            return;
        }

        // Resize locals buffer if necessary.
        if total_inst_count > self.locals_storage_buffer.size() / Locals::SIZE {
            log::warn!(
                "Too many instances to draw in one frame: {} > {}, enlarge instance locals buffer!",
                total_inst_count,
                INITIAL_MESHES_COUNT
            );
            let mut new_size = self.locals_storage_buffer.size();
            // Resize instance buffer if necessary.
            while new_size < total_inst_count as u64 * Locals::SIZE {
                new_size += 128 * Locals::SIZE;
            }
            log::info!("Resize instance locals buffer to {}", new_size);
            self.locals_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("shading_locals_storage_buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            log::info!("Recreate locals bind group");
            self.locals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shading_locals_bind_group"),
                layout: &self.locals_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.locals_storage_buffer.as_entire_binding(),
                }],
            });
        }

        // Bind locals.
        render_pass.set_bind_group(1, &self.locals_bind_group, &[]);
        // Preparing locals for each mesh.
        let mut locals = vec![Locals::identity(); total_inst_count as usize];

        // Get the mesh buffer, which contains all vertex attributes.
        let buffer = renderer.meshes.buffer();
        let mut locals_offset = 0u32;
        for bundle in mesh_bundles {
            // Update locals.
            let instancing = renderer
                .instancing
                .get(&bundle.mesh)
                .expect("Unreachable! Instancing should be created for all meshes!");
            let mut inst_count = 0;
            for (i, node) in instancing.nodes.iter().enumerate() {
                if !scene.nodes[*node].is_visible() {
                    continue;
                }
                inst_count += 1;
                let model_mat = scene.nodes.world(*node).to_mat4();
                locals[locals_offset as usize + i] = Locals {
                    model: model_mat.to_cols_array(),
                    model_view_inv: (view_mat * model_mat).inverse().to_cols_array(),
                };
            }
            debug_assert!(
                inst_count > 0,
                "Unreachable! Only visible nodes should be rendered!"
            );

            // Update push constants: instance base index.
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::bytes_of(&(locals_offset as u32)),
            );
            locals_offset += inst_count;
            let inst_range = 0..inst_count;

            let mtls = renderer.material_bundles.get(bundle.materials).unwrap();
            // let node = &scene.nodes[*node_idx];
            let texs = renderer.texture_bundles.get(bundle.textures).unwrap();
            // let model_mat = scene.nodes.world(*node_idx).to_mat4();

            match renderer.meshes.get(bundle.mesh) {
                None => {
                    log::error!("Missing mesh {:?}", bundle.mesh);
                    continue;
                }
                Some(mesh) => {
                    if let Some(pos_range) =
                        mesh.get_vertex_attribute_range(VertexAttribute::POSITION)
                    {
                        // Bind vertex buffer - position.
                        render_pass.set_vertex_buffer(0, buffer.slice(pos_range.clone()));

                        // Bind vertex buffer - normal.
                        if let Some(normals_range) =
                            mesh.get_vertex_attribute_range(VertexAttribute::NORMAL)
                        {
                            render_pass.set_vertex_buffer(1, buffer.slice(normals_range.clone()));
                        }
                        // Bind vertex buffer - uv0.
                        if let Some(uv_range) =
                            mesh.get_vertex_attribute_range(VertexAttribute::UV0)
                        {
                            render_pass.set_vertex_buffer(2, buffer.slice(uv_range.clone()));
                        }

                        // Bind material.
                        render_pass.set_bind_group(2, &mtls.bind_group, &[]);
                        // Bind textures.
                        render_pass.set_bind_group(3, texs.bind_group.as_ref().unwrap(), &[]);

                        // TODO: support material override.
                        // let override_material = node
                        //     .material_override
                        //     .map(|id| id.min(mtls.n_materials - 1));
                        let override_material = None;

                        match mesh.index_format {
                            None => {
                                // No index buffer, draw directly.
                                match mesh.sub_meshes.as_ref() {
                                    None => {
                                        // No sub-meshes, use the default material.
                                        // Update material index.
                                        let material_id = override_material.unwrap_or(0);
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                                            64,
                                            bytemuck::bytes_of(&material_id),
                                        );
                                        render_pass.draw(0..mesh.vertex_count, inst_range);
                                    }
                                    Some(sub_meshes) => {
                                        // Draw each sub-mesh.
                                        for sub_mesh in sub_meshes {
                                            let material_id =
                                                override_material.unwrap_or_else(|| {
                                                    sub_mesh
                                                        .material
                                                        .unwrap_or(mtls.n_materials - 1)
                                                });
                                            // Update material index.
                                            render_pass.set_push_constants(
                                                wgpu::ShaderStages::VERTEX_FRAGMENT,
                                                64,
                                                bytemuck::bytes_of(&material_id),
                                            );
                                            render_pass.draw(
                                                sub_mesh.range.start..sub_mesh.range.end,
                                                inst_range.clone(),
                                            )
                                        }
                                    }
                                }
                            }
                            Some(index_format) => {
                                render_pass.set_index_buffer(
                                    buffer.slice(mesh.index_range.clone()),
                                    index_format,
                                );
                                match mesh.sub_meshes.as_ref() {
                                    None => {
                                        log::trace!("Draw mesh with index, no sub-meshes");
                                        // No sub-meshes, use the default material.
                                        // Update material index.
                                        let material_id = override_material.unwrap_or(0);
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                                            4,
                                            bytemuck::bytes_of(&material_id),
                                        );
                                        render_pass.draw_indexed(
                                            0..mesh.index_count,
                                            0,
                                            inst_range,
                                        );
                                    }
                                    Some(sub_meshes) => {
                                        log::trace!("Draw mesh with index, with sub-meshes");
                                        for sm in sub_meshes {
                                            log::trace!(
                                                "Draw sub-mesh {}-{}",
                                                sm.range.start,
                                                sm.range.end
                                            );
                                            let material_id =
                                                override_material.unwrap_or_else(|| {
                                                    sm.material.unwrap_or(mtls.n_materials - 1)
                                                });
                                            // Update material index.
                                            render_pass.set_push_constants(
                                                wgpu::ShaderStages::VERTEX_FRAGMENT,
                                                4,
                                                bytemuck::bytes_of(&material_id),
                                            );
                                            // Draw the sub-mesh.
                                            render_pass.draw_indexed(
                                                sm.range.start..sm.range.end,
                                                0,
                                                inst_range.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update locals before submitting the render pass.
        queue.write_buffer(
            &self.locals_storage_buffer,
            0,
            bytemuck::cast_slice(&locals),
        );
    }
}
