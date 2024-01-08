use crate::core::{
    mesh::{AttribContainer, Indices, Mesh, SubMesh, VertexAttribute},
    Alignment, Color, Material,
};
use glam::Vec3;
use numpy as np;
use pyo3::Python;
use std::path::PathBuf;

/// Topology of a mesh primitive.
#[pyo3::pyclass]
#[pyo3(name = "Topology")]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub enum PyTopology {
    /// Vertex data is a list of points. Each vertex is a new point.
    PointList = 0,
    /// Vertex data is a list of lines. Each pair of vertices composes a new
    /// line.
    ///
    /// Vertices `0 1 2 3` create two lines `0 1` and `2 3`
    LineList = 1,
    /// Vertex data is a strip of lines. Each set of two adjacent vertices form
    /// a line.
    ///
    /// Vertices `0 1 2 3` create three lines `0 1`, `1 2`, and `2 3`.
    LineStrip = 2,
    /// Vertex data is a list of triangles. Each set of 3 vertices composes a
    /// new triangle.
    ///
    /// Vertices `0 1 2 3 4 5` create two triangles `0 1 2` and `3 4 5`
    #[default]
    TriangleList = 3,
    /// Vertex data is a triangle strip. Each set of three adjacent vertices
    /// form a triangle.
    ///
    /// Vertices `0 1 2 3 4 5` create four triangles `0 1 2`, `2 1 3`, `2 3 4`,
    /// and `4 3 5`
    TriangleStrip = 4,
}

