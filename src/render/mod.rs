use crate::{
    color,
    core::{Color, FxHasher},
};
use crossbeam_channel::Receiver;
use std::{collections::hash_map::Entry, hash::Hasher, path::Path, sync::Arc};
use wgpu::util::DeviceExt;

mod context;
mod pipeline;
pub use pipeline::*;
pub mod rpass;
mod sampler;
pub mod surface;
mod target;
pub mod util;

pub use sampler::*;

pub use target::*;

use crate::{
    app::command::{Command, CommandReceiver},
    core::{
        assets::{GpuMeshAssets, Handle, MaterialBundleAssets, TextureAssets, TextureBundleAssets},
        mesh::{AestheticBundle, Mesh, MeshBundle},
        FxHashMap, GpuMaterial, Material, MaterialBundle, SmlString, Texture, TextureBundle,
        TextureType,
    },
    render::rpass::{
        texture_bundle_bind_group_layout, BlinnPhongRenderPass, LightsBindGroup, RenderingPass,
    },
    scene::{NodeIdx, Scene},
};
pub use context::*;
// TODO: render bundles enables us to create N uniform buffers and dispatch N
// render calls, which is a bit slow if we iterate over all of them every frame,
// but render bundles can speed this up.

// Currently, we only support instancing for meshes (not materials).

/// Shading mode.
#[pyo3::pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadingMode {
    /// Flat shading.
    Flat,
    /// Gouraud shading.
    Gouraud,
    /// Blinn-Phong shading.
    BlinnPhong,
}

pub struct RenderParams {
    /// Shading mode.
    pub mode: ShadingMode,
    /// Whether to enable back face culling.
    pub enable_back_face_culling: bool,
    /// Whether to enable occlusion culling. TODO: implement occlusion culling.
    pub enable_occlusion_culling: bool,
    /// Whether to draw wireframe.
    pub enable_wireframe: bool,
    /// Whether to enable shadow.
    pub enable_shadows: bool,
    /// Whether to enable lighing.
    pub enable_lighting: bool,
    /// Whether to write shadow maps once.
    #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
    pub write_shadow_maps: bool,
}

impl RenderParams {
    pub fn new() -> Self {
        Self {
            mode: ShadingMode::BlinnPhong,
            enable_back_face_culling: true,
            enable_occlusion_culling: false,
            enable_wireframe: false,
            enable_shadows: false,
            enable_lighting: true,
            #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
            write_shadow_maps: false,
        }
    }

    /// Whether to cast shadows.
    #[inline]
    pub const fn casting_shadows(&self) -> bool {
        self.enable_shadows && !self.enable_wireframe && self.enable_lighting
    }
}

pub struct Renderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    limits: wgpu::Limits,
    pub(crate) meshes: GpuMeshAssets,
    textures: TextureAssets,

    material_bundles: MaterialBundleAssets,
    texture_bundles: TextureBundleAssets,
    // samplers: FxHashMap<SmlString, wgpu::Sampler>,

    // TODO: remove these default bundles, make them inside the assets.
    default_material_bundle: Handle<MaterialBundle>,
    default_texture_bundle: Handle<TextureBundle>,
    aesthetic_bundles: Vec<AestheticBundle>,
    /// Nodes that use instancing for each mesh bundle.
    pub(crate) instancing: FxHashMap<MeshBundle, Vec<NodeIdx>>,
    samplers: FxHashMap<SmlString, Sampler>,
    params: RenderParams,
    cmd_receiver: Receiver<Command>,

    // Variable controlling the scale of the orthographic projection matrix
    // of the shadow map.
    //
    // TODO: shadow map projection should be automatically calculated according
    // to the camera's frustum, light's parameters and the scene's bounding box.
    light_proj_scale: f32,
}

impl Renderer {
    /// Clear color of the renderer.
    pub const CLEAR_COLOR: Color = color!(0.60383, 0.66539, 0.42327);

