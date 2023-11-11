use glam::UVec2;

#[derive(Debug, Clone)]
pub struct Texture {
    pub format: wgpu::TextureFormat,
    pub size: UVec2,
}
