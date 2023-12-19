use crate::{
    core::{
        camera::Camera,
        mesh::{MeshBundle, VertexAttribute},
        FxHashSet, GpuMaterial, Light,
    },
    render::{
        rpass::{
            BlinnPhongRenderPass, Globals, GlobalsBindGroup, GpuLight, InstanceLocals, LightArray,
            LightsBindGroup, Locals, LocalsBindGroup, PConsts, RenderingPass, ShadowMaps,
            ShadowPassLocals, DEPTH_FORMAT,
        },
        PipelineId, PipelineKind, Pipelines, RenderParams, RenderTarget, Renderer,
    },
    scene::{NodeIdx, Nodes, Scene},
};
use glam::{Mat4, Vec3};
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

impl<L: InstanceLocals> LocalsBindGroup<L> {
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
            size: L::SIZE as u64 * Self::INITIAL_INSTANCE_CAPACITY as u64,
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
            _marker: Default::default(),
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
        let size = new_capacity as u64 * L::SIZE as u64;
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
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: LightArray::BUFFER_SIZE,
                },
                count: None,
            }],
        });

        // Preallocate a buffer for lights.
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blph_lights_buffer"),
            size: LightArray::SIZE as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blph_lights_bind_group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
        });

        Self {
            group: bind_group,
            layout,
            lights_buffer,
            lights: LightArray::default(),
        }
    }

    /// Updates the cached light data in the bind group,
    /// and updates the light buffers.
    pub fn update_lights(
        &mut self,
        lights: &[(&Light, &NodeIdx)],
        nodes: &Nodes,
        queue: &wgpu::Queue,
    ) {
        self.lights.clear();
        for (light, node_idx) in lights {
            let len = self.lights.len[0] as usize;
            self.lights.lights[len] = match light {
                Light::Directional { direction, color } => {
                    // In shader, the light direction is the opposite of the
                    // actual direction.
                    let rev_dir = -direction.normalize();
                    GpuLight {
                        dir_or_pos: [10.0 * rev_dir.x, 10.0 * rev_dir.y, 10.0 * rev_dir.z, 0.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                        w2l: (Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 0.1, 500.0)
                            * Mat4::look_at_rh(rev_dir, Vec3::ZERO, Vec3::Y))
                        .to_cols_array(),
                    }
                }
                Light::Point { color } => {
                    let transform = nodes.world(**node_idx);
                    let position = transform.translation;
                    GpuLight {
                        dir_or_pos: [position.x, position.y, position.z, 1.0],
                        color: [color.r as f32, color.g as f32, color.b as f32, 1.0],
                        w2l: (Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0)
                            * Mat4::look_at_rh(position, Vec3::ZERO, Vec3::Y))
                        .to_cols_array(),
                    }
                }
            };
            self.lights.len[0] += 1;
        }
        // Update light buffers.
        queue.write_buffer(&self.lights_buffer, 0, bytemuck::bytes_of(&self.lights));
    }
}

