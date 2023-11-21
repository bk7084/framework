use crate::render::context::GpuContext;
use std::ops::{Deref, DerefMut};
use winit::window::Window;

/// Surface of the window used to render.
///
/// Wraps a `wgpu::Surface` and its configuration.
pub struct Surface<'w> {
    /// Surface created from the window.
    pub inner: wgpu::Surface,
    /// Configuration of the surface (size, format, etc.).
    pub config: wgpu::SurfaceConfiguration,
    /// Phantom data to tie the lifetime of the surface to the window.
    _marker: std::marker::PhantomData<&'w ()>,
}

impl<'w> Deref for Surface<'w> {
    type Target = wgpu::Surface;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'w> DerefMut for Surface<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Accessors.
impl<'w> Surface<'w> {
    /// Returns the texture format of the surface.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Returns the width of the surface.
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Returns the height of the surface.
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Returns the present mode of the surface.
    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    /// Returns the aspect ratio of the surface.
    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

impl<'w> Surface<'w> {
    /// Creates a new surface from a window and configures it.
    pub fn new(context: &GpuContext, window: &Window) -> Self {
        profiling::scope!("Surface::new");
        let surface = unsafe { context.instance.create_surface(window).unwrap() };
        let caps = surface.get_capabilities(&context.adapter);
        let mut format = caps.formats[0];
        for fmt in caps.formats {
            if fmt == wgpu::TextureFormat::Bgra8UnormSrgb
                || fmt == wgpu::TextureFormat::Rgba8UnormSrgb
            {
                format = fmt;
                break;
            }
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&context.device, &config);

        Self {
            inner: surface,
            config,
            _marker: Default::default(),
        }
    }

    /// Resizes the surface and reconfigures it.
    ///
    /// Size is expressed in physical pixels.
    ///
    /// Returns `true` if the surface was resized.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) -> bool {
        if self.config.width == width && self.config.height == height {
            return false;
        }
        self.config.width = width;
        self.config.height = height;
        self.inner.configure(device, &self.config);
        true
    }
}
