use crate::{color, renderer::context::GPUContext};
use std::sync::Arc;
use wgpu::StoreOp;

pub mod context;
pub mod surface;

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    features: wgpu::Features,
    limits: wgpu::Limits,
}

impl Renderer {
    /// Clear color of the renderer.
    pub const CLEAR_COLOR: wgpu::Color = color!(0.60383, 0.66539, 0.42327);

    /// Creates a new renderer.
    pub fn new(context: &GPUContext, aspect_ratio: f32) -> Self {
        profiling::scope!("Renderer::new");
        let device = context.device.clone();
        let queue = context.queue.clone();
        let features = context.features;
        let limits = context.limits.clone();

        Self {
            device,
            queue,
            features,
            limits,
        }
    }

    /// Renders a frame.
    pub fn render(&mut self, frame: &wgpu::SurfaceTexture) -> Result<(), wgpu::SurfaceError> {
        profiling::scope!("Renderer::render");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Self::CLEAR_COLOR),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
