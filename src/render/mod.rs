use crate::{color, core::Color};
use crossbeam_channel::Receiver;
use std::{path::Path, sync::Arc};
use wgpu::util::DeviceExt;

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
        assets::{GpuMeshAssets, Handle, MaterialAssets, MaterialBundleAssets, TextureAssets},
        mesh::{GpuMesh, Mesh},
        FxHashMap, GpuMaterial, Material, MaterialBundle, SmlString, Texture,
    },
    render::rpass::{MaterialUniform, RenderingPass},
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
    material_bundles: MaterialBundleAssets,
    textures: TextureAssets,
    default_material: Handle<GpuMaterial>,
    default_material_bundle: Handle<MaterialBundle>,
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
        let mut materials = MaterialAssets::new();
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
        let default_material = materials.add(GpuMaterial {
            name: SmlString::from("default"),
            ka: Some([0.0, 0.0, 0.0]),
            kd: Some([0.8, 0.1, 0.8]),
            ks: Some([0.0, 0.0, 0.0]),
            ns: Some(0.0),
            ni: None,
            opacity: None,
            map_ka: None,
            map_kd: None,
            map_ks: None,
            map_ns: None,
            map_d: None,
            map_bump: None,
            map_disp: None,
            map_decal: None,
            map_norm: None,
            illumination_model: None,
        });
        let mut material_bundles = MaterialBundleAssets::new();
        let default_material_bundle = MaterialBundle::default(&context.device);
        let default_material_bundle_handle = material_bundles.add(default_material_bundle);
        Self {
            device,
            queue,
            features,
            limits,
            pipelines: Pipelines::new(),
            meshes,
            materials,
            material_bundles,
            textures,
            default_material,
            default_material_bundle: default_material_bundle_handle,
            samplers: Default::default(),
            cmd_receiver: receiver,
        }
    }

    /// Uploads a mesh to the GPU, creates `GpuMesh` from `Mesh` then adds it to
    /// the renderer.
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> Handle<GpuMesh> {
        self.meshes.add(&self.device, &self.queue, mesh)
    }

    /// Creates a bundle of materials and a bundle of textures from a list of
    /// materials.
    pub fn upload_materials(
        &mut self,
        materials: Option<&Vec<Material>>,
    ) -> Handle<MaterialBundle> {
        // Temporarily omitting the texture loading.
        let material_bundle = match materials {
            None => {
                // Mesh has no material, use default material.
                self.default_material_bundle
            }
            Some(materials) => {
                // Material bundle size is the number of materials + 1 (for the
                // default material).
                let materials_data = materials
                    .iter()
                    .map(|mtl| MaterialUniform::from_material_new(mtl))
                    .chain(std::iter::once(MaterialUniform::default()))
                    .collect::<Vec<_>>();
                let bundle = MaterialBundle::new(&self.device, &materials_data);
                self.material_bundles.add(bundle)
            }
        };

        material_bundle

        // let gpu_material = GpuMaterial {
        //     name: material.name.clone(),
        //     ka: material.ka,
        //     kd: material.kd,
        //     ks: material.ks,
        //     ns: material.ns,
        //     ni: material.ni,
        //     opacity: material.opacity,
        //     map_ka: material.map_ka.as_ref().map(|path|
        // self.add_texture(&path)),     map_kd:
        // material.map_kd.as_ref().map(|path| self.add_texture(&path)),
        //     map_ks: material.map_ks.as_ref().map(|path|
        // self.add_texture(&path)),     map_ns:
        // material.map_ns.as_ref().map(|path| self.add_texture(&path)),
        //     map_d: material.map_d.as_ref().map(|path|
        // self.add_texture(&path)),     map_bump: material
        //         .map_bump
        //         .as_ref()
        //         .map(|path| self.add_texture(&path)),
        //     map_disp: material
        //         .map_disp
        //         .as_ref()
        //         .map(|path| self.add_texture(&path)),
        //     map_decal: material
        //         .map_decal
        //         .as_ref()
        //         .map(|path| self.add_texture(&path)),
        //     map_norm: material
        //         .map_norm
        //         .as_ref()
        //         .map(|path| self.add_texture(&path)),
        //     illumination_model: material.illumination_model,
        // };
        // self.materials.add(gpu_material)
    }

    pub fn add_texture(&mut self, filepath: &Path) -> Handle<Texture> {
        self.textures
            .load_from_file(&self.device, &self.queue, filepath)
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
