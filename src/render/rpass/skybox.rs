use crate::{
    render::{rpass::RenderingPass, RenderTarget, Renderer},
    scene::Scene,
};

/// A skybox environment map.
///
/// This is a cube map texture that is used to render the skybox. It is either
/// created from a set of 6 images or from a single equirectangular image
/// (compute shader involved).
pub struct EnvironmentMap {
    /// The cube map texture of 6 layers stored in the order of +X, -X, +Y, -Y,
    /// +Z, -Z.
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

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
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// Uniform buffer containing the `Globals` uniform struct.
    pub globals_uniform_buffer: &'a wgpu::Buffer,
    pub env_map: EnvironmentMap,
}

impl<'a> SkyboxRenderPass<'a> {
    /// Creates a new skybox render pass.
    ///
    /// # Arguments
    ///
    /// * `globals` - The global uniform buffer, which contains the view and
    ///   projection matrices.
    pub fn new(globals: &'a wgpu::Buffer) -> Self {
        todo!()
    }
}

impl<'a> RenderingPass for SkyboxRenderPass<'a> {
    fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        renderer: &Renderer,
        scene: &Scene,
    ) {
        todo!()
    }
}
