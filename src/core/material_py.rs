use crate::core::{Color, IllumModel, Material, SmlString, TextureType};
use pyo3::types::PyDict;
use std::path::PathBuf;

#[pyo3::pymethods]
impl Material {
    #[new]
    #[pyo3(signature = (name=None))]
    pub fn new_py(name: Option<String>) -> Self {
        match name {
            None => Self::new(),
            Some(name) => Self::new_with_name(name.as_str()),
        }
    }

    #[setter]
    pub fn set_name(&mut self, name: &str) {
        self.name = SmlString::from(name);
    }

    #[getter]
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[setter]
    pub fn set_diffuse(&mut self, kd: Color) {
        self.diffuse = Some([kd.r as f32, kd.g as f32, kd.b as f32]);
    }

    #[setter]
    #[deprecated(note = "Use `set_diffuse` instead")]
    pub fn set_kd(&mut self, kd: [f32; 3]) {
        self.diffuse = Some(kd);
    }

    #[getter]
    pub fn get_diffuse(&self) -> Option<[f32; 3]> {
        self.diffuse
    }

    #[setter]
    pub fn set_ambient(&mut self, ka: Color) {
        self.ambient = Some([ka.r as f32, ka.g as f32, ka.b as f32]);
    }

    #[getter]
    pub fn get_ambient(&self) -> Option<[f32; 3]> {
        self.ambient
    }

    #[setter]
    pub fn set_specular(&mut self, ks: Color) {
        self.specular = Some([ks.r as f32, ks.g as f32, ks.b as f32]);
    }

    #[getter]
    pub fn get_specular(&self) -> Option<[f32; 3]> {
        self.specular
    }

    #[setter]
    pub fn set_shininess(&mut self, ns: f32) {
        self.shininess = Some(ns);
    }

    #[getter]
    pub fn get_shininess(&self) -> Option<f32> {
        self.shininess
    }

    #[setter]
    pub fn set_illum_model(&mut self, illum: IllumModel) {
        self.illumination_model = Some(illum as u8);
    }

    #[getter]
    pub fn get_illum_model(&self) -> Option<IllumModel> {
        self.illumination_model.map(|i| i.into())
    }

    /// Sets the textures for the material.
    ///
    /// The textures are passed as a dictionary where the key is the texture
    /// type and the value is the path to the texture.
    #[setter]
    pub fn set_textures(&mut self, textures: &PyDict) {
        if textures.is_empty() {
            return;
        }
        for (key, value) in textures.iter() {
            let key: String = key
                .extract()
                .expect("Failed to extract texture type string");
            if value.is_none() {
                log::warn!("Texture path is None for key: {}", key);
                continue;
            }
            let value: String = value
                .extract()
                .expect("Failed to downcast texture path to string");

            let texture_type = match key.to_lowercase().as_str() {
                "map_ka" | "ambient_texture" => TextureType::MapKa,
                "map_kd" | "diffuse_texture" => TextureType::MapKd,
                "map_ks" | "specular_texture" => TextureType::MapKs,
                "map_ns" | "shininess_texture" => TextureType::MapNs,
                "map_d" | "opacity_texture" => TextureType::MapD,
                "map_bump" | "bump_texture" => TextureType::MapBump,
                "map_disp" | "displacement_texture" => TextureType::MapDisp,
                "map_decal" | "decal_texture" => TextureType::MapDecal,
                "map_norm" | "normal_texture" => TextureType::MapNorm,
                _ => TextureType::Unknown,
            };

            match texture_type {
                TextureType::Unknown => {
                    log::warn!("Unknown texture type: {}", key);
                    continue;
                }
                texture_type => {
                    self.textures
                        .insert(texture_type, PathBuf::from(value.as_str()));
                }
            }
        }
    }
}
