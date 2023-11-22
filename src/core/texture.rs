use crate::core::{
    assets::{Asset, Handle},
    SmlString,
};
use std::ops::Deref;

/// Texture sampler.
#[derive(Debug)]
pub struct TextureSampler {
    /// Reference to the allocated sampler data.
    pub raw: wgpu::Sampler,
    pub desc: wgpu::SamplerDescriptor<'static>,
}

/// GPU texture and view.
///
/// It could be used as a render target or as a texture to be used in a shader.
#[derive(Debug)]
pub struct Texture {
    /// Image (data) allocated on GPU. It holds the pixels and main
    /// memory of the texture, but doesn't contain a lot information
    /// on how to interpret the data.
    pub raw: wgpu::Texture,
    /// Reference to the allocated image data. Besides, it holds information
    /// about how to interpret the data of the texture.
    pub view: wgpu::TextureView,
    /// Size of the texture.
    pub size: wgpu::Extent3d,
    /// Name of the sampler to be used by the texture.
    pub sampler: SmlString,
}

impl Asset for Texture {}

impl Deref for Texture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

/// A collection of textures and samplers.
pub struct TextureBundle {
    pub textures: Vec<Handle<Texture>>,
    pub samplers: Vec<SmlString>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub sampler_index_buffer: Option<wgpu::Buffer>,
}

impl Asset for TextureBundle {}

impl Default for TextureBundle {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
            samplers: Vec::new(),
            bind_group: None,
            sampler_index_buffer: None,
        }
    }
}