impl BlinnPhongRenderPass {
    /// Creates a new blinn-phong shading render pass.
    pub fn new(device: &wgpu::Device, limits: &wgpu::Limits, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shading_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blph.wgsl").into()),
        });

        let globals_bind_group = GlobalsBindGroup::new(device);
        let locals_bind_group = LocalsBindGroup::new(device);
        let shadow_pass_locals_bind_group = LocalsBindGroup::new(device);

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

        let mut pipelines = Pipelines::new();

        // Create shadow maps pass pipeline. This pipeline is used to evaluate
        // shadow maps for all meshes that cast shadows.
        {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shadow_maps_shader_module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blinn_phong_shadow_maps_pipeline_layout"),
                bind_group_layouts: &[&locals_bind_group.layout, &lights_bind_group.layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    // NOTE: The size of the push constant is the same as in the main render pass.
                    // But the second u32 is used to store the index of the light in the light
                    // array.
                    range: 0..PConsts::SIZE as u32,
                }],
            });
            let (id, pipeline) =
                Self::create_shadow_maps_pass_pipeline(device, &layout, &shader_module);
            pipelines.insert("shadow", id, pipeline);
        }

        let shadow_maps = ShadowMaps::new(device, limits, 1024, 1024, 1);

        // Create main render pass pipeline.
        {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blinn_phong_shading_pipeline_layout"),
                bind_group_layouts: &[
                    &globals_bind_group.layout,
                    &locals_bind_group.layout,
                    &materials_bind_group_layout,
                    &textures_bind_group_layout,
                    &lights_bind_group.layout,
                    &shadow_maps.bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: 0..PConsts::SIZE as u32,
                }],
            });

            for cull_mode in [Some(wgpu::Face::Back), None] {
                for polygon_mode in [wgpu::PolygonMode::Fill, wgpu::PolygonMode::Line] {
                    let (id, pipeline) = Self::create_main_render_pass_pipeline(
                        device,
                        &layout,
                        format,
                        &shader_module,
                        polygon_mode,
                        cull_mode,
                    );
                    pipelines.insert("entity", id, pipeline);
                }
            }
        }

        Self {
            depth_att: None,
            globals_bind_group,
            locals_bind_group,
            shadow_pass_locals_bind_group,
            materials_bind_group_layout,
            textures_bind_group_layout,
            lights_bind_group,
            pipelines,
            shadow_maps,
        }
    }

    /// Evaluates shadow maps.
    fn eval_shadow_maps_pass<'a, M>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mesh_bundles: M,
        scene: &Scene,
        renderer: &Renderer,
    ) where
        M: Iterator<Item = (&'a MeshBundle, &'a NodeIdx)>,
    {
        profiling::scope!("BlinnPhongShading::eval_shadow_maps_pass");
        let (_, pipeline) = &self.pipelines.get_all("shadow").unwrap()[0];

        // Process meshes;
        let mut unique_bundles = FxHashSet::default();
        let mut n_inst = 0;
        for (bundle, _) in mesh_bundles {
            unique_bundles.insert(bundle);
            n_inst += 1;
        }

        log::debug!(
            "{} instances of {} meshes cast shadows",
            n_inst,
            unique_bundles.len()
        );

        if n_inst == 0 {
            return;
        }

        // Update locals buffer content.
        self.shadow_pass_locals_bind_group
            .resize(&renderer.device, n_inst);
        let mut locals = vec![ShadowPassLocals::identity(); n_inst as usize];
        let mut offsets_and_inst_count = vec![(0, 0); n_inst as usize];
        let mut offset = 0u32;
        for (i, bundle) in unique_bundles.iter().enumerate() {
            let instances = renderer
                .instancing
                .get(bundle)
                .expect("Unreachable! Instancing should be created for all meshes!");
            offsets_and_inst_count[i].0 = offset;
            for (j, node_idx) in instances.iter().enumerate() {
                let node = &scene.nodes[*node_idx];
                if !node.is_visible() {
                    continue;
                }
                offsets_and_inst_count[i].1 += 1;
                locals[offset as usize + j] = ShadowPassLocals {
                    model: scene.nodes.world(*node_idx).to_mat4().to_cols_array(),
                }
            }
            offset += offsets_and_inst_count[i].1;
        }
        renderer.queue.write_buffer(
            &self.shadow_pass_locals_bind_group.buffer,
            0,
            bytemuck::cast_slice(&locals),
        );

        let mesh_buffer = renderer.meshes.buffer();

        for (light_idx, shadow_map) in self.shadow_maps.shadow_map_views.iter().enumerate() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blinn_phong_shadow_maps_pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: shadow_map,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(pipeline);
            // Bind locals.
            render_pass.set_bind_group(0, &self.shadow_pass_locals_bind_group, &[]);
            // Bind lights storage buffer.
            render_pass.set_bind_group(1, &self.lights_bind_group, &[]);
            // Set push constants - light index.
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                4,
                bytemuck::bytes_of(&(light_idx as u32)),
            );

            for (bundle, (offset, inst_count)) in
                unique_bundles.iter().zip(offsets_and_inst_count.iter())
            {
                match renderer.meshes.get(bundle.mesh) {
                    Some(mesh) => {
                        // Bind vertex buffer - position.
                        if let Some(pos_range) =
                            mesh.get_vertex_attribute_range(VertexAttribute::POSITION)
                        {
                            render_pass.set_vertex_buffer(0, mesh_buffer.slice(pos_range.clone()));
                        }
                        // Set push constants - instance base index.
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            0,
                            bytemuck::bytes_of(offset),
                        );

                        match mesh.index_format {
                            Some(index_format) => {
                                render_pass.set_index_buffer(
                                    mesh_buffer.slice(mesh.index_range.clone()),
                                    index_format,
                                );
                                match mesh.sub_meshes.as_ref() {
                                    Some(sub_meshes) => {
                                        for sm in sub_meshes {
                                            render_pass.draw_indexed(
                                                sm.range.start..sm.range.end,
                                                0,
                                                0..*inst_count,
                                            );
                                        }
                                    }
                                    None => {
                                        render_pass.draw_indexed(
                                            0..mesh.index_count,
                                            0,
                                            0..*inst_count,
                                        );
                                    }
                                }
                            }
                            None => match mesh.sub_meshes.as_ref() {
                                Some(sub_meshes) => {
                                    for sm in sub_meshes {
                                        render_pass
                                            .draw(sm.range.start..sm.range.end, 0..*inst_count)
                                    }
                                }
                                None => {
                                    render_pass.draw(0..mesh.vertex_count, 0..*inst_count);
                                }
                            },
                        }
                    }
                    None => {
                        log::error!("Missing mesh {:?}", bundle.mesh);
                        continue;
                    }
                }
            }
        }
    }

    /// Evaluates the main render pass.
    fn eval_main_render_pass<'a>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        meshes: &[(&'a MeshBundle, &'a NodeIdx)],
        scene: &Scene,
        renderer: &Renderer,
        params: &RenderParams,
        target: &RenderTarget,
    ) {
        profiling::scope!("BlinnPhongShading::eval_main_render_pass");
        // Update globals.
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
            renderer.queue.write_buffer(
                &self.globals_bind_group.buffer,
                0,
                bytemuck::bytes_of(&globals),
            );
            (view_mat, camera.background)
        };

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

        // Choose the pipeline.
        let pipeline = self.pipelines.get_all_filtered("entity", |id| {
            let cull_mode = if params.enable_back_face_culling {
                Some(wgpu::Face::Back)
            } else {
                None
            };
            let polygon_mode = if params.enable_wireframe {
                wgpu::PolygonMode::Line
            } else {
                wgpu::PolygonMode::Fill
            };
            id.cull_mode() == cull_mode && id.polygon_mode() == polygon_mode
        });

        match pipeline {
            None => {
                log::error!("Missing pipeline for entity shading!");
                return;
            }
            Some(pipelines) => {
                render_pass.set_pipeline(pipelines[0]);
            }
        }

        // Bind globals.
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
        // Bind shadow maps and sampler.
        render_pass.set_bind_group(5, &self.shadow_maps.bind_group, &[]);

        {
            let mut unique_meshes = FxHashSet::default();
            let mut n_inst = 0;
            for (mesh, _) in meshes {
                unique_meshes.insert(mesh);
                n_inst += 1;
            }

            log::debug!(
                "Processed {} instances of {} meshes",
                n_inst,
                unique_meshes.len()
            );

            if n_inst == 0 {
                return;
            }

            // Resize locals buffer in case the number of instances is larger than
            // the current capacity.
            self.locals_bind_group.resize(&renderer.device, n_inst);
            // Bind instance locals.
            render_pass.set_bind_group(1, &self.locals_bind_group, &[]);
            // Bind lights storage buffer.
            render_pass.set_bind_group(4, &self.lights_bind_group, &[]);

            // Preparing locals for each mesh.
            let mut locals = vec![Locals::identity(); n_inst as usize];
            let mut locals_offset = 0u32;
            // Get the mesh buffer, which contains all vertex attributes.
            let mesh_buffer = renderer.meshes.buffer();
            for bundle in unique_meshes {
                let instances = renderer
                    .instancing
                    .get(bundle)
                    .expect("Unreachable! Instancing should be created for all meshes!");
                let mut inst_count = 0;
                for (i, node_idx) in instances.iter().enumerate() {
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
                    }
                }
                debug_assert!(
                    inst_count > 0,
                    "Unreachable! Only visible nodes will be rendered!"
                );
                // Update push constants: isntance base index.
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    bytemuck::bytes_of(&locals_offset),
                );
                locals_offset += inst_count;
                let inst_range = 0..inst_count;

                let mtls = renderer
                    .material_bundles
                    .get(bundle.aesthetic.materials)
                    .unwrap();
                let texs = renderer
                    .texture_bundles
                    .get(bundle.aesthetic.textures)
                    .unwrap();

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
                            render_pass.set_vertex_buffer(0, mesh_buffer.slice(pos_range.clone()));

                            // Bind vertex buffer - normal.
                            if let Some(normals_range) =
                                mesh.get_vertex_attribute_range(VertexAttribute::NORMAL)
                            {
                                render_pass
                                    .set_vertex_buffer(1, mesh_buffer.slice(normals_range.clone()));
                            }
                            // Bind vertex buffer - uv.
                            if let Some(uv_range) =
                                mesh.get_vertex_attribute_range(VertexAttribute::UV)
                            {
                                render_pass
                                    .set_vertex_buffer(2, mesh_buffer.slice(uv_range.clone()));
                            }
                            // Bind vertex buffer - tangent.
                            if let Some(tangent_range) =
                                mesh.get_vertex_attribute_range(VertexAttribute::TANGENT)
                            {
                                render_pass.set_vertex_buffer(
                                    VertexAttribute::TANGENT.shader_location,
                                    mesh_buffer.slice(tangent_range.clone()),
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
                                                4,
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
                                                    4,
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
                                        mesh_buffer.slice(mesh.index_range.clone()),
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
            renderer.queue.write_buffer(
                &self.locals_bind_group.buffer,
                0,
                bytemuck::cast_slice(&locals),
            );
        }
    }

    fn create_shadow_maps_pass_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
    ) -> (PipelineId, wgpu::RenderPipeline) {
        let id = PipelineId::from_states(
            PipelineKind::Render,
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
            Some(wgpu::Face::Back),
        );
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blinn_phong_shadow_maps_pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
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
                }],
            },
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
            fragment: None,
            multiview: None,
        });
        (id, pipeline)
    }

    fn create_main_render_pass_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        output_format: wgpu::TextureFormat,
        shader_module: &wgpu::ShaderModule,
        polygon_mode: wgpu::PolygonMode,
        cull_mode: Option<wgpu::Face>,
    ) -> (PipelineId, wgpu::RenderPipeline) {
        let id = PipelineId::from_states(
            PipelineKind::Render,
            wgpu::PrimitiveTopology::TriangleList,
            polygon_mode,
            cull_mode,
        );
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                module: shader_module,
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
                cull_mode,
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
        });
        (id, pipeline)
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
        params: &RenderParams,
        scene: &Scene,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        profiling::scope!("BlinnPhongShading::record");

        let mut mesh_bundle_query = <(&MeshBundle, &NodeIdx)>::query();
        let visible_meshes = mesh_bundle_query
            .iter(&scene.world)
            .filter(|(_, node_idx)| scene.nodes[**node_idx].is_visible())
            .collect::<Vec<_>>();

        if visible_meshes.is_empty() {
            // No visible meshes, skip rendering.
            return;
        }

        // Update lights information.
        {
            let mut light_query = <(&Light, &NodeIdx)>::query();
            let active_lights = light_query
                .iter(&scene.world)
                .filter(|(_, node_idx)| scene.nodes[**node_idx].is_active())
                .collect::<Vec<_>>();
            self.lights_bind_group
                .update_lights(&active_lights, &scene.nodes, &renderer.queue);
            self.shadow_maps.update(
                &renderer.device,
                &renderer.limits,
                1024,
                1024,
                active_lights.len() as u32,
            );
        }

        // Resize depth buffer if necessary.
        // The depth buffer is shared by all render passes.
        {
            let need_recreate = match &self.depth_att {
                None => true,
                Some(depth) => target.size != depth.0.size(),
            };

            if need_recreate {
                let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
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

        // // Evaluate shadow maps only if shadows are enabled and wireframe is
        // disabled. if params.casting_shadows() {
        //     let meshes_casting_shadow = mesh_bundle_query
        //         .iter(&scene.world)
        //         .filter(|(_, node_idx)| scene.nodes[**node_idx].cast_shadows());
        //     self.eval_shadow_maps_pass(encoder, meshes_casting_shadow, scene,
        // renderer); }

        // Evaluate the main render pass.
        self.eval_main_render_pass(encoder, &visible_meshes, scene, renderer, params, target);
    }
}
