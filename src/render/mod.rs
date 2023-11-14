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
        assets::{GpuMeshAssets, Handle, MaterialBundleAssets, TextureAssets, TextureBundleAssets},
        mesh::{GpuMesh, Mesh},
        FxHashMap, GpuMaterial, Material, MaterialBundle, SmlString, Texture, TextureBundle,
        TextureType,
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
    textures: TextureAssets,
    material_bundles: MaterialBundleAssets,
    texture_bundles: TextureBundleAssets,
    default_material_bundle: Handle<MaterialBundle>,
    default_texture_bundle: Handle<TextureBundle>,
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
        let mut material_bundles = MaterialBundleAssets::new();
        let default_material_bundle =
            material_bundles.add(MaterialBundle::default(&context.device));
        let default_texture_bundle = TextureBundleAssets::new().add(TextureBundle::default());
        Self {
            device,
            queue,
            features,
            limits,
            pipelines: Pipelines::new(),
            meshes,
            // materials,
            material_bundles,
            textures,
            default_material_bundle,
            default_texture_bundle,
            samplers: Default::default(),
            cmd_receiver: receiver,
            texture_bundles: TextureBundleAssets::new(),
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
    ) -> (Handle<MaterialBundle>, Handle<TextureBundle>) {
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
                    .chain(std::iter::once(&Material::default()))
                    .map(|mtl| GpuMaterial::from_material(mtl))
                    .collect::<Vec<_>>();
                let bundle = MaterialBundle::new(&self.device, &materials_data);
                self.material_bundles.add(bundle)
            }
        };

        match materials {
            None => {
                // Mesh has no material, use default material.
                (self.default_material_bundle, self.default_texture_bundle)
            }
            Some(mtls) => {
                // Material bundle size is the number of materials + 1 (for the
                // default material).
                let mut gpu_mtls = mtls
                    .iter()
                    .chain(std::iter::once(&Material::default()))
                    .map(|mtl| GpuMaterial::from_material(mtl))
                    .collect::<Vec<_>>();

                // Load textures and create a bundle of textures.
                let mut textures = Vec::new();
                for (mtl, gpu_mtl) in mtls.iter().zip(gpu_mtls.iter_mut()) {
                    for (tex_ty, tex_path) in mtl.textures.iter() {
                        let texture_hdl = self.add_texture(&tex_path);
                        let texture_idx = textures.len();
                        textures.push(texture_hdl);
                        match tex_ty {
                            TextureType::MapKa => {
                                gpu_mtl.map_ka = texture_idx as u32;
                            }
                            TextureType::MapKd => {
                                gpu_mtl.map_kd = texture_idx as u32;
                            }
                            TextureType::MapKs => {
                                gpu_mtl.map_ks = texture_idx as u32;
                            }
                            TextureType::MapNs => {
                                gpu_mtl.map_ns = texture_idx as u32;
                            }
                            TextureType::MapD => {
                                gpu_mtl.map_d = texture_idx as u32;
                            }
                            TextureType::MapBump => {
                                gpu_mtl.map_bump = texture_idx as u32;
                            }
                            TextureType::MapDisp => {
                                gpu_mtl.map_disp = texture_idx as u32;
                            }
                            TextureType::MapDecal => {
                                gpu_mtl.map_decal = texture_idx as u32;
                            }
                            TextureType::MapNorm => {
                                gpu_mtl.map_norm = texture_idx as u32;
                            }
                        }
                    }
                }
                let bundle = MaterialBundle::new(&self.device, &gpu_mtls);
                let material_bundle = self.material_bundles.add(bundle);
                let samplers = textures
                    .iter()
                    .map(|tex| {
                        let texture = self.textures.get(*tex).unwrap();
                        texture.sampler.clone()
                    })
                    .collect();
                let texture_bundle = self
                    .texture_bundles
                    .add(TextureBundle { textures, samplers });
                (material_bundle, texture_bundle)
            }
        }
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
