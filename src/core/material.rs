use crate::core::assets::Asset;
use std::path::PathBuf;

use crate::core::{FxHashMap, SmlString};

/// Material description derived from a `MTL` file.
#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name.
    pub name: String,
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
    /// Key value pairs of any unrecognized parameters encountered while parsing
    /// the material.
    pub unknown_params: Option<FxHashMap<SmlString, String>>,
}

impl Asset for Material {}
