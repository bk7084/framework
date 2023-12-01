use crate::{color, core::Color};
use crossbeam_channel::Receiver;
use std::{collections::hash_map::Entry, path::Path, sync::Arc};
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
        mesh::{GpuMesh, Mesh, MeshBundle},
        FxHashMap, GpuMaterial, Material, MaterialBundle, SmlString, Texture, TextureBundle,
        TextureType,
    },
    render::rpass::{texture_bundle_bind_group_layout, BlinnPhongRenderPass, RenderingPass},
    scene::{NodeIdx, Scene},
};
pub use context::*;

// TODO: render bundles enables us to create N uniform buffers and dispatch N
// render calls, which is a bit slow if we iterate over all of them every frame,
// but render bundles can speed this up.

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
    default_texture: Handle<Texture>,
    default_material_bundle: Handle<MaterialBundle>,
    default_texture_bundle: Handle<TextureBundle>,
    mesh_bundles: FxHashMap<Handle<GpuMesh>, MeshBundle>,
    instancing: FxHashMap<Handle<GpuMesh>, Instancing>,
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
        let mut textures = TextureAssets::new();
        let bytes = include_bytes!("../../data/textures/checker.png");
        let default_texture =
            textures.load_from_bytes(&context.device, &context.queue, bytes, None, None);
        let mut samplers = FxHashMap::default();
        let default_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler_default"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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
        let mut texture_bundles = TextureBundleAssets::new();
        let default_texture_bundle = texture_bundles.add(TextureBundle {
            textures: vec![default_texture],
            samplers: vec!["default".into()],
            bind_group: None,
            sampler_index_buffer: None,
        });

        Self {
            device,
            queue,
            features,
            limits,
            pipelines: Pipelines::new(),
            meshes,
            material_bundles,
            textures,
            default_material_bundle,
            default_texture_bundle,
            mesh_bundles: FxHashMap::default(),
            instancing: FxHashMap::default(),
            samplers,
            cmd_receiver: receiver,
            texture_bundles,
            default_texture,
        }
    }

    /// Uploads a mesh to the GPU, creates `GpuMesh` from `Mesh` then adds it to
    /// the renderer.
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> (Handle<GpuMesh>, bool) {
        log::debug!("Uploading mesh#{}", mesh.id);
        self.meshes.add(&self.device, &self.queue, mesh)
    }

    /// Creates a bundle of materials and a bundle of textures from a list of
    /// materials.
    pub fn upload_materials(
        &mut self,
        materials: Option<&Vec<Material>>,
    ) -> (Handle<MaterialBundle>, Handle<TextureBundle>) {
        log::debug!("materials: {:?}", materials);
        match materials {
            None => {
                // Mesh has no material, use default material.
                log::debug!("Using default material and texture bundles");
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
                        let format = match tex_ty {
                            TextureType::MapNorm => Some(wgpu::TextureFormat::Rgba8Unorm),
                            _ => None,
                        };
                        let texture_hdl = self.add_texture(&tex_path, format);
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
                            _ => {}
                        }
                    }
                }
                log::debug!("loaded textures: {:?}", textures);
                log::debug!("GpuMaterials to be uploaded: {:?}", gpu_mtls);
                textures.push(self.default_texture);
                let bundle = MaterialBundle::new(&self.device, &gpu_mtls);
                let material_bundle = self.material_bundles.add(bundle);
                let samplers = textures
                    .iter()
                    .map(|tex| {
                        let texture = self.textures.get(*tex).unwrap();
                        texture.sampler.clone()
                    })
                    .collect();
                let texture_bundle = self.texture_bundles.add(TextureBundle {
                    textures,
                    samplers,
                    bind_group: None,
                    sampler_index_buffer: None,
                });
                (material_bundle, texture_bundle)
            }
        }
    }

    /// Gets a mesh bundle.
    pub fn get_mesh_bundle(&self, mesh: Handle<GpuMesh>) -> Option<MeshBundle> {
        self.mesh_bundles.get(&mesh).cloned()
    }

    pub fn insert_mesh_bundle(&mut self, mesh: Handle<GpuMesh>, bundle: MeshBundle) {
        self.mesh_bundles.insert(mesh, bundle);
    }

    /// Adds a new instancing data for a mesh.
    pub fn add_instancing(&mut self, mesh: Handle<GpuMesh>, nodes: &[NodeIdx]) {
        if nodes.is_empty() {
            return;
        }
        match self.instancing.entry(mesh) {
            Entry::Occupied(mut instancing) => {
                instancing.get_mut().nodes.extend(nodes.iter());
            }
            Entry::Vacant(_) => {
                self.instancing.insert(
                    mesh,
                    Instancing {
                        nodes: nodes.to_vec(),
                    },
                );
            }
        }
    }

    /// Removes one instancing data for a mesh.
    pub fn remove_instancing(&mut self, mesh: Handle<GpuMesh>, node: NodeIdx) {
        if let Some(instancing) = self.instancing.get_mut(&mesh) {
            instancing.nodes.retain(|n| *n != node);
        }
    }

    pub fn add_texture(
        &mut self,
        filepath: &Path,
        format: Option<wgpu::TextureFormat>,
    ) -> Handle<Texture> {
        self.textures
            .load_from_file(&self.device, &self.queue, filepath, format)
    }

    /// Prepares the renderer for rendering.
    pub fn prepare(&mut self) {
        profiling::scope!("Renderer::prepare");
        let mut sampler_indices = [0u32; BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN];
        let default_texture = self.textures.get(self.default_texture).unwrap();
        let default_sampler = self.samplers.get("default").unwrap();
        let default_texture_view = &default_texture.view;
        // Create bind groups for each texture bundle.
        for bundle in self.texture_bundles.iter_mut() {
            sampler_indices.fill(0);
            if bundle.textures.is_empty() || bundle.bind_group.is_some() {
                continue;
            }

            // Populate texture views and samplers with default values.
            let mut views = [default_texture_view; BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN];
            let mut samplers = [default_sampler; BlinnPhongRenderPass::MAX_SAMPLER_ARRAY_LEN];

            for (i, hdl) in bundle.textures.iter().enumerate() {
                let texture = self.textures.get(*hdl).unwrap();
                views[i] = &texture.view;
            }

            let mut unique_samplers = vec![];
            for (idx, sampler) in bundle.samplers.iter().enumerate() {
                if !unique_samplers.contains(sampler) {
                    sampler_indices[idx] = unique_samplers.len() as u32;
                    unique_samplers.push(sampler.clone());
                }
            }

            for (i, sampler) in unique_samplers.iter().enumerate() {
                samplers[i] = self.samplers.get(sampler).unwrap();
            }

            bundle.sampler_index_buffer = Some(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("sampler_index_buffer"),
                    contents: bytemuck::cast_slice(&sampler_indices),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));

            let bind_group_layout = texture_bundle_bind_group_layout(&self.device);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shading_textures_bind_group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&views),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: bundle
                            .sampler_index_buffer
                            .as_ref()
                            .unwrap()
                            .as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::SamplerArray(&samplers),
                    },
                ],
            });
            bundle.bind_group.replace(bind_group);
        }
    }

    /// Renders a frame.
    pub fn render(
        &mut self,
        scene: &Scene,
        target: &RenderTarget,
        rpass: &mut dyn RenderingPass,
        mode: ShadingMode,
    ) -> Result<(), wgpu::SurfaceError> {
        profiling::scope!("Renderer::render");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            });

        rpass.record(
            self,
            target,
            scene,
            &self.device,
            &self.queue,
            &mut encoder,
            mode,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

/// Instancing information for a mesh.
#[derive(Clone, Debug, Default)]
pub struct Instancing {
    /// Nodes that use this mesh.
    pub nodes: Vec<NodeIdx>,
}
