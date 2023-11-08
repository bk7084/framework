use crate::{color, core::Color};
use legion::{Resources, Schedule};
use std::sync::Arc;
use wgpu::StoreOp;

mod context;
pub mod rpass;
pub mod surface;
mod target;
pub use target::*;

use crate::{render::rpass::RenderingPass, scene::Scene};
pub use context::*;

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    features: wgpu::Features,
    limits: wgpu::Limits,
    // scheduler: Schedule,
}

impl Renderer {
    /// Clear color of the renderer.
    pub const CLEAR_COLOR: Color = color!(0.60383, 0.66539, 0.42327);

    /// Creates a new renderer.
    pub fn new(context: &GpuContext, aspect_ratio: f32) -> Self {
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
            // scheduler: Schedule::builder()
            //     .add_system(super::systems::update_transforms_system())
            //     .add_system(super::systems::update_meshes_system())
            //     .add_system(super::systems::update_materials_system())
            //     .add_system(super::systems::update_cameras_system(aspect_ratio))
            //     .build(),
        }
    }

    /// Renders a frame.
    pub fn render(
        &mut self,
        scene: &Scene,
        rpass: &mut dyn RenderingPass,
        target: &RenderTarget,
    ) -> Result<(), wgpu::SurfaceError> {
        profiling::scope!("Renderer::render");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            });

        rpass.record(&self.device, &self.queue, &mut encoder, target, scene);

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
