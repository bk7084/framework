use crate::{color, core::Color};
use crossbeam_channel::Receiver;
use std::sync::Arc;

mod context;
mod pipeline;
pub use pipeline::*;
pub mod rpass;
pub mod surface;
mod target;

pub use target::*;

use crate::{
    app::command::{Command, CommandReceiver},
    core::{
        assets::{GpuMeshAssets, Handle, MaterialAssets, TextureAssets},
        mesh::{GpuMesh, Mesh},
        FxHashMap, SmlString,
    },
    render::rpass::RenderingPass,
    scene::Scene,
};
pub use context::*;

/// Shading mode.
#[pyo3::pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadingMode {
    /// Wireframe, no lighting.
    Wireframe,
    /// Flat shading.
    Flat,
    /// Gouraud shading.
    Gouraud,
    /// Blinn-Phong shading.
    BlinnPhong,
}

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    features: wgpu::Features,
    limits: wgpu::Limits,
    pipelines: Pipelines,
    meshes: GpuMeshAssets,
    materials: MaterialAssets,
    textures: TextureAssets,
    samplers: FxHashMap<SmlString, wgpu::Sampler>,
    cmd_receiver: Receiver<Command>,
}

impl Renderer {
    /// Clear color of the renderer.
    pub const CLEAR_COLOR: Color = color!(0.60383, 0.66539, 0.42327);

    /// Creates a new renderer.
    pub fn new(context: &GpuContext, receiver: CommandReceiver) -> Self {
        profiling::scope!("Renderer::new");
        let device = context.device.clone();
        let queue = context.queue.clone();
        let features = context.features;
        let limits = context.limits.clone();
        let meshes = GpuMeshAssets::new(&device);
        let materials = MaterialAssets::new();
        let textures = TextureAssets::new();
        let mut samplers = FxHashMap::default();
        let default_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler_default"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        let depth_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler_depth"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        samplers.insert(SmlString::from("default"), default_sampler);
        samplers.insert(SmlString::from("default_depth"), depth_sampler);
        Self {
            device,
            queue,
            features,
            limits,
            pipelines: Pipelines::new(),
            meshes,
            materials,
            textures,
            samplers: Default::default(),
            cmd_receiver: receiver,
        }
    }

    /// Adds a mesh to the renderer (creates `GpuMesh`).
    pub fn add_mesh(&mut self, mesh: &Mesh) -> Handle<GpuMesh> {
        self.meshes.add(&self.device, &self.queue, mesh)
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

        rpass.record(&self.device, &self.queue, &mut encoder, target, self, scene);

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
