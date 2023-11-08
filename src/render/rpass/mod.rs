mod clear;
mod wireframe;

use crate::{render::RenderTarget, scene::Scene};
pub use clear::*;
use glam::{Mat4, Vec4};
pub use wireframe::*;

pub trait RenderingPass {
    /// Records the pass.
    fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        scene: &Scene,
    );
}
