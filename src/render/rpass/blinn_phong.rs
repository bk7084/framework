use crate::{
    core::{
        assets::Handle,
        camera::Camera,
        mesh::{GpuMesh, VertexAttribute},
        Color, GpuMaterial, Light, MaterialBundle, TextureBundle,
    },
    render::{rpass::RenderingPass, RenderTarget, Renderer},
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
struct Globals {
    view: [f32; 16],
    proj: [f32; 16],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Locals {
    model: [f32; 16],
}

impl Locals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

pub const INIT_OBJECTS_CAPACITY: usize = 512;

/// Push constants for the shading pipeline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PConsts {
    /// Inverse of the model view matrix (for transforming normals).
    model_view_inv: [f32; 16],
    /// Material index.
    material: u32,
}

impl PConsts {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

pub const MAX_DIRECTIONAL_LIGHTS: usize = 256;
pub const MAX_POINT_LIGHTS: usize = 256;

impl Globals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

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
    pub locals_uniform_buffer: wgpu::Buffer,
    pub locals_bind_group_layout: wgpu::BindGroupLayout,

    pub materials_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,

    pub lights_bind_group_layout: wgpu::BindGroupLayout,
    pub lights_bind_group: wgpu::BindGroup,
    pub directional_lights_storage_buffer: wgpu::Buffer,
    pub point_lights_storage_buffer: wgpu::Buffer,

    pub pipeline: wgpu::RenderPipeline,
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(Locals::SIZE),
                    },
                    count: None,
                }],
            });

        let locals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shading_locals_uniform_buffer"),
            size: Locals::SIZE * INIT_OBJECTS_CAPACITY as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let locals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shading_locals_bind_group"),
            layout: &locals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &locals_uniform_buffer,
                    offset: 0,
                    size: NonZeroU64::new(Locals::SIZE),
                }),
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
            locals_uniform_buffer,
            locals_bind_group_layout,
            materials_bind_group_layout,
            textures_bind_group_layout,
            lights_bind_group_layout,
            lights_bind_group,
            directional_lights_storage_buffer: directional_lights_buffer,
            point_lights_storage_buffer: point_lights_buffer,
            pipeline,
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

        let mut view_mat = Mat4::IDENTITY;

        // Update globals.
        let mut camera_query = <(&Camera, &NodeIdx)>::query();
        // TODO: support multiple cameras.
        for (camera, node_idx) in camera_query.iter(&scene.world) {
            if camera.is_main {
                view_mat = scene.nodes.inverse_world(*node_idx).to_mat4();
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
                break;
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

        render_pass.set_pipeline(&self.pipeline);
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

        let mut mesh_query = <(
            &Handle<GpuMesh>,
            &NodeIdx,
            &Handle<MaterialBundle>,
            &Handle<TextureBundle>,
        )>::query();

        let mesh_count = mesh_query.iter(&scene.world).count();

        if mesh_count == 0 {
            return;
        }

        // Resize locals buffer if necessary.
        if mesh_count > INIT_OBJECTS_CAPACITY {
            log::warn!(
                "Too many meshes to draw in one frame: {} > {}, enlarge locals buffer!",
                mesh_count,
                INIT_OBJECTS_CAPACITY
            );
            let mut new_locals_size = self.locals_uniform_buffer.size();
            // Resize locals buffer if necessary.
            while new_locals_size < mesh_count as u64 * Locals::SIZE as u64 {
                new_locals_size += 256 * Locals::SIZE as u64;
            }
            log::info!("Resize locals buffer to {}", new_locals_size);
            self.locals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("shading_locals_uniform_buffer"),
                size: new_locals_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Get the mesh buffer, which contains all vertex attributes.
        let buffer = renderer.meshes.buffer();
        for ((i, (mesh_hdl, node_idx, materials_hdl, textures_hdl))) in mesh_query
            .iter(&scene.world)
            .filter(|(_, node_idx, _, _)| scene.nodes[**node_idx].is_visible())
            .enumerate()
        {
            let node = &scene.nodes[*node_idx];
            let mtls = renderer.material_bundles.get(*materials_hdl).unwrap();
            let texture_bundle = renderer.texture_bundles.get(*textures_hdl).unwrap();
            let model_mat = scene.nodes.world(*node_idx).to_mat4();

            // Update locals.
            let offset = i as u64 * Locals::SIZE;
            queue.write_buffer(
                &self.locals_uniform_buffer,
                offset,
                bytemuck::cast_slice(&model_mat.to_cols_array()),
            );

            // Bind locals.
            render_pass.set_bind_group(1, &self.locals_bind_group, &[offset as u32]);

            let model_view_inv = (view_mat * model_mat).inverse();
            match renderer.meshes.get(*mesh_hdl) {
                None => {
                    log::error!("Missing mesh {:?}", mesh_hdl);
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

                        // Set push constants.
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                            0,
                            bytemuck::cast_slice(&[model_view_inv.to_cols_array()]),
                        );
                        // Bind material.
                        render_pass.set_bind_group(2, &mtls.bind_group, &[]);
                        // Bind textures.
                        render_pass.set_bind_group(
                            3,
                            texture_bundle.bind_group.as_ref().unwrap(),
                            &[],
                        );
                        let override_material = node
                            .material_override
                            .map(|id| id.min(mtls.n_materials - 1));
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
                                        render_pass.draw(0..mesh.vertex_count, 0..1);
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
                                                0..1,
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
                                            64,
                                            bytemuck::bytes_of(&material_id),
                                        );
                                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
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
                                                64,
                                                bytemuck::bytes_of(&material_id),
                                            );
                                            // Draw the sub-mesh.
                                            render_pass.draw_indexed(
                                                sm.range.start..sm.range.end,
                                                0,
                                                0..1,
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
    }
}
