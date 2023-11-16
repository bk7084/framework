use crate::core::assets::Asset;
use bytemuck::{Pod, Zeroable};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use wgpu::util::DeviceExt;

use crate::core::{FxHashMap, SmlString};

/// Texture type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    MapKa,    // ambient
    MapKd,    // diffuse
    MapKs,    // specular,
    MapNs,    // shininess,
    MapD,     // opacity,
    MapBump,  // bump,
    MapDisp,  // displacement,
    MapDecal, // stencil decal,
    MapNorm,  // normal,
}

/// Material description derived from a `MTL` file.
#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name.
    pub name: SmlString,
    /// Ambient color. `Ka` in the `MTL` spec.
    pub ka: Option<[f32; 3]>,
    /// Diffuse color. `Kd` in the `MTL` spec.
    pub kd: Option<[f32; 3]>,
    /// Specular color. `Ks` in the `MTL` spec.
    pub ks: Option<[f32; 3]>,
    /// Shininess or glossiness. `Ns` in the `MTL` spec.
    pub ns: Option<f32>,
    /// Optical density also known as index of refraction. Called
    /// `optical_density` in the `MTL` specc. Takes on a value between 0.001
    /// and 10.0. 1.0 means light does not bend as it passes through
    /// the object.
    pub ni: Option<f32>,
    /// Dissolve attribute is the alpha term for the material. Referred to as
    /// dissolve since that's what the `MTL` file format docs refer to it as.
    /// Takes on a value between 0.0 and 1.0. 0.0 is completely transparent,
    /// 1.0 is completely opaque. `d` in the `MTL` spec. It is called `Tr` in
    /// the `OBJ` spec which is 1.0 - `d`.
    pub opacity: Option<f32>,
    /// The illumnination model to use for this material. The different
    /// illumination models are specified in the [`MTL` spec](http://paulbourke.net/dataformats/mtl/).
    ///
    /// - 0: Color on and Ambient off
    /// - 1: Color on and Ambient on
    /// - 2: Highlight on
    /// - 3: Reflection on and Ray trace on
    /// - 4: Transparency: Glass on, Reflection: Ray trace on
    /// - 5: Reflection: Fresnel on and Ray trace on
    /// - 6: Transparency: Refraction on, Reflection: Fresnel off and Ray trace
    ///   on
    /// - 7: Transparency: Refraction on, Reflection: Fresnel on and Ray trace
    ///   on
    /// - 8: Reflection on and Ray trace off
    /// - 9: Transparency: Glass on, Reflection: Ray trace off
    /// - 10: Casts shadows onto invisible surfaces
    pub illumination_model: Option<u8>,
    /// Textures for the material. The key is the texture type and the value
    /// is the path to the texture.
    pub textures: FxHashMap<TextureType, PathBuf>,
}

impl Asset for Material {}

impl Material {
    /// Creates a new material from a loaded `MTL` file.
    ///
    /// # Arguments
    ///
    /// * `mtl` - The loaded material.
    /// * `filepath` - The path to the obj file.
    pub fn from_tobj_material(mtl: tobj::Material, filepath: &Path) -> Self {
        let mut textures = FxHashMap::default();
        let base = filepath.parent().unwrap_or_else(|| Path::new(""));
        match mtl.unknown_param.get("map_bump") {
            None => {
                // Try with "bump" instead
                if let Some(path) = mtl.unknown_param.get("bump").map(AsRef::<Path>::as_ref) {
                    if let Some(resolved) = resolve_path(path, base) {
                        textures.insert(TextureType::MapBump, resolved);
                    }
                }
            }
            Some(path) => {
                if let Some(resolved) = resolve_path(path.as_ref(), base) {
                    textures.insert(TextureType::MapBump, resolved);
                }
            }
        }

        if let Some(path) = mtl.unknown_param.get("map_disp") {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapDisp, resolved);
            }
        }

        if let Some(path) = mtl.unknown_param.get("map_decal") {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapDecal, resolved);
            }
        }

        if let Some(path) = mtl.ambient_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKa, resolved);
            }
        }

        if let Some(path) = mtl.diffuse_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKd, resolved);
            }
        }

        if let Some(path) = mtl.specular_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKs, resolved);
            }
        }

        if let Some(path) = mtl.shininess_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapNs, resolved);
            }
        }

        if let Some(path) = mtl.dissolve_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapD, resolved);
            }
        }

        if let Some(path) = mtl.normal_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapNorm, resolved);
            }
        }

        Self {
            name: mtl.name.into(),
            ka: mtl.ambient,
            kd: mtl.diffuse,
            ks: mtl.specular,
            ns: mtl.shininess,
            ni: mtl.optical_density,
            opacity: mtl.dissolve,
            illumination_model: mtl.illumination_model,
            textures,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: SmlString::from("material_default"),
            ka: Some([1.0, 1.0, 1.0]),
            kd: Some([0.6, 0.8, 0.3]),
            ks: Some([1.0, 0.0, 0.0]),
            ns: Some(1.0),
            ni: Some(0.0),
            opacity: Some(1.0),
            illumination_model: Some(0),
            textures: FxHashMap::default(),
        }
    }
}

