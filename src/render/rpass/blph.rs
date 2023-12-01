use crate::{
    core::{
        camera::Camera,
        mesh::{MeshBundle, VertexAttribute},
        FxHashSet, GpuMaterial, Light,
    },
    render::{
        rpass::{
            BlinnPhongRenderPass, DirLight, DirLightArray, Globals, GlobalsBindGroup,
            LightsBindGroup, Locals, LocalsBindGroup, PConsts, PntLight, PntLightArray,
            RenderingPass, DEPTH_FORMAT,
        },
        RenderTarget, Renderer, ShadingMode,
    },
    scene::{NodeIdx, Nodes, Scene},
};
use legion::IntoQuery;
use std::num::{NonZeroU32, NonZeroU64};

impl GlobalsBindGroup {
    /// Creates a new globals bind group.
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blph_globals_bg_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Globals::BUFFER_SIZE,
                },
                count: None,
            }],
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_globals_buffer"),
            size: Globals::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blph_globals_bg"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            group: bind_group,
            layout,
            buffer,
        }
    }
}

impl LocalsBindGroup {
    /// Creates a new locals bind group.
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blph_locals_bg_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Locals::BUFFER_SIZE,
                },
                count: None,
            }],
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_locals_buffer"),
            size: Locals::SIZE as u64 * Self::INITIAL_INSTANCE_CAPACITY as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blph_locals_bg"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            group: bind_group,
            layout,
            buffer,
            capacity: Self::INITIAL_INSTANCE_CAPACITY as u32,
        }
    }

    /// Resize the locals buffer to the capacity greater than or equal to the
    /// given number of instances.
    pub fn resize(&mut self, device: &wgpu::Device, n_instances: u32) {
        if n_instances <= self.capacity {
            log::debug!("No need to resize instance locals buffer");
            return;
        }
        // Calculate the new capacity with an increment of 256 instances.
        let new_capacity = (n_instances / 256 + 1) * 256;
        let size = new_capacity as u64 * Locals::SIZE as u64;
        log::debug!("Resize instance locals buffer to {}", size);
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_locals_buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        log::debug!("Recreate locals bind group");
        self.group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blph_locals_bg"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_entire_binding(),
            }],
        });
        self.capacity = new_capacity;
    }
}

impl LightsBindGroup {
    /// Creates a new lights bind group.
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blph_lights_bg_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: DirLightArray::BUFFER_SIZE,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: PntLightArray::BUFFER_SIZE,
                    },
                    count: None,
                },
            ],
        });

        // Preallocate a buffer for directional lights.
        let dir_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_dir_lights_buffer"),
            size: DirLightArray::SIZE as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Preallocate a buffer for point lights.
        let pnt_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_pnt_lights_buffer"),
            size: PntLightArray::SIZE as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blph_lights_bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dir_lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pnt_lights_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            group: bind_group,
            layout,
            dir_lights_buffer,
            pnt_lights_buffer,
            dir_lights: DirLightArray::default(),
            pnt_lights: PntLightArray::default(),
        }
    }

    /// Updates the cached light data in the bind group,
    /// and updates the light buffers.
    pub fn update_lights<'a, L>(&mut self, lights: L, nodes: &Nodes, queue: &wgpu::Queue)
    where
        L: Iterator<Item = (&'a Light, &'a NodeIdx)>,
    {
        self.dir_lights.clear();
        self.pnt_lights.clear();

        for (light, node_idx) in lights {
            match light {
                Light::Directional { direction, color } => {
                    let len = self.dir_lights.len[0] as usize;
                    self.dir_lights.lights[len] = DirLight {
                        direction: [-direction.x, -direction.y, -direction.z, 0.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                    };
                    self.dir_lights.len[0] += 1;
                }
                Light::Point { color } => {
                    let transform = nodes.world(*node_idx);
                    let position = transform.translation;
                    let len = self.pnt_lights.len[0] as usize;
                    self.pnt_lights.lights[len] = PntLight {
                        position: [position.x, position.y, position.z, 1.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                    };
                    self.pnt_lights.len[0] += 1;
                }
            }
        }
        // Update light buffers.
        queue.write_buffer(
            &self.dir_lights_buffer,
            0,
            bytemuck::bytes_of(&self.dir_lights),
        );
        queue.write_buffer(
            &self.pnt_lights_buffer,
            0,
            bytemuck::bytes_of(&self.pnt_lights),
        );
    }
}

impl BlinnPhongRenderPass {
    /// Creates a new blinn-phong shading render pass.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shading_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blph.wgsl").into()),
        });

        let globals_bind_group = GlobalsBindGroup::new(device);
        let locals_bind_group = LocalsBindGroup::new(device);

        let materials_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shading_materials_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(GpuMaterial::SIZE),
                    },
                    count: None,
                }],
            });

        let textures_bind_group_layout = texture_bundle_bind_group_layout(device);

        let lights_bind_group = LightsBindGroup::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blinn_phong_shading_pipeline_layout"),
            bind_group_layouts: &[
                &globals_bind_group.layout,
                &locals_bind_group.layout,
                &materials_bind_group_layout,
                &textures_bind_group_layout,
                &lights_bind_group.layout,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..PConsts::SIZE as u32,
            }],
        });

        let pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            format,
            &shader_module,
            wgpu::PolygonMode::Fill,
        );

        Self {
            depth_att: None,
            globals_bind_group,
            locals_bind_group,
            materials_bind_group_layout,
            textures_bind_group_layout,
            lights_bind_group,
            entity_pipeline: pipeline,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        output_format: wgpu::TextureFormat,
        shader_module: &wgpu::ShaderModule,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blinn_phong_shading_pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader_module,
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
                            // UV.
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: VertexAttribute::TANGENT.size as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Tangent.
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: VertexAttribute::TANGENT.shader_location,
                                format: VertexAttribute::TANGENT.format,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode,
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
        })
    }
}
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
                count: NonZeroU32::new(BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN as u32),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(
                        std::mem::size_of::<u32>() as u64
                            * BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN as u64,
                    ),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: NonZeroU32::new(BlinnPhongRenderPass::MAX_SAMPLER_ARRAY_LEN as u32),
            },
        ],
    })
}