    /// Creates a new renderer.
    pub fn new(context: &GpuContext, receiver: CommandReceiver) -> Self {
        profiling::scope!("Renderer::new");
        let device = context.device.clone();
        let queue = context.queue.clone();
        let limits = context.limits.clone();
        let meshes = GpuMeshAssets::new(&device);
        let textures = TextureAssets::new(&context.device, &context.queue);
        let samplers = Self::create_samplers(&context.device);
        let mut material_bundles = MaterialBundleAssets::new();
        let default_material_bundle =
            material_bundles.add(MaterialBundle::default(&context.device));
        let mut texture_bundles = TextureBundleAssets::new();
        let default_texture_bundle = texture_bundles.add(TextureBundle {
            textures: vec![textures.default_texture()],
            samplers: vec!["linear".into()],
            bind_group: None,
            sampler_index_buffer: None,
        });

        Self {
            device,
            queue,
            limits,
            meshes,
            material_bundles,
            textures,
            default_material_bundle,
            default_texture_bundle,
            aesthetic_bundles: vec![],
            instancing: FxHashMap::default(),
            samplers,
            params: RenderParams {
                mode: ShadingMode::BlinnPhong,
                enable_back_face_culling: true,
                enable_occlusion_culling: false,
                enable_wireframe: false,
                enable_shadows: false,
                enable_lighting: true,
                #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
                write_shadow_maps: true,
            },
            cmd_receiver: receiver,
            texture_bundles,
            light_proj_scale: 1.0,
        }
    }

