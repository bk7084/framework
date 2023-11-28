use crate::render::rpass::{Globals, DEPTH_FORMAT};

/// A skybox environment map.
///
/// This is a cube map texture that is used to render the skybox. It is either
/// created from a set of 6 images or from a single equirectangular image
/// (compute shader involved).
pub struct EnvironmentMap {
    /// The cube map texture of 6 layers stored in the order of
    /// +X, -X, +Y, -Y, +Z, -Z.
    pub texture: wgpu::Texture,
    /// The texture view of the cube map texture.
    pub view: wgpu::TextureView,
    /// The sampler of the cube map texture.
    pub sampler: wgpu::Sampler,
}

impl EnvironmentMap {
    /// Creates a new environment map from a set of 6 images.
    pub fn new_from_images(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        images: [image::RgbaImage; 6],
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("environment_map"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 6,
            },
            mip_level_count: 4,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("environment_map_view"),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("environment_map_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&images[0]),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 6,
            },
        );

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Creates a new environment map from a single equirectangular image.
    pub fn new_from_equirectangular(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        image: image::RgbaImage,
    ) -> Self {
        todo!()
    }
}

pub struct SkyboxRenderPass<'a> {
    /// The bind group layout of the skybox render pass.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// The bind group containing the uniform buffer and the environment map.
    pub bind_group: wgpu::BindGroup,
    /// Uniform buffer containing the `Globals` uniform struct.
    pub globals: &'a wgpu::Buffer,
    /// The environment map.
    pub env_map: EnvironmentMap,
    /// The pipeline layout of the skybox render pass.
    pub pipeline_layout: wgpu::PipelineLayout,
    /// The pipeline of the skybox render pass.
    pub pipeline: wgpu::RenderPipeline,
}

impl<'a> SkyboxRenderPass<'a> {
    /// Creates a new skybox render pass.
    ///
    /// # Arguments
    ///
    /// * `globals` - The global uniform buffer, which contains the view and
    ///   projection matrices.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        globals: &'a wgpu::Buffer,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        // let env_map = EnvironmentMap::new_from_images(
        //     device,
        //     queue,
        //     1024,
        //     1024,
        //     [
        //         image::load_from_memory(include_bytes!("../../../data/skybox/right.
        // jpg"))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //         image::load_from_memory(include_bytes!("../../../data/skybox/left.
        // jpg"))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //         image::load_from_memory(include_bytes!("../../../data/skybox/top.jpg"
        // ))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //         image::load_from_memory(include_bytes!("../../../data/skybox/bottom.
        // jpg"))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //         image::load_from_memory(include_bytes!("../../../data/skybox/front.
        // jpg"))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //         image::load_from_memory(include_bytes!("../../../data/skybox/back.
        // jpg"))             .expect("Failed to load skybox texture!")
        //             .to_rgba8(),
        //     ],
        // );
        // let bind_group_layout =
        // device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("skybox_bind_group_layout"),
        //     entries: &[
        //         // Globals uniform buffer.
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: Globals::BUFFER_SIZE,
        //             },
        //             count: None,
        //         },
        //         // Environment map.
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::Cube,
        //                 sample_type: wgpu::TextureSampleType::Float { filterable:
        // true },             },
        //             count: None,
        //         },
        //         // Environment map sampler.
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 2,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty:
        // wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        // });
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("skybox_bind_group"),
        //     layout: &bind_group_layout,
        //     entries: &[
        //         // Globals uniform buffer.
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: globals.as_entire_binding(),
        //         },
        //         // Environment map.
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(&env_map.view),
        //         },
        //         // Environment map sampler.
        //         wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::Sampler(&env_map.sampler),
        //         },
        //     ],
        // });
        //
        // let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor
        // {     label: Some("skybox_shader_module"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
        // });
        //
        // let pipeline_layout =
        // device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("skybox_pipeline_layout"),
        //     bind_group_layouts: &[&bind_group_layout],
        //     push_constant_ranges: &[],
        // });
        //
        // let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
        // {     label: Some("skybox_pipeline"),
        //     layout: Some(&pipeline_layout),
        //     vertex: wgpu::VertexState {
        //         module: &shader_module,
        //         entry_point: "vs_main",
        //         buffers: &[],
        //     },
        //     primitive: Default::default(),
        //     depth_stencil: Some(wgpu::DepthStencilState {
        //         format: DEPTH_FORMAT,
        //         depth_write_enabled: false,
        //         depth_compare: wgpu::CompareFunction::LessEqual,
        //         stencil: Default::default(),
        //         bias: Default::default(),
        //     }),
        //     multisample: wgpu::MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     fragment: Some(wgpu::FragmentState {
        //         module: &shader_module,
        //         entry_point: "fs_main",
        //         targets: &[Some(wgpu::ColorTargetState {
        //             format: output_format,
        //             blend: None,
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })],
        //     }),
        //     multiview: None,
        // });
        //
        // Self {
        //     bind_group_layout,
        //     bind_group,
        //     globals,
        //     env_map,
        //     pipeline_layout,
        //     pipeline,
        // }
        todo!()
    }
}

// impl<'a> RenderingPass for SkyboxRenderPass<'a> {
//     fn record(
//         &mut self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &RenderTarget,
//         renderer: &Renderer,
//         scene: &Scene,
//         depth_texture: Option<&wgpu::TextureView>,
//     ) {
//         let mut render_pass =
// encoder.begin_render_pass(&wgpu::RenderPassDescriptor {             label:
// Some("skybox_render_pass"),             color_attachments:
// &[wgpu::RenderPassColorAttachmentDescriptor {                 attachment:
// &target.view,                 resolve_target: None,
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(renderer.clear_color),
//                     store: true,
//                 },
//             }],
//             depth_stencil_attachment: depth_texture.map(|texture| {
//                 wgpu::RenderPassDepthStencilAttachmentDescriptor {
//                     attachment: texture,
//                     depth_ops: Some(wgpu::Operations {
//                         load: wgpu::LoadOp::Load,
//                         store: false,
//                     }),
//                     stencil_ops: None,
//                 }
//             }),
//         });
//
//         render_pass.set_pipeline(&self.pipeline);
//         render_pass.set_bind_group(0, &self.bind_group, &[]);
//         render_pass.draw(0..3, 0..1);
//     }
// }
