use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use pyo3::{pyclass, pymethods};

type Vec2 = [f32; 2];
type Vec3 = [f32; 3];
type Vec4 = [f32; 4];

pub trait AttribContainer {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> AttribContainer for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

type VertAttribs = HashMap<String, Arc<dyn AttribContainer + Send + Sync + 'static>>;

#[derive(Clone)]
#[pyclass]
pub struct Mesh {
    pub vert_attribs: VertAttribs,
    pub indices: Vec<u32>,
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("vert_attribs", &self.vert_attribs.keys())
            .finish()
    }
}

#[pymethods]
impl Mesh {
    #[new]
    pub fn new() -> Self {
        let mut vert_attribs: VertAttribs = HashMap::new();
        vert_attribs.insert("position".to_string(), Arc::new(Vec::<Vec3>::new()));
        vert_attribs.insert("normal".to_string(), Arc::new(Vec::<Vec3>::new()));
        vert_attribs.insert("uv0".to_string(), Arc::new(Vec::<Vec2>::new()));
        vert_attribs.insert("uv1".to_string(), Arc::new(Vec::<Vec2>::new()));
        vert_attribs.insert("color0".to_string(), Arc::new(Vec::<Vec4>::new()));
        vert_attribs.insert("color1".to_string(), Arc::new(Vec::<Vec4>::new()));
        vert_attribs.insert("tangent".to_string(), Arc::new(Vec::<Vec3>::new()));
        vert_attribs.insert("bitangent".to_string(), Arc::new(Vec::<Vec3>::new()));
        Self {
            vert_attribs,
            indices: vec![]
        }
    }

    // /// Validate that all vertex attributes have the same length.
    // pub fn validate(&self) -> Result<(), >

    pub fn compute_normals(&mut self) {}
    pub fn compute_tangents(&mut self) {}
}
