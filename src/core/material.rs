use crate::core::assets::Asset;
use bytemuck::{Pod, Zeroable};
use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};
use tobj::NormalTexture;
use wgpu::util::DeviceExt;

use crate::core::{FxHashMap, FxHasher, SmlString};

#[path = "material_py.rs"]
pub mod py;

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
    Unknown,  // unknown
}

/// Material name counter.
static MATERIAL_NAME_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Material description derived from a `MTL` file.
///
/// Material is a collection of parameters that describe how the surface of an
/// object reflects light. The parameters are used by the shading algorithm to
/// determine the color of a surface at a given point.
///
/// Materials are identified by a it's name, the name should be unique.
#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name.
    pub name: SmlString,
    /// Ambient color. `Ka` in the `MTL` spec.
    pub ambient: Option<[f32; 3]>,
    /// Diffuse color. `Kd` in the `MTL` spec.
    pub diffuse: Option<[f32; 3]>,
    /// Specular color. `Ks` in the `MTL` spec.
    pub specular: Option<[f32; 3]>,
    /// Shininess or glossiness. `Ns` in the `MTL` spec.
    pub shininess: Option<f32>,
    /// Optical density also known as index of refraction. Called
    /// `optical_density` in the `MTL` specc. Takes on a value between 0.001
    /// and 10.0. 1.0 means light does not bend as it passes through
    /// the object.
    pub refractive_index: Option<f32>,
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

