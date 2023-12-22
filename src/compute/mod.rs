use std::num::NonZeroU64;

use wgpu::BindGroupLayoutEntry;

use crate::{
    core::{
        mesh::{MeshBundle, VertexAttribute},
        FxHashMap, FxHashSet,
    },
    render::{
        rpass::{LocalsBindGroup, ShadowPassLocals},
        Renderer,
    },
    scene::{NodeIdx, Scene},
};

pub const MAX_SUN_POSITIONS_NUM: usize = 16;

pub struct SunlightScore {
    /// The occlusion map for each of the 11 sun positions.
    light_maps: wgpu::Texture,
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
    scores_buffer: wgpu::Buffer,
    /// Scores bind group.
    scores_bind_group: wgpu::BindGroup,
    // /// Pipeline computing the sunlight score.
    // cpass_pipeline: wgpu::ComputePipeline,
    // /// The bind group containing the occlusion map used for computing.
    // cpass_light_maps_bind_group: wgpu::BindGroup,
}

impl SunlightScore {
    /// Creates a new sunlight score compute.
    pub fn new(device: &wgpu::Device) -> Self {
        let scores_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scores_buffer"),
            size: (MAX_SUN_POSITIONS_NUM * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let scores_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scores_bind_group"),
            layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scores_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new((MAX_SUN_POSITIONS_NUM * 4) as u64),
                    },
                    count: None,
                }],
            }),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scores_buffer.as_entire_binding(),
            }],
        });

        let rpass_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light_matrices_buffer"),
            size: (MAX_SUN_POSITIONS_NUM * 4 * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
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

        let light_maps = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light_maps"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: MAX_SUN_POSITIONS_NUM as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
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
        // let cpass_light_maps_bg_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("light_maps_bind_group_layout"),
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::COMPUTE,
        //             ty: wgpu::BindingType::StorageTexture {
        //                 access: wgpu::StorageTextureAccess::ReadOnly,
        //                 format: wgpu::TextureFormat::R32Uint,
        //                 view_dimension: wgpu::TextureViewDimension::D2Array,
        //             },
        //             count: None,
        //         }],
        //     });

        // let cpass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("compute_shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("score.wgsl").into()),
        // });

        // let compute_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("compute_pipeline_layout"),
        //         bind_group_layouts: &[&cpass_light_maps_bg_layout],
        //         push_constant_ranges: &[],
        //     });

        // let compute_pipeline =
        // device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        //     label: Some("compute_pipeline"),
        //     layout: Some(&compute_pipeline_layout),
        //     module: &cpass_shader,
        //     entry_point: "main",
        // });

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
                    range: 0..8,
                }],
            });

        let rpass_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&rpass_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &rpass_shader,
                entry_point: "vs_main",
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
                entry_point: "fs_main",
                targets: &[],
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
        });

        Self {
            light_maps,
            rpass_pipeline,
            rpass_light_maps_bind_group,
            scores_buffer,
            scores_bind_group,
            rpass_light_buffer,
            rpass_light_bind_group,
            rpass_locals_bind_group,
            // cpass_light_maps_bind_group,
            // cpass_pipeline: compute_pipeline,
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

        let mesh_buffer = renderer.meshes.buffer();
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_occlusion_map_rpass"),
            color_attachments: &[],
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
                bytemuck::cast_slice(&[i as u32]),
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
                                        rpass.draw_indexed(0..mesh.index_count, 0, 0..*inst_count);
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
        })
    }

    fn compute_sunlight_scores(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        todo!()
    }

    pub fn compute<'a, M>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        renderer: &Renderer,
        meshes: M,
    ) -> Vec<f32>
    where
        M: Iterator<Item = (&'a MeshBundle, &'a NodeIdx)>,
    {
        self.render_occlusion_maps(device, queue, encoder, scene, renderer, meshes);
        self.compute_sunlight_scores(device, queue);
        Vec::new()
    }
}
