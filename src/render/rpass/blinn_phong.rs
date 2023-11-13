use crate::{
    core::{
        assets::Handle,
        camera::Camera,
        mesh::{GpuMesh, SubMesh, VertexAttribute},
        Color, GpuMaterial, Material, MaterialBundle,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PushConstants {
    /// Model matrix.
    model: [f32; 16],
    /// Material index.
    material: u32,
}

impl Globals {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    pub ka: [f32; 4],
    pub kd: [f32; 4],
    pub ks: [f32; 4],
    pub ns: f32,
    pub ni: f32,
    pub d: f32,
    pub illum: i32,
    // map_ka: i32,
    // map_kd: i32,
    // map_ks: i32,
    // map_ns: i32,
    // map_d: i32,
    // map_bump: i32,
    // map_disp: i32,
    // map_decal: i32,
    // map_norm: i32,
    // vertex_color: u32,
    // padding: [u32; 2],
}

impl Default for MaterialUniform {
    fn default() -> Self {
        MaterialUniform {
            ka: [1.0, 1.0, 1.0, 0.0],
            kd: [0.6, 0.8, 0.3, 0.0],
            ks: [1.0, 0.0, 0.0, 0.0],
            ns: 0.0,
            ni: 0.0,
            d: 0.0,
            illum: 0,
        }
    }
}

impl MaterialUniform {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    pub fn from_material(mtl: &GpuMaterial) -> Self {
        let ka = mtl.ka.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let kd = mtl.kd.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let ks = mtl.ks.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        Self {
            ka,
            kd,
            ks,
            ns: mtl.ns.unwrap_or(0.0),
            ni: mtl.ni.unwrap_or(0.0),
            d: mtl.opacity.unwrap_or(1.0),
            illum: mtl.illumination_model.unwrap_or(0) as i32,
            // map_ka: value.map_ka.map(|_| 1).unwrap_or(0),
            // map_kd: value.map_kd.map(|_| 1).unwrap_or(0),
            // map_ks: value.map_ks.map(|_| 1).unwrap_or(0),
            // map_ns: value.map_ns.map(|_| 1).unwrap_or(0),
            // map_d: value.map_d.map(|_| 1).unwrap_or(0),
            // map_bump: value.map_bump.map(|_| 1).unwrap_or(0),
            // map_disp: value.map_disp.map(|_| 1).unwrap_or(0),
            // map_decal: value.map_decal.map(|_| 1).unwrap_or(0),
            // map_norm: value.map_norm.map(|_| 1).unwrap_or(0),
            // vertex_color: value.vertex_color as u32,
            // padding: [0; 2],
        }
    }

    pub fn from_material_new(mtl: &Material) -> Self {
        let ka = mtl.ka.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let kd = mtl.kd.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let ks = mtl.ks.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        Self {
            ka,
            kd,
            ks,
            ns: mtl.ns.unwrap_or(0.0),
            ni: mtl.ni.unwrap_or(0.0),
            d: mtl.opacity.unwrap_or(1.0),
            illum: mtl.illumination_model.unwrap_or(0) as i32,
            // map_ka: value.map_ka.map(|_| 1).unwrap_or(0),
            // map_kd: value.map_kd.map(|_| 1).unwrap_or(0),
            // map_ks: value.map_ks.map(|_| 1).unwrap_or(0),
            // map_ns: value.map_ns.map(|_| 1).unwrap_or(0),
            // map_d: value.map_d.map(|_| 1).unwrap_or(0),
            // map_bump: value.map_bump.map(|_| 1).unwrap_or(0),
            // map_disp: value.map_disp.map(|_| 1).unwrap_or(0),
            // map_decal: value.map_decal.map(|_| 1).unwrap_or(0),
            // map_norm: value.map_norm.map(|_| 1).unwrap_or(0),
            // vertex_color: value.vertex_color as u32,
            // padding: [0; 2],
        }
    }
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct BlinnPhongShading {
    pub depth_texture: Option<(wgpu::Texture, wgpu::TextureView)>,

    pub globals_bind_group: wgpu::BindGroup,
    pub globals_uniform_buffer: wgpu::Buffer,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,

    // pub materials_bind_group: wgpu::BindGroup,
    // pub materials_uniform_buffer: wgpu::Buffer,
    pub materials_bind_group_layout: wgpu::BindGroupLayout,

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
                        min_binding_size: wgpu::BufferSize::new(MaterialUniform::SIZE),
                    },
                    count: None,
                }],
            });

        let model_matrix_size = std::mem::size_of::<[f32; 16]>() as u32;
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shading_pipeline_layout"),
            bind_group_layouts: &[&globals_bind_group_layout, &materials_bind_group_layout],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..model_matrix_size,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: model_matrix_size..model_matrix_size + 4,
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shading_pipeline"),
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
            pipeline,
        }
    }
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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

        let mut mesh_query = <(&Handle<GpuMesh>, &NodeIdx, &Handle<MaterialBundle>)>::query();
        let buffer = renderer.meshes.buffer();
        for (mesh_hdl, node_idx, mtls_hdl) in mesh_query.iter(&scene.world) {
            let mtls = renderer.material_bundles.get(*mtls_hdl).unwrap();
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
                        // Bind vertex buffer.
                        render_pass.set_vertex_buffer(0, buffer.slice(pos_range.clone()));
                        // Set push constants.
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            0,
                            bytemuck::cast_slice(&transform.to_cols_array()),
                        );
                        // Bind material.
                        render_pass.set_bind_group(1, &mtls.bind_group, &[]);
                        match mesh.index_format {
                            None => {
                                // No index buffer, draw directly.
                                match mesh.sub_meshes.as_ref() {
                                    None => {
                                        // No sub-meshes, use the default material.
                                        // Update material index.
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::FRAGMENT,
                                            std::mem::size_of::<[f32; 16]>() as u32,
                                            bytemuck::cast_slice(&[0u32]),
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
                                                std::mem::size_of::<[f32; 16]>() as u32,
                                                bytemuck::cast_slice(&[material_idx as u32]),
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
                                            std::mem::size_of::<[f32; 16]>() as u32,
                                            bytemuck::cast_slice(&[0u32]),
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
                                                std::mem::size_of::<[f32; 16]>() as u32,
                                                bytemuck::cast_slice(&[material_idx]),
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
