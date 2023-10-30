/// Surface of the window used to render.
pub struct Surface {
    /// Surface created from the window.
    pub inner: wgpu::Surface,
    /// Configuration of the surface (size, format, etc.).
    pub config: wgpu::SurfaceConfiguration,
}
