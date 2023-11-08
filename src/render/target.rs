/// A render target is a texture that can be rendered to.
pub struct RenderTarget {
    /// The size of the render target.
    pub size: wgpu::Extent3d,
    /// The texture view of the render target.
    pub view: wgpu::TextureView,
    /// The texture format of the render target.
    pub format: wgpu::TextureFormat,
}

impl RenderTarget {
    pub fn aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}
