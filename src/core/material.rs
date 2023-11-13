use crate::core::assets::{Asset, Handle};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use wgpu::util::DeviceExt;

use crate::{
    core::{SmlString, Texture},
    render::rpass::MaterialUniform,
};

/// Material description derived from a `MTL` file.
// #[pyo3::pyclass]
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
    /// Texture for ambient color. `map_Ka` in the `MTL` spec.
    pub map_ka: Option<PathBuf>,
    /// Texture for diffuse color. `map_Kd` in the `MTL` spec.
    pub map_kd: Option<PathBuf>,
    /// Texture for specular color. `map_Ks` in the `MTL` spec.
    pub map_ks: Option<PathBuf>,
    /// Texture for specular exponent/shininess/glossiness. `map_Ns` in the
    /// `MTL` spec.
    pub map_ns: Option<PathBuf>,
    /// Texture for alpha/opacity. `map_d` in the `MTL` spec.
    pub map_d: Option<PathBuf>,
    /// Texture for bump map. `map_bump`/`bump` in the `MTL` spec.
    pub map_bump: Option<PathBuf>,
    /// Texture for displacement map. `map_disp`/`disp` in the `MTL` spec.
    pub map_disp: Option<PathBuf>,
    /// Texture for stencil decal. `map_decal`/`decal` in the `MTL` spec.
    pub map_decal: Option<PathBuf>,
    /// Texture for normal map. `map_norm`/`norm` in the `MTL` spec.
    pub map_norm: Option<PathBuf>,
    /// The illumnination model to use for this material. The different
    /// illumination models are specified in the [`MTL` spec](http://paulbourke.net/dataformats/mtl/).
    pub illumination_model: Option<u8>,
}

impl Asset for Material {}

impl Material {
    pub fn from_tobj_material(mtl: tobj::Material, relative_to: &Path) -> Self {
        let map_bump = match mtl.unknown_param.get("map_bump").map(AsRef::<Path>::as_ref) {
            None => mtl.unknown_param.get("bump").map(AsRef::<Path>::as_ref),
            Some(v) => Some(v),
        }
        .map(|v| resolve_path(&v, relative_to))
        .flatten();
        let map_disp = match mtl.unknown_param.get("map_disp").map(AsRef::<Path>::as_ref) {
            None => mtl.unknown_param.get("disp").map(AsRef::<Path>::as_ref),
            Some(v) => Some(v),
        }
        .map(|v| resolve_path(&v, relative_to))
        .flatten();
        let map_decal = match mtl
            .unknown_param
            .get("map_decal")
            .map(AsRef::<Path>::as_ref)
        {
            None => mtl.unknown_param.get("decal").map(AsRef::<Path>::as_ref),
            Some(v) => Some(v),
        }
        .map(|v| resolve_path(&v, relative_to))
        .flatten();

        Self {
            name: mtl.name.into(),
            ka: mtl.ambient,
            kd: mtl.diffuse,
            ks: mtl.specular,
            ns: mtl.shininess,
            ni: mtl.optical_density,
            opacity: mtl.dissolve,
            map_ka: mtl
                .ambient_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            map_kd: mtl
                .diffuse_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            map_ks: mtl
                .specular_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            map_ns: mtl
                .shininess_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            map_d: mtl
                .dissolve_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            map_bump,
            map_disp,
            map_decal,
            map_norm: mtl
                .normal_texture
                .map(|v| resolve_path(v.as_ref(), relative_to))
                .flatten(),
            illumination_model: mtl.illumination_model,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: SmlString::from("material_default"),
            ka: None,
            kd: None,
            ks: None,
            ns: None,
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
                    min_binding_size: wgpu::BufferSize::new(MaterialUniform::SIZE),
                },
                count: None,
            }],
        })
    }

    pub fn default(device: &wgpu::Device) -> Self {
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_material_bundle_buffer"),
            contents: bytemuck::cast_slice(&[MaterialUniform::default()]),
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

    pub fn new(device: &wgpu::Device, mtls: &[MaterialUniform]) -> Self {
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

/// A collection of textures that uploaded to the GPU.
pub struct TextureBundle(wgpu::Buffer);

impl Asset for MaterialBundle {}

impl Asset for TextureBundle {}

pub struct GpuMaterial {
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
    /// Texture for ambient color. `map_Ka` in the `MTL` spec.
    pub map_ka: Option<Handle<Texture>>,
    /// Texture for diffuse color. `map_Kd` in the `MTL` spec.
    pub map_kd: Option<Handle<Texture>>,
    /// Texture for specular color. `map_Ks` in the `MTL` spec.
    pub map_ks: Option<Handle<Texture>>,
    /// Texture for specular exponent/shininess/glossiness. `map_Ns` in the
    /// `MTL` spec.
    pub map_ns: Option<Handle<Texture>>,
    /// Texture for alpha/opacity. `map_d` in the `MTL` spec.
    pub map_d: Option<Handle<Texture>>,
    /// Texture for bump map. `map_bump`/`bump` in the `MTL` spec.
    pub map_bump: Option<Handle<Texture>>,
    /// Texture for displacement map. `map_disp`/`disp` in the `MTL` spec.
    pub map_disp: Option<Handle<Texture>>,
    /// Texture for stencil decal. `map_decal`/`decal` in the `MTL` spec.
    pub map_decal: Option<Handle<Texture>>,
    /// Texture for normal map. `map_norm`/`norm` in the `MTL` spec.
    pub map_norm: Option<Handle<Texture>>,
    /// The illumnination model to use for this material. The different
    /// illumination models are specified in the [`MTL` spec](http://paulbourke.net/dataformats/mtl/).
    pub illumination_model: Option<u8>,
}

impl Asset for GpuMaterial {}

fn resolve_path(path: &Path, base: &Path) -> Option<PathBuf> {
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
