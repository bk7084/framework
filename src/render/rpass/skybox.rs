use crate::{
    render::{rpass::RenderingPass, RenderTarget, Renderer},
    scene::Scene,
};
use wgpu::{CommandEncoder, Device, Queue};

pub struct SkyboxRenderPass<'a> {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: &'a wgpu::Buffer,
}

impl<'a> SkyboxRenderPass<'a> {
    pub fn new(globals: &'a wgpu::Buffer) -> Self {}
}

impl RenderingPass for SkyboxRenderPass {
    fn record(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        target: &RenderTarget,
        renderer: &Renderer,
        scene: &Scene,
    ) {
    }
}