    fn create_samplers(device: &wgpu::Device) -> FxHashMap<SmlString, Sampler> {
        let mut samplers = FxHashMap::default();
        samplers.insert(
            SmlString::from("linear"),
            Sampler::new(
                &device,
                wgpu::SamplerDescriptor {
                    label: Some("linear_repeat_sampler"),
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                },
            ),
        );
        samplers.insert(
            SmlString::from("nearest"),
            Sampler::new(
                &device,
                wgpu::SamplerDescriptor {
                    label: Some("nearest_repeat_sampler"),
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                },
            ),
        );
        samplers.insert(
            SmlString::from("depth"),
            Sampler::new(
                &device,
                wgpu::SamplerDescriptor {
                    label: Some("depth_sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    compare: Some(wgpu::CompareFunction::LessEqual),
                    ..Default::default()
                },
            ),
        );
        samplers
    }

    /// Uploads a mesh to the GPU, creates `GpuMesh` from `Mesh` then adds it to
    /// the renderer.
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> MeshBundle {
        log::debug!("Uploading mesh#{}", mesh.name);
        log::debug!("Mesh materials: {:?}", mesh.materials);

        let mesh_hdl = self.meshes.add(&self.device, &self.queue, mesh);
        // Upload materials and create a material bundle.
        match &mesh.materials {
            None => {
                log::info!("Mesh#{} has no materials, use default.", mesh.name);
                MeshBundle {
                    mesh: mesh_hdl,
                    aesthetic: AestheticBundle {
                        materials: self.default_material_bundle,
                        textures: self.default_texture_bundle,
                    },
                }
            }
            Some(materials) => {
                let aesthetic = self.upload_materials(materials);
                log::info!("Mesh#{} uses aesthetic: {:?}", mesh.name, aesthetic);
                MeshBundle {
                    mesh: mesh_hdl,
                    aesthetic,
                }
            }
        }
    }

    /// Creates a bundle of materials and a bundle of textures from a list of
    /// materials.
    fn upload_materials(&mut self, materials: &[Material]) -> AestheticBundle {
        let materials_name_hashes = materials
            .iter()
            .map(|mtl| {
                let mut hasher = FxHasher::default();
                hasher.write(mtl.name.as_bytes());
                hasher.finish()
            })
            .collect::<Vec<_>>();
        log::debug!(
            "Try to find material name hashes: {:?}",
            materials_name_hashes
        );
        let mut aesthetic_bundle = None;
        for bundle in &self.aesthetic_bundles {
            let material_bundle = self.material_bundles.get(bundle.materials).unwrap();
            if material_bundle.materials.len() - 1 != materials.len() {
                continue;
            }
            if materials_name_hashes
                .iter()
                .all(|mtl| material_bundle.materials.contains(mtl))
            {
                log::debug!("Found existing material bundle: {:?}", bundle);
                aesthetic_bundle = Some(bundle);
                break;
            }
        }

        match aesthetic_bundle {
            Some(bundle) => *bundle,
            None => {
                log::debug!("No existing material bundle found, create a new one.");
                let default_material = Material::default();
                // Material bundle size is the number of materials + 1 (for the
                // default material, last one).
                let mtls = materials.iter().chain(std::iter::once(&default_material));
                let mut gpu_mtls = mtls
                    .clone()
                    .map(GpuMaterial::from_material)
                    .collect::<Vec<_>>();

                // Load textures and create a bundle of textures.
                let mut textures = Vec::new();
                for (mtl, gpu_mtl) in mtls.clone().zip(gpu_mtls.iter_mut()) {
                    for (tex_ty, tex_path) in mtl.textures.iter() {
                        let format = match tex_ty {
                            TextureType::MapNorm => Some(wgpu::TextureFormat::Rgba8Unorm),
                            _ => None,
                        };
                        let texture_hdl = self.add_texture(tex_path, format);
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
                textures.push(self.textures.default_texture());
                let bundle = MaterialBundle::new(&self.device, mtls, &gpu_mtls);
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
                let aesthetic = AestheticBundle {
                    materials: material_bundle,
                    textures: texture_bundle,
                };
                self.aesthetic_bundles.push(aesthetic);
                aesthetic
            }
        }
    }

    /// Adds a new instancing data for a mesh.
    pub fn add_instancing(&mut self, mesh: MeshBundle, nodes: &[NodeIdx]) {
        if nodes.is_empty() {
            return;
        }
        match self.instancing.entry(mesh) {
            Entry::Occupied(mut instancing) => {
                instancing.get_mut().extend(nodes.iter());
            }
            Entry::Vacant(_) => {
                self.instancing.insert(mesh, nodes.to_vec());
            }
        }
        log::debug!("Instancing: {:?}", self.instancing);
    }

    /// Removes one instancing data for a mesh.
    pub fn remove_instancing(&mut self, mesh: MeshBundle, node: NodeIdx) {
        if let Some(nodes) = self.instancing.get_mut(&mesh) {
            nodes.retain(|n| *n != node);
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
        while let Ok(cmd) = self.cmd_receiver.try_recv() {
            match cmd {
                Command::EnableBackfaceCulling(enable) => {
                    self.params.enable_back_face_culling = enable;
                }
                Command::EnableWireframe(enable) => {
                    self.params.enable_wireframe = enable;
                }
                Command::EnableShadows(enable) => {
                    self.params.enable_shadows = enable;
                }
                Command::EnableLighting(enable) => {
                    self.params.enable_lighting = enable;
                }
                Command::UpdateShadowMapOrthoProj(size) => {
                    let scale = size * 0.9 / LightsBindGroup::ORTHO_H;
                    log::debug!("Update shadow map ortho proj scale: {}", scale.max(1.0));
                    self.light_proj_scale = scale.max(1.0);
                }
                _ => {}
            }
        }

        let mut sampler_indices = [0u32; BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN];
        let default_texture = self.textures.get(self.textures.default_texture()).unwrap();
        let default_sampler = self.samplers.get("linear").unwrap();
        let default_texture_view = &default_texture.view;
        // Create bind groups for each texture bundle.
        for bundle in self.texture_bundles.iter_mut() {
            sampler_indices.fill(0);
            if bundle.textures.is_empty() || bundle.bind_group.is_some() {
                continue;
            }

            // Populate texture views and samplers with default values.
            let mut views = [default_texture_view; BlinnPhongRenderPass::MAX_TEXTURE_ARRAY_LEN];
            let mut samplers =
                [&default_sampler.sampler; BlinnPhongRenderPass::MAX_SAMPLER_ARRAY_LEN];

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
    ) -> Result<(), wgpu::SurfaceError> {
        profiling::scope!("Renderer::render");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            });

        rpass.record(self, target, &self.params, scene, &mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
