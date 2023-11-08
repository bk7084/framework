use crate::{
    core::Color,
    render::{rpass::RenderingPass, RenderTarget},
    scene::Scene,
};
use wgpu::{CommandEncoder, TextureView};

pub struct ClearPass {
    pub clear_color: Color,
}

impl ClearPass {
    pub fn new(clear_color: Color) -> Self {
        Self { clear_color }
    }
}

impl RenderingPass for ClearPass {
    fn record(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
        target: &RenderTarget,
        _scene: &Scene,
    ) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(*self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
