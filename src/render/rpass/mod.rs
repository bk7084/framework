mod blinn_phong;
mod clear;
mod skybox;
mod wireframe;

use crate::{
    render::{RenderTarget, Renderer},
    scene::Scene,
};
pub use blinn_phong::*;
use bytemuck::{Pod, Zeroable};
pub use clear::*;
pub use wireframe::*;

/// The global uniforms for the rendering passes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    /// The view matrix.
    pub view: [f32; 16],
    /// The projection matrix.
    pub proj: [f32; 16],
}

impl Globals {
    /// The size of the global uniforms.
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
}

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
