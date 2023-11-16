mod blinn_phong;
mod clear;
mod wireframe;

use crate::{
    render::{RenderTarget, Renderer},
    scene::Scene,
};
pub use blinn_phong::*;
pub use clear::*;
pub use wireframe::*;

pub trait RenderingPass {
    /// Records the pass.
    fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        renderer: &Renderer,
        scene: &Scene,
    );
}