/// Material equality is based on the name.
impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Material {
    /// Creates a new material with a unique name.
    pub fn new_with_name(name: &str) -> Self {
        Self {
            name: SmlString::from(name),
            ..Default::default()
        }
    }

    pub fn new() -> Self {
        Self {
            name: SmlString::from(format!(
                "material_{}",
                MATERIAL_NAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            )),
            ..Default::default()
        }
    }

    /// Creates a new material from a loaded `MTL` file.
    ///
    /// # Arguments
    ///
    /// * `mtl` - The loaded material.
    /// * `filepath` - The path to the obj file.
    pub fn from_tobj_material(mtl: tobj::Material, filepath: &Path) -> Self {
        let mut textures = FxHashMap::default();
        let base = filepath.parent().unwrap_or_else(|| Path::new(""));
        if let Some(path) = mtl.unknown_param.get("map_disp") {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapDisp, resolved);
            } else {
                log::error!("Displacement map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.unknown_param.get("map_decal") {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapDecal, resolved);
            } else {
                log::error!("Decal map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.ambient_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKa, resolved);
            } else {
                log::error!("Ambient map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.diffuse_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKd, resolved);
            } else {
                log::error!("Diffuse map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.specular_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapKs, resolved);
            } else {
                log::error!("Specular map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.shininess_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapNs, resolved);
            } else {
                log::error!("Shininess map can't be loaded: {:?}", path);
            }
        }

        if let Some(path) = mtl.dissolve_texture.as_ref() {
            if let Some(resolved) = resolve_path(path.as_ref(), base) {
                textures.insert(TextureType::MapD, resolved);
            } else {
                log::error!("Opacity map can't be loaded: {:?}", path);
            }
        }

        if let Some(tex) = mtl.normal_texture.as_ref() {
            match tex {
                NormalTexture::BumpMap(path) => {
                    if let Some(resolved) = resolve_path(path.as_ref(), base) {
                        textures.insert(TextureType::MapBump, resolved);
                    } else {
                        log::error!("Bump map can't be loaded: {:?}", path);
                    }
                }
                NormalTexture::NormalMap(path) => {
                    if let Some(resolved) = resolve_path(path.as_ref(), base) {
                        textures.insert(TextureType::MapNorm, resolved);
                    } else {
                        log::error!("Normal map can't be loaded: {:?}", path);
                    }
                }
            }
        }

        Self {
            name: mtl.name.into(),
            ambient: mtl.ambient,
            diffuse: mtl.diffuse,
            specular: mtl.specular,
            shininess: mtl.shininess,
            refractive_index: mtl.optical_density,
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
            ambient: Some([1.0, 1.0, 1.0]),
            diffuse: Some([0.7, 0.7, 0.7]),
            specular: Some([0.5, 0.5, 0.5]),
            shininess: Some(10.0),
            refractive_index: Some(1.0),
            opacity: Some(1.0),
            illumination_model: Some(2),
            textures: FxHashMap::default(),
        }
    }
}

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
        let ka = mtl
            .ambient
            .map(|c| [c[0], c[1], c[2], 0.0])
            .unwrap_or([0.0; 4]);
        let kd = mtl
            .diffuse
            .map(|c| [c[0], c[1], c[2], 0.0])
            .unwrap_or([0.0; 4]);
        let ks = mtl
            .specular
            .map(|c| [c[0], c[1], c[2], 0.0])
            .unwrap_or([0.0; 4]);
        Self {
            ka,
            kd,
            ks,
            ns: mtl.shininess.unwrap_or(0.0),
            ni: mtl.refractive_index.unwrap_or(0.0),
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
    /// List of materials (hash values of the material names).
    pub materials: Vec<u64>,
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
    // TODO: use the same as the one in BlinnPhongRenderPass
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shading_materials_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        let material = Material::default();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_material_bundle_buffer"),
            contents: bytemuck::cast_slice(&[GpuMaterial::from_material(&material)]),
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
        let mut hasher = FxHasher::default();
        hasher.write(material.name.as_bytes());
        Self {
            materials: vec![hasher.finish()],
            buffer: material_buffer,
            bind_group,
            n_materials: 1,
        }
    }

    pub fn new<'a, M>(device: &wgpu::Device, materials: M, mtls: &[GpuMaterial]) -> Self
    where
        M: Iterator<Item = &'a Material>,
    {
        log::debug!(
            "Creating material bundle with {} materials: \n{:?}",
            mtls.len(),
            mtls
        );
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mtls),
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
        let materials = materials
            .map(|m| {
                let mut hasher = FxHasher::default();
                hasher.write(m.name.as_bytes());
                hasher.finish()
            })
            .collect();
        log::debug!("Material bundle created with materials: {:?}", materials);
        Self {
            materials,
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
    path.canonicalize()
        .and_then(|path| {
            log::debug!("Resolved path: {:?}", path);
            if path.exists() {
                Ok(path)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Path does not exist: {:?}", path),
                ))
            }
        })
        .ok()
}

#[pyo3::pyclass]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllumModel {
    ColorOnAmbientOff = 0,
    ColorOnAmbientOn = 1,
    HighlightOn = 2,
    ReflectionOnRayTraceOn = 3,
    TransparencyGlassOnReflectionRayTraceOn = 4,
    ReflectionFresnelOnRayTraceOn = 5,
    TransparencyRefractionOnReflectionFresnelOffRayTraceOn = 6,
    TransparencyRefractionOnReflectionFresnelOnRayTraceOn = 7,
    ReflectionOnRayTraceOff = 8,
    TransparencyGlassOnReflectionRayTraceOff = 9,
    CastsShadowsOntoInvisibleSurfaces = 10,

    DiffuseNoShading = 11,
    SpecularNoShading = 12,
    TextureCoordinates = 13,
    NormalInViewSpace = 14,
}

impl From<u8> for IllumModel {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ColorOnAmbientOff,
            1 => Self::ColorOnAmbientOn,
            2 => Self::HighlightOn,
            3 => Self::ReflectionOnRayTraceOn,
            4 => Self::TransparencyGlassOnReflectionRayTraceOn,
            5 => Self::ReflectionFresnelOnRayTraceOn,
            6 => Self::TransparencyRefractionOnReflectionFresnelOffRayTraceOn,
            7 => Self::TransparencyRefractionOnReflectionFresnelOnRayTraceOn,
            8 => Self::ReflectionOnRayTraceOff,
            9 => Self::TransparencyGlassOnReflectionRayTraceOff,
            10 => Self::CastsShadowsOntoInvisibleSurfaces,
            11 => Self::DiffuseNoShading,
            12 => Self::SpecularNoShading,
            13 => Self::TextureCoordinates,
            14 => Self::NormalInViewSpace,
            _ => panic!("Unknown illumination model: {}", value),
        }
    }
}
