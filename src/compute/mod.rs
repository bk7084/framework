use std::num::NonZeroU64;

use glam::{Mat3, Mat4, Vec3};
use wgpu::{util::DeviceExt, BindGroupLayoutEntry};

use crate::{
    core::{
        mesh::{MeshBundle, VertexAttribute},
        FxHashSet,
    },
    render::{
        rpass::{LocalsBindGroup, PConstsShadowPass, ShadowPassLocals},
        Renderer,
    },
    scene::{NodeIdx, Scene},
};

pub const MAX_SUN_POSITIONS_NUM: usize = 16;

pub struct SunlightScore {
    /// The occlusion map for each of the 11 sun positions.
    light_maps: wgpu::Texture,
    /// Occlusion map pipeline output (only for satisfying the pipeline layout)
    rpass_output: wgpu::Texture,
    /// Pipeline generating the occlusion map.
    rpass_pipeline: wgpu::RenderPipeline,
    /// The bind group containing the occlusion map used for rendering.
    rpass_light_maps_bind_group: wgpu::BindGroup,
    /// The buffer containing the light space matrices.
    rpass_light_buffer: wgpu::Buffer,
    /// Light space matrices bind group.
    rpass_light_bind_group: wgpu::BindGroup,
    /// The buffer containing the local transform matrices.
    rpass_locals_bind_group: LocalsBindGroup<ShadowPassLocals>,
    /// The buffer containing the final sunlight scores.
    cpass_scores_buffer: wgpu::Buffer,
    /// Scores bind group.
    cpass_scores_bind_group: wgpu::BindGroup,
    /// Pipeline computing the sunlight score.
    cpass_pipeline: wgpu::ComputePipeline,
    /// The bind group containing the occlusion map used for computing.
    cpass_light_maps_bind_group: wgpu::BindGroup,
    /// Scores for each sun position.
    scores: [f32; MAX_SUN_POSITIONS_NUM],
    #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
    pub storage_buffer: wgpu::Buffer,
    #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
    pub output_storage_buffer: wgpu::Buffer,
}

impl SunlightScore {
    pub const LIGHT_MAP_LAYER_COLS: u32 = 1024;
    pub const LIGHT_MAP_LAYER_ROWS: u32 = 1024;
    pub const LIGHT_MAP_LAYER_PIXEL_COUNT: u32 =
        Self::LIGHT_MAP_LAYER_COLS * Self::LIGHT_MAP_LAYER_ROWS;
    pub const LIGHT_MAP_LAYER_SIZE: u32 = Self::LIGHT_MAP_LAYER_PIXEL_COUNT * 4;