impl RenderingPass for BlinnPhongRenderPass {
    fn record(
        &mut self,
        renderer: &Renderer,
        target: &RenderTarget,
        scene: &Scene,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        _mode: ShadingMode, // TODO: support shading mode.
    ) {
        profiling::scope!("BlinnPhongShading::record");
        let (view_mat, clear_color) = {
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
                &self.globals_bind_group.buffer,
                0,
                bytemuck::bytes_of(&globals),
            );
            (view_mat, camera.background)
        };

        {
            // (Re-)create depth texture if necessary.
            let need_recreate = match &self.depth_att {
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
                self.depth_att = Some((texture, view));
            }
        }

        // Create render pass.
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blinn_phong_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(*clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_att.as_ref().unwrap().1,
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

        {
            // Update light data.
            let mut light_query = <(&Light, &NodeIdx)>::query();
            let active_lights = light_query
                .iter(&scene.world)
                .filter(|(_, node_idx)| scene.nodes[**node_idx].is_active());
            self.lights_bind_group
                .update_lights(active_lights, &scene.nodes, queue);
            // Bind lights.
            render_pass.set_bind_group(4, &self.lights_bind_group, &[]);
        }

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

        self.locals_bind_group.resize(device, total_inst_count);

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
            for (i, node_idx) in instancing.nodes.iter().enumerate() {
                let node = &scene.nodes[*node_idx];
                if !node.is_visible() {
                    continue;
                }
                inst_count += 1;
                let model_mat = scene.nodes.world(*node_idx).to_mat4();
                locals[locals_offset as usize + i] = Locals {
                    model: model_mat.to_cols_array(),
                    model_view_it: (view_mat * model_mat).inverse().transpose().to_cols_array(),
                    material_index: [
                        node.material_override.unwrap_or(u32::MAX),
                        u32::MAX,
                        u32::MAX,
                        u32::MAX,
                    ],
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
            let texs = renderer.texture_bundles.get(bundle.textures).unwrap();

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
                        // Bind vertex buffer - uv.
                        if let Some(uv_range) = mesh.get_vertex_attribute_range(VertexAttribute::UV)
                        {
                            render_pass.set_vertex_buffer(2, buffer.slice(uv_range.clone()));
                        }
                        // Bind vertex buffer - tangent.
                        if let Some(tangent_range) =
                            mesh.get_vertex_attribute_range(VertexAttribute::TANGENT)
                        {
                            render_pass.set_vertex_buffer(
                                VertexAttribute::TANGENT.shader_location,
                                buffer.slice(tangent_range.clone()),
                            );
                        }

                        // Bind material.
                        render_pass.set_bind_group(2, &mtls.bind_group, &[]);
                        // Bind textures.
                        render_pass.set_bind_group(3, texs.bind_group.as_ref().unwrap(), &[]);

                        match mesh.index_format {
                            None => {
                                // No index buffer, draw directly.
                                match mesh.sub_meshes.as_ref() {
                                    None => {
                                        // No sub-meshes, use the default material.
                                        // Update material index.
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                                            64,
                                            bytemuck::bytes_of(&0u32),
                                        );
                                        render_pass.draw(0..mesh.vertex_count, inst_range);
                                    }
                                    Some(sub_meshes) => {
                                        // Draw each sub-mesh.
                                        for sm in sub_meshes {
                                            let material_id =
                                                sm.material.unwrap_or(mtls.n_materials - 1);
                                            // Update material index.
                                            render_pass.set_push_constants(
                                                wgpu::ShaderStages::VERTEX_FRAGMENT,
                                                64,
                                                bytemuck::bytes_of(&material_id),
                                            );
                                            render_pass.draw(
                                                sm.range.start..sm.range.end,
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
                                        render_pass.set_push_constants(
                                            wgpu::ShaderStages::VERTEX_FRAGMENT,
                                            4,
                                            bytemuck::bytes_of(&0u32),
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
                                                sm.material.unwrap_or(mtls.n_materials - 1);
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
            &self.locals_bind_group.buffer,
            0,
            bytemuck::cast_slice(&locals),
        );
    }
}
