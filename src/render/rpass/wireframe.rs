use crate::{
    core::{
        assets::Handle,
        camera::Camera,
        mesh::{GpuMesh, VertexAttribute},
        Color,
    },
    render::{rpass::RenderingPass, RenderTarget, Renderer},
    scene::{NodeIdx, Scene},
};
use bytemuck::{Pod, Zeroable};

use legion::IntoQuery;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Globals {
    view: [f32; 16],
    proj: [f32; 16],
}

impl Globals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Wireframe render pass.
pub struct Wireframe {
    pub depth_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
    pub globals_bind_group: wgpu::BindGroup,
    pub globals_uniform_buffer: wgpu::Buffer,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl Wireframe {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wireframe_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("wireframe.wgsl").into()),
        });
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("wireframe_globals_bind_group_layout"),
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
            label: Some("wireframe_globals_uniform_buffer"),
            size: Globals::SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wireframe_globals_bind_group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wireframe_pipeline_layout"),
            bind_group_layouts: &[&globals_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..std::mem::size_of::<[f32; 16]>() as u32,
            }],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wireframe_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line,
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
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        Self {
            depth_texture: None,
            globals_bind_group,
            globals_uniform_buffer,
            globals_bind_group_layout,
            pipeline,
        }
    }
}

impl RenderingPass for Wireframe {
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture = Some((texture, view));
        }

        #[rustfmt::skip]
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
            label: Some("wireframe_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // load: wgpu::LoadOp::Clear(*Renderer::CLEAR_COLOR),
                    load: wgpu::LoadOp::Clear(*Color::PURPLISH_GREY),
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

        let mut mesh_query = <(&Handle<GpuMesh>, &NodeIdx)>::query();
        let buffer = renderer.meshes.buffer();
        for (mesh_handle, node_idx) in mesh_query.iter(&scene.world) {
            match renderer.meshes.get(*mesh_handle) {
                None => {
                    log::error!("Missing mesh {:?}", mesh_handle);
                    continue;
                }
                Some(mesh) => {
                    let transform = scene.nodes.world(*node_idx).to_mat4();
                    if let Some(pos_range) =
                        mesh.get_vertex_attribute_range(VertexAttribute::POSITION)
                    {
                        render_pass.set_vertex_buffer(0, buffer.slice(pos_range.clone()));
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            0,
                            bytemuck::cast_slice(&transform.to_cols_array()),
                        );
                        match mesh.index_format {
                            None => render_pass.draw(0..mesh.vertex_count, 0..1),
                            Some(index_format) => {
                                render_pass.set_index_buffer(
                                    buffer.slice(mesh.index_range.clone()),
                                    index_format,
                                );
                                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                            }
                        }
                    }
                }
            }
        }
    }
}