    /// Creates a new sunlight score compute.
    pub fn new(device: &wgpu::Device) -> Self {
        let cpass_scores_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cpass_scores_buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[2.0f32; MAX_SUN_POSITIONS_NUM]),
        });
        let cpass_scores_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cpass_scores_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let cpass_scores_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cpass_scores_bind_group"),
            layout: &cpass_scores_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cpass_scores_buffer.as_entire_binding(),
            }],
        });

        const ORTHO_NEAR: f32 = -80.0;
        const ORTHO_FAR: f32 = 80.0;
        const ORTHO_H: f32 = 40.0;
        const ORTHO_W: f32 = 40.0;
        // Sun's light space matrices at each of the 11 positions.
        let mut light_matrices = [[0f32; 16]; 16];
        let inclination = std::f32::consts::FRAC_PI_8;
        let center_pos = Vec3::new(0.0, inclination.cos(), inclination.sin());
        for i in 0..11 {
            let angle = (i as f32 - 5.0) * std::f32::consts::FRAC_PI_6 * 0.5;
            let pos = Mat3::from_rotation_z(angle) * center_pos;
            light_matrices[i] = (Mat4::orthographic_rh(
                -ORTHO_W, ORTHO_W, -ORTHO_H, ORTHO_H, ORTHO_NEAR, ORTHO_FAR,
            ) * Mat4::look_at_rh(pos, Vec3::ZERO, Vec3::Y))
            .to_cols_array();
        }
        let rpass_light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_matrices_buffer"),
            contents: bytemuck::cast_slice(&light_matrices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let rpass_light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("light_matrices_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new((MAX_SUN_POSITIONS_NUM * 4 * 16) as u64),
                    },
                    count: None,
                }],
            });
        let rpass_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_matrices_bind_group"),
            layout: &rpass_light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rpass_light_buffer.as_entire_binding(),
            }],
        });

        let rpass_locals_bind_group = LocalsBindGroup::new(device);

        let rpass_output = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rpass_output"),
            size: wgpu::Extent3d {
                width: Self::LIGHT_MAP_LAYER_COLS,
                height: Self::LIGHT_MAP_LAYER_ROWS,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
        let output_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output_storage_buffer"),
            size: (Self::LIGHT_MAP_LAYER_COLS * Self::LIGHT_MAP_LAYER_ROWS * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("storage_buffer_sunlight_map"),
            size: Self::LIGHT_MAP_LAYER_SIZE as u64 * MAX_SUN_POSITIONS_NUM as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let light_maps = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light_maps"),
            size: wgpu::Extent3d {
                width: Self::LIGHT_MAP_LAYER_COLS,
                height: Self::LIGHT_MAP_LAYER_ROWS,
                depth_or_array_layers: MAX_SUN_POSITIONS_NUM as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let light_maps_view = light_maps.create_view(&wgpu::TextureViewDescriptor {
            label: Some("light_maps_view"),
            format: Some(wgpu::TextureFormat::R32Uint),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_array_layer: 0,
            array_layer_count: Some(MAX_SUN_POSITIONS_NUM as u32),
            ..Default::default()
        });
        let rpass_light_maps_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("light_maps_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::R32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                }],
            });
        let rpass_light_maps_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_maps_bind_group"),
            layout: &rpass_light_maps_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&light_maps_view),
            }],
        });
        let cpass_light_maps_bg_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cpass_light_maps_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::R32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                }],
            });
        let cpass_light_maps_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cpass_light_maps_bind_group"),
            layout: &cpass_light_maps_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&light_maps_view),
            }],
        });

        let cpass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("score.wgsl").into()),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&cpass_scores_bg_layout, &cpass_light_maps_bg_layout],
                push_constant_ranges: &[],
            });

        let cpass_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cpass_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let rpass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sunlight_score_rpass_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lightmap.wgsl").into()),
        });

        let rpass_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    &rpass_light_maps_bg_layout,
                    &rpass_light_bind_group_layout,
                    &rpass_locals_bind_group.layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: 0..PConstsShadowPass::SIZE as u32,
                }],
            });

        let rpass_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&rpass_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &rpass_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &rpass_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            light_maps,
            rpass_pipeline,
            rpass_light_maps_bind_group,
            cpass_scores_buffer,
            cpass_scores_bind_group,
            rpass_light_buffer,
            rpass_light_bind_group,
            rpass_locals_bind_group,
            #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
            storage_buffer,
            #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
            output_storage_buffer,
            rpass_output,
            cpass_light_maps_bind_group,
            cpass_pipeline,
            scores: [0.0; MAX_SUN_POSITIONS_NUM],
        }
    }

    #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
    pub fn write_sunlight_maps(&mut self, device: &wgpu::Device) {
        {
            let buffer_slice = self.storage_buffer.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            device.poll(wgpu::Maintain::Wait);
            pollster::block_on(async {
                receiver.recv_async().await.unwrap().unwrap();
            });
            {
                let buffer_view = buffer_slice.get_mapped_range();
                let (_, data, _) = unsafe { buffer_view.align_to::<u32>() };
                let mut imgbuf =
                    image::ImageBuffer::new(Self::LIGHT_MAP_LAYER_COLS, Self::LIGHT_MAP_LAYER_ROWS);
                for i in 0..MAX_SUN_POSITIONS_NUM {
                    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                        let idx = (y * Self::LIGHT_MAP_LAYER_COLS + x) as usize;
                        let val = data[idx + i * Self::LIGHT_MAP_LAYER_PIXEL_COUNT as usize];
                        *pixel = image::Luma([val as u8]);
                    }
                    imgbuf.save(format!("sunlight_map_{:02}.png", i)).unwrap();
                    imgbuf.enumerate_pixels_mut().for_each(|(_, _, pixel)| {
                        *pixel = image::Luma([0u8]);
                    });
                }
            }
            self.storage_buffer.unmap();
        }

        {
            let buffer_slice = self.output_storage_buffer.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            device.poll(wgpu::Maintain::Wait);
            pollster::block_on(async {
                receiver.recv_async().await.unwrap().unwrap();
            });
            {
                let buffer_view = buffer_slice.get_mapped_range();
                let (_, data, _) = unsafe { buffer_view.align_to::<u8>() };
                let mut imgbuf =
                    image::ImageBuffer::new(Self::LIGHT_MAP_LAYER_COLS, Self::LIGHT_MAP_LAYER_ROWS);
                for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    let idx = (y * Self::LIGHT_MAP_LAYER_COLS + x) as usize * 4;
                    let mut val = [0u8; 4];
                    for i in 0..4 {
                        val[i] = data[idx + i];
                    }
                    *pixel = image::Rgba(val);
                }
                imgbuf.save(format!("sunlight_output.png")).unwrap();
            }
            self.output_storage_buffer.unmap();
        }
    }

    /// Renders the occlusion map for the given sun positions.
    fn render_occlusion_maps<'a, M>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
        renderer: &Renderer,
        mesh_bundles: M,
    ) where
        M: Iterator<Item = (&'a MeshBundle, &'a NodeIdx)>,
    {
        profiling::scope!("render_occlusion_maps");
        log::debug!("Render sunlight occlusion maps");
        let mut unique_bundles = FxHashSet::default();
        let mut n_inst = 0;
        for (bundle, _) in mesh_bundles {
            unique_bundles.insert(bundle);
            n_inst += 1;
        }

        log::debug!(
            "Rendering occlusion maps of {} instances of {} visible meshes",
            n_inst,
            unique_bundles.len()
        );

        if n_inst == 0 {
            return;
        }

        // Resizes the locals bind group.
        self.rpass_locals_bind_group.resize(device, n_inst);
        // Preparing the data for the locals bind group.
        let mut locals = vec![ShadowPassLocals::identity(); n_inst as usize];
        let mut offsets_and_inst_counts = vec![(0, 0); n_inst as usize];
        let mut offset = 0u32;
        for (i, bundle) in unique_bundles.iter().enumerate() {
            let instances = renderer
                .instancing
                .get(bundle)
                .expect("Unreachable: the bundle is in the unique bundles list");
            offsets_and_inst_counts[i].0 = offset;
            for (j, node_idx) in instances.iter().enumerate() {
                let node = &scene.nodes[*node_idx];
                if !node.is_visible() {
                    continue;
                }
                offsets_and_inst_counts[i].1 += 1;
                locals[offset as usize + j] = ShadowPassLocals {
                    model: scene.nodes.world(*node_idx).to_mat4().to_cols_array(),
                }
            }
            offset += offsets_and_inst_counts[i].1;
        }
        queue.write_buffer(
            &self.rpass_locals_bind_group.buffer,
            0,
            bytemuck::cast_slice(&locals),
        );
        // Clearing the light maps.
        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &self.light_maps,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8; (Self::LIGHT_MAP_LAYER_SIZE as usize) * MAX_SUN_POSITIONS_NUM],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * Self::LIGHT_MAP_LAYER_COLS),
                rows_per_image: Some(Self::LIGHT_MAP_LAYER_ROWS),
            },
            wgpu::Extent3d {
                width: Self::LIGHT_MAP_LAYER_COLS,
                height: Self::LIGHT_MAP_LAYER_ROWS,
                depth_or_array_layers: MAX_SUN_POSITIONS_NUM as u32,
            },
        );

        let mesh_buffer = renderer.meshes.buffer();
        let output_view = self
            .rpass_output
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_occlusion_map_rpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.rpass_pipeline);
            rpass.set_bind_group(0, &self.rpass_light_maps_bind_group, &[]);
            rpass.set_bind_group(1, &self.rpass_light_bind_group, &[]);
            rpass.set_bind_group(2, &self.rpass_locals_bind_group, &[]);

            // Rendering the occlusion maps for each sun position.
            (0..11).into_iter().for_each(|i| {
                profiling::scope!("render_occlusion_map_rpass");
                rpass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    4,
                    bytemuck::bytes_of(&(i as u32)),
                );

                for (bundle, (offset, inst_count)) in
                    unique_bundles.iter().zip(offsets_and_inst_counts.iter())
                {
                    match renderer.meshes.get(bundle.mesh) {
                        Some(mesh) => {
                            // Bind vertex buffer - position.
                            if let Some(pos_range) =
                                mesh.get_vertex_attribute_range(VertexAttribute::POSITION)
                            {
                                rpass.set_vertex_buffer(0, mesh_buffer.slice(pos_range.clone()));
                            }
                            // Set push constants - instance base index.
                            rpass.set_push_constants(
                                wgpu::ShaderStages::VERTEX_FRAGMENT,
                                0,
                                bytemuck::bytes_of(offset),
                            );

                            match mesh.index_format {
                                Some(index_format) => {
                                    rpass.set_index_buffer(
                                        mesh_buffer.slice(mesh.index_range.clone()),
                                        index_format,
                                    );
                                    match mesh.sub_meshes.as_ref() {
                                        Some(sub_meshes) => {
                                            for sm in sub_meshes {
                                                rpass.draw_indexed(
                                                    sm.range.start..sm.range.end,
                                                    0,
                                                    0..*inst_count,
                                                );
                                            }
                                        }
                                        None => {
                                            rpass.draw_indexed(
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
                                            rpass.draw(sm.range.start..sm.range.end, 0..*inst_count)
                                        }
                                    }
                                    None => {
                                        rpass.draw(0..mesh.vertex_count, 0..*inst_count);
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
            });
        }

        // Copying the occlusion maps and the output to the corresponding storage
        // buffer.
        #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
        {
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &self.light_maps,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.storage_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * Self::LIGHT_MAP_LAYER_COLS),
                        rows_per_image: Some(Self::LIGHT_MAP_LAYER_ROWS),
                    },
                },
                wgpu::Extent3d {
                    width: Self::LIGHT_MAP_LAYER_COLS,
                    height: Self::LIGHT_MAP_LAYER_ROWS,
                    depth_or_array_layers: MAX_SUN_POSITIONS_NUM as u32,
                },
            );

            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &self.rpass_output,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.output_storage_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * Self::LIGHT_MAP_LAYER_COLS),
                        rows_per_image: Some(Self::LIGHT_MAP_LAYER_ROWS),
                    },
                },
                wgpu::Extent3d {
                    width: Self::LIGHT_MAP_LAYER_COLS,
                    height: Self::LIGHT_MAP_LAYER_ROWS,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn compute_sunlight_scores(&self, encoder: &mut wgpu::CommandEncoder) {
        profiling::scope!("compute_sunlight_scores");
        log::debug!("Compute sunlight scores");
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute_sunlight_scores_cpass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.cpass_pipeline);
        cpass.set_bind_group(0, &self.cpass_scores_bind_group, &[]);
        cpass.set_bind_group(1, &self.cpass_light_maps_bind_group, &[]);
        cpass.dispatch_workgroups(MAX_SUN_POSITIONS_NUM as u32, 1, 1);
    }

    fn read_scores(&mut self, device: &wgpu::Device) {
        let buffer_slice = self.cpass_scores_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
        device.poll(wgpu::Maintain::Wait);
        pollster::block_on(async {
            receiver.recv_async().await.unwrap().unwrap();
        });
        {
            let buffer_view = buffer_slice.get_mapped_range();
            self.scores
                .copy_from_slice(bytemuck::cast_slice(&buffer_view));
        }
        self.cpass_scores_buffer.unmap();
    }

    pub fn compute<'a, M>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        renderer: &Renderer,
        meshes: M,
    ) -> Vec<f32>
    where
        M: Iterator<Item = (&'a MeshBundle, &'a NodeIdx)>,
    {
        {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("occlusion_map_encoder"),
            });
            self.render_occlusion_maps(device, queue, &mut encoder, scene, renderer, meshes);
            self.compute_sunlight_scores(&mut encoder);
            queue.submit(std::iter::once(encoder.finish()));
        }

        #[cfg(all(debug_assertions, feature = "debug-sunlight-map"))]
        self.write_sunlight_maps(device);
        self.read_scores(device);

        return self.scores.to_vec();
    }
}
