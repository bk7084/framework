use crate::{
    core::{
        assets::Handle,
        camera::Camera,
        mesh::{GpuMesh, SubMesh, VertexAttribute},
        Color, GpuMaterial, Material, MaterialBundle, TextureBundle,
    },
    render::{rpass::RenderingPass, Pipelines, RenderTarget, Renderer},
    scene::{NodeIdx, Scene},
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use legion::IntoQuery;
use std::num::NonZeroU32;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Globals {
    view: [f32; 16],
    proj: [f32; 16],
}

/// Push constants for the shading pipeline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PConsts {
    /// Model matrix.
    model: [f32; 16],
    /// Material index.
    material: u32,
}

impl PConsts {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const MODEL_OFFSET: wgpu::BufferAddress = 0;
    pub const MODEL_SIZE: wgpu::BufferAddress =
        std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress;
    pub const MATERIAL_OFFSET: wgpu::BufferAddress = Self::MODEL_OFFSET + Self::MODEL_SIZE;
    pub const MATERIAL_SIZE: wgpu::BufferAddress =
        std::mem::size_of::<u32>() as wgpu::BufferAddress;
}

static_assertions::assert_eq_align!(PConsts, [f32; 17]);
static_assertions::assert_eq_size!(PConsts, [f32; 17]);

impl Globals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub struct BlinnPhongShading {
    pub depth_texture: Option<(wgpu::Texture, wgpu::TextureView)>,

    pub globals_bind_group: wgpu::BindGroup,
    pub globals_uniform_buffer: wgpu::Buffer,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,

    pub materials_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,

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
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blinn_phong_shading_pipeline_layout"),
            bind_group_layouts: &[
                &globals_bind_group_layout,
                &materials_bind_group_layout,
                &textures_bind_group_layout,
            ],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: PConsts::MODEL_OFFSET as u32
                        ..PConsts::MODEL_OFFSET as u32 + PConsts::MODEL_SIZE as u32,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: PConsts::MATERIAL_OFFSET as u32
                        ..PConsts::MATERIAL_OFFSET as u32 + PConsts::MATERIAL_SIZE as u32,
                },
            ],
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
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
            materials_bind_group_layout,
            textures_bind_group_layout,
            pipeline,
        }
    }
}

/// Maximum number of textures in a texture array.
pub const MAX_TEXTURE_ARRAY_LEN: u32 = 128;

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
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: NonZeroU32::new(MAX_TEXTURE_ARRAY_LEN),
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

        // Update globals.
        let mut camera_query = <(&Camera, &NodeIdx)>::query();
        // TODO: support multiple cameras.
        for (camera, node_idx) in camera_query.iter(&scene.world) {
            if camera.is_main {
                let view = scene.nodes.inverse_world(*node_idx).to_mat4();
                let proj = camera.proj_matrix(target.aspect_ratio());
                let globals = Globals {
                    view: view.to_cols_array(),
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

        let mut mesh_query = <(
            &Handle<GpuMesh>,
            &NodeIdx,
            &Handle<MaterialBundle>,
            &Handle<TextureBundle>,
        )>::query();
        let buffer = renderer.meshes.buffer();
        for (mesh_hdl, node_idx, materials_hdl, textures_hdl) in mesh_query.iter(&scene.world) {
            let mtls = renderer.material_bundles.get(*materials_hdl).unwrap();
            let texture_bundle = renderer.texture_bundles.get(*textures_hdl).unwrap();
            let transform = scene.nodes.world(*node_idx).to_mat4();
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
                            wgpu::ShaderStages::VERTEX,
                            PConsts::MODEL_OFFSET as u32,
                            bytemuck::cast_slice(&transform.to_cols_array()),
                        );
                        // Bind material.
                        render_pass.set_bind_group(1, &mtls.bind_group, &[]);
                        // Bind textures.
                        render_pass.set_bind_group(
                            2,
                            texture_bundle.bind_group.as_ref().unwrap(),
                            &[],
                        );
                        match mesh.index_format {
                            None => {
                                // No index buffer, draw directly.
                                match mesh.sub_meshes.as_ref() {
                                    None => {
                                        // No sub-meshes, use the default material.
                                        // Update material index.
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::FRAGMENT,
                                            PConsts::MATERIAL_OFFSET as u32,
                                            bytemuck::bytes_of(&0u32),
                                        );
                                        render_pass.draw(0..mesh.vertex_count, 0..1);
                                    }
                                    Some(sub_meshes) => {
                                        // Draw each sub-mesh.
                                        for sub_mesh in sub_meshes {
                                            let material_idx =
                                                sub_mesh.material.unwrap_or(mtls.n_materials - 1);
                                            // Update material index.
                                            render_pass.set_push_constants(
                                                wgpu::ShaderStages::FRAGMENT,
                                                PConsts::MATERIAL_OFFSET as u32,
                                                bytemuck::bytes_of(&material_idx),
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
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::FRAGMENT,
                                            PConsts::MATERIAL_OFFSET as u32,
                                            bytemuck::bytes_of(&0u32),
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
                                            let material_idx =
                                                sm.material.unwrap_or(mtls.n_materials - 1);
                                            // Update material index.
                                            render_pass.set_push_constants(
                                                wgpu::ShaderStages::FRAGMENT,
                                                PConsts::MATERIAL_OFFSET as u32,
                                                bytemuck::bytes_of(&material_idx),
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