impl From<wgpu::PrimitiveTopology> for PyTopology {
    fn from(topology: wgpu::PrimitiveTopology) -> Self {
        match topology {
            wgpu::PrimitiveTopology::PointList => Self::PointList,
            wgpu::PrimitiveTopology::LineList => Self::LineList,
            wgpu::PrimitiveTopology::LineStrip => Self::LineStrip,
            wgpu::PrimitiveTopology::TriangleList => Self::TriangleList,
            wgpu::PrimitiveTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

impl From<PyTopology> for wgpu::PrimitiveTopology {
    fn from(value: PyTopology) -> Self {
        match value {
            PyTopology::PointList => Self::PointList,
            PyTopology::LineList => Self::LineList,
            PyTopology::LineStrip => Self::LineStrip,
            PyTopology::TriangleList => Self::TriangleList,
            PyTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

#[pyo3::pymethods]
impl Mesh {
    #[new]
    #[pyo3(signature = (name=None, topology=PyTopology::TriangleList))]
    pub fn new_py(name: Option<String>, topology: PyTopology) -> Self {
        match name {
            None => Self::new(topology.into()),
            Some(name) => Self::new_with_name(name.as_str(), topology.into()),
        }
    }

    #[staticmethod]
    #[pyo3(name = "create_cube")]
    pub fn new_cube_py(length: f32) -> Self {
        Self::cube(length)
    }

    #[staticmethod]
    #[pyo3(name = "create_quad")]
    pub fn new_quad_py(length: f32, align: Alignment) -> Self {
        Self::plane(length, align)
    }

    #[staticmethod]
    #[pyo3(name = "create_sphere")]
    pub fn new_sphere_py(radius: f32, segments: u32, rings: u32) -> Self {
        Self::sphere(radius, segments, rings)
    }

    #[staticmethod]
    #[pyo3(name = "create_grid")]
    pub fn new_grid_py(
        width: f32,
        height: f32,
        spacing: (f32, f32),
        align: Alignment,
        color: Color,
    ) -> Self {
        Self::grid(width, height, spacing, align, color)
    }

    #[staticmethod]
    #[pyo3(name = "create_triangle")]
    pub fn new_triangle_py(
        p0: &np::PyArray2<f32>,
        p1: &np::PyArray2<f32>,
        p2: &np::PyArray2<f32>,
    ) -> Self {
        log::debug!("Creating triangle.");
        Python::with_gil(|_py| {
            let v0 = Vec3::from_slice(p0.readonly().as_slice().unwrap());
            let v1 = Vec3::from_slice(p1.readonly().as_slice().unwrap());
            let v2 = Vec3::from_slice(p2.readonly().as_slice().unwrap());
            Self::triangle(&[v0, v1, v2])
        })
    }

    #[staticmethod]
    #[pyo3(name = "load_from")]
    pub fn load_from_py(path: &str) -> Self {
        let path = PathBuf::from(path);
        Self::load_from_obj(&path)
    }

    #[deprecated]
    pub fn apply_material(&mut self, material: Material) {
        self.set_material(material)
    }

    /// Applies a single material to the whole mesh.
    pub fn set_material(&mut self, material: Material) {
        if self.materials.is_none() {
            self.materials = Some(Vec::new());
        }
        self.materials.as_mut().unwrap().push(material);
        let material_index = self.materials.as_ref().unwrap().len() as u32 - 1;
        self.sub_meshes = Some(vec![SubMesh {
            range: 0..self.indices.as_ref().unwrap().len() as u32,
            material: Some(material_index),
        }]);
    }

    /// Sets the materials of the mesh.
    #[setter]
    pub fn set_materials(&mut self, materials: Option<Vec<Material>>) {
        self.materials = materials;
    }

    /// Returns the materials of the mesh.
    #[getter]
    pub fn get_materials(&self) -> Option<Vec<Material>> {
        self.materials.clone()
    }

    /// Appends a material to the current list of materials of the mesh.
    pub fn append_material(&mut self, material: Material) -> u32 {
        if self.materials.is_none() {
            self.materials = Some(Vec::new());
        }
        self.materials.as_mut().unwrap().push(material);
        self.materials.as_ref().unwrap().len() as u32 - 1
    }

    /// Appends a list of materials to the current list of materials of the
    /// mesh.
    pub fn append_materials(&mut self, materials: Vec<Material>) -> Vec<u32> {
        if self.materials.is_none() {
            self.materials = Some(Vec::new());
        }
        let existing = self.materials.as_mut().unwrap();
        let base_index = existing.len() as u32;
        let num = materials.len() as u32;
        existing.extend(materials);
        (base_index..base_index + num).collect()
    }

    /// Sets the submeshes of the mesh.
    #[setter]
    pub fn set_sub_meshes(&mut self, sub_meshes: Option<Vec<SubMesh>>) {
        // Check that the submeshes are valid.
        let indices_len = self.indices.as_ref().unwrap().len() as u32;
        match &sub_meshes {
            Some(sub_meshes) => {
                for submesh in sub_meshes {
                    if submesh.range.start > indices_len || submesh.range.end > indices_len {
                        panic!(
                            "Submesh range is greater than the number of indices {}",
                            indices_len
                        );
                    }
                    if submesh.range.end < submesh.range.start {
                        panic!("Submesh range end is smaller than start");
                    }
                }
            }
            None => {}
        }
        self.sub_meshes = sub_meshes;
    }

    #[setter]
    pub fn set_positions(&mut self, vertices: Option<Vec<[f32; 3]>>) {
        if let Some(vertices) = vertices {
            self.attributes
                .insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        }
    }

    #[setter]
    pub fn set_normals(&mut self, normals: Vec<[f32; 3]>) {
        self.attributes
            .insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
    }

    #[setter]
    pub fn set_texcoords(&mut self, uvs: Option<Vec<[f32; 2]>>) {
        if let Some(uvs) = uvs {
            self.attributes
                .insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        }
    }

    #[setter]
    pub fn set_triangles(&mut self, triangles: Option<Vec<[u32; 3]>>) {
        if let Some(triangles) = triangles {
            self.indices = Some(Indices::U32(triangles.into_iter().flatten().collect()));
        }
    }

    #[getter]
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[setter]
    pub fn set_name(&mut self, name: String) {
        self.name = name.into();
    }

    /// Computes per vertex normals for the mesh from the UVs.
    #[pyo3(name = "compute_normals")]
    pub fn compute_normals_py(&mut self) {
        self.compute_normals();
    }

    /// Computes per vertex tangents for the mesh from the UVs.
    #[pyo3(name = "compute_tangents")]
    pub fn compute_tangents_py(&mut self) {
        self.compute_tangents();
    }
}