// TODO: renamed to GpuMaterial
/// Material parameters that are uploaded to the GPU.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuMaterial {
    pub ka: [f32; 4],
    pub kd: [f32; 4],
    pub ks: [f32; 4],
    pub ns: f32,
    pub ni: f32,
    pub d: f32,
    pub illum: u32,

    pub map_ka: u32,
    pub map_kd: u32,
    pub map_ks: u32,
    pub map_ns: u32,

    pub map_d: u32,
    pub map_bump: u32,
    pub map_disp: u32,
    pub map_decal: u32,

    pub map_norm: u32,
    _padding: [u32; 3],
}

static_assertions::assert_eq_size!(GpuMaterial, [u8; 112]);

impl Asset for GpuMaterial {}

impl GpuMaterial {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;

    /// Create a `MaterialUniform` from a `Material`.
    ///
    /// Note that the texture indices are not set.
    pub fn from_material(mtl: &Material) -> Self {
        let ka = mtl.ka.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let kd = mtl.kd.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        let ks = mtl.ks.map(|c| [c[0], c[1], c[2], 0.0]).unwrap_or([0.0; 4]);
        Self {
            ka,
            kd,
            ks,
            ns: mtl.ns.unwrap_or(0.0),
            ni: mtl.ni.unwrap_or(0.0),
            d: mtl.opacity.unwrap_or(1.0),
            illum: mtl.illumination_model.unwrap_or(0) as u32,
            map_ka: u32::MAX,
            map_kd: u32::MAX,
            map_ks: u32::MAX,
            map_ns: u32::MAX,
            map_d: u32::MAX,
            map_bump: u32::MAX,
            map_disp: u32::MAX,
            map_decal: u32::MAX,
            map_norm: u32::MAX,
            _padding: [0; 3],
        }
    }
}

/// A collection of materials that uploaded to the GPU.
pub struct MaterialBundle {
    /// Buffer containing the material data.
    pub buffer: wgpu::Buffer,
    /// Bind group for the material buffer.
    pub bind_group: wgpu::BindGroup,
    /// Number of materials in the bundle.
    pub n_materials: u32,
}

impl Deref for MaterialBundle {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl MaterialBundle {
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shading_materials_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(GpuMaterial::SIZE),
                },
                count: None,
            }],
        })
    }

    pub fn default(device: &wgpu::Device) -> Self {
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_material_bundle_buffer"),
            contents: bytemuck::cast_slice(&[GpuMaterial::from_material(&Material::default())]),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });
        let layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("default_material_bundle_bind_group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
        });
        Self {
            buffer: material_buffer,
            bind_group,
            n_materials: 1,
        }
    }

    pub fn new(device: &wgpu::Device, mtls: &[GpuMaterial]) -> Self {
        log::debug!(
            "Creating material bundle with {} materials: \n{:?}",
            mtls.len(),
            mtls
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mtls),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });
        let layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            buffer,
            bind_group,
            n_materials: mtls.len() as u32,
        }
    }
}

impl Asset for MaterialBundle {}

fn resolve_path(path: &Path, base: &Path) -> Option<PathBuf> {
    log::debug!("Resolving path: {:?} with base: {:?}", path, base);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[pyo3::pymethods]
impl Material {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[setter]
    pub fn set_kd(&mut self, kd: [f32; 3]) {
        self.kd = Some(kd);
    }
}
