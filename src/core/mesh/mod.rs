use glam::Vec3;
use numpy as np;
use pyo3::Python;
use rustc_hash::FxHashMap;
use std::{
    fmt::Debug,
    ops::Range,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

mod attribute;

use crate::core::{
    assets::{Asset, Handle},
    Alignment, Material, MaterialBundle, TextureBundle,
};
pub use attribute::*;

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

pub trait IndexType: Copy {
    fn as_u32(&self) -> u32;
    fn as_usize(&self) -> usize;
}

impl IndexType for u32 {
    fn as_u32(&self) -> u32 {
        *self
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl IndexType for u16 {
    fn as_u32(&self) -> u32 {
        *self as u32
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

/// Indices of a mesh.
#[derive(Clone, Debug)]
pub enum Indices {
    U32(Vec<u32>),
    U16(Vec<u16>),
}

impl Indices {
    /// Returns the number of indices in the index buffer.
    pub fn len(&self) -> usize {
        match self {
            Self::U32(indices) => indices.len(),
            Self::U16(indices) => indices.len(),
        }
    }

    /// Returns the number of bytes required to store the index buffer.
    pub fn n_bytes(&self) -> usize {
        match self {
            Self::U32(indices) => indices.len() * std::mem::size_of::<u32>(),
            Self::U16(indices) => indices.len() * std::mem::size_of::<u16>(),
        }
    }

    /// Returns the index buffer as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::U32(indices) => bytemuck::cast_slice(indices),
            Self::U16(indices) => bytemuck::cast_slice(indices),
        }
    }

    /// Returns the index buffer format as a wgpu::IndexFormat.
    pub fn format(&self) -> wgpu::IndexFormat {
        match self {
            Self::U32(_) => wgpu::IndexFormat::Uint32,
            Self::U16(_) => wgpu::IndexFormat::Uint16,
        }
    }
}

/// A submesh is a range of indices, it specifies a range of indices to be
/// rendered with a specific material.
#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct SubMesh {
    /// Range of indices/vertices of the submesh.
    pub range: Range<u32>,
    /// Material of the submesh (index into the material array of the mesh).
    /// If the material is None, the submesh uses the default material.
    pub material: Option<u32>,
}

#[pyo3::pymethods]
impl SubMesh {
    #[new]
    pub fn new(start: u32, end: u32, index: u32) -> Self {
        Self {
            range: start..end,
            material: Some(index),
        }
    }
}

/// Mesh id counter. 0 and 1 are reserved for the default cube and quad.
static MESH_ID: AtomicU64 = AtomicU64::new(2);

/// A mesh is a collection of vertices with optional indices and materials.
/// Vertices can have different attributes such as position, normal, uv, etc.
#[pyo3::pyclass]
pub struct Mesh {
    /// Unique id of the mesh.
    pub(crate) id: u64,
    /// Topology of the mesh primitive.
    pub(crate) topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    pub(crate) attributes: VertexAttributes,
    /// Indices of the mesh.
    pub(crate) indices: Option<Indices>,
    /// Sub-meshes of the mesh. If the mesh has no sub-meshes, it is assumed
    /// that the entire mesh is using the default material.
    pub(crate) sub_meshes: Option<Vec<SubMesh>>,
    /// Path to the mesh file, if it's loaded from a file.
    pub(crate) path: Option<PathBuf>,
    /// Materials of the mesh.
    pub(crate) materials: Option<Vec<Material>>,
}

impl Asset for Mesh {}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            topology: self.topology,
            attributes: self.attributes.clone(),
            indices: self.indices.clone(),
            sub_meshes: self.sub_meshes.clone(),
            path: self.path.clone(),
            materials: self.materials.clone(),
        }
    }
}

#[pyo3::pymethods]
impl Mesh {
    #[new]
    pub fn new(topology: PyTopology) -> Self {
        Self {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            path: None,
            topology: topology.into(),
            attributes: VertexAttributes::default(),
            indices: None,
            sub_meshes: None,
            materials: None,
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
    pub fn set_materials(&mut self, materials: Vec<Material>) {
        self.materials = Some(materials);
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
    pub fn set_sub_meshes(&mut self, sub_meshes: Vec<SubMesh>) {
        self.sub_meshes = Some(sub_meshes);
    }

    /// Computes per vertex tangents for the mesh from the UVs.
    pub fn update_tangents(&mut self) {
        if self.attributes.0.contains_key(&VertexAttribute::TANGENT) {
            log::warn!("Mesh already has tangents and bitangents. Skipping tangent computation.");
            return;
        }
        let vertices = self
            .attributes
            .0
            .get(&VertexAttribute::POSITION)
            .unwrap()
            .as_slice::<[f32; 3]>();
        let uvs = self
            .attributes
            .0
            .get(&VertexAttribute::UV)
            .unwrap()
            .as_slice::<[f32; 2]>();
        let normals = self
            .attributes
            .0
            .get(&VertexAttribute::NORMAL)
            .unwrap()
            .as_slice::<[f32; 3]>();
        let mut tangents: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; vertices.len()];
        match &self.indices {
            None => {
                panic!("Indices are required to compute the bi/tangents");
            }
            Some(indices) => match indices {
                Indices::U32(indices) => {
                    compute_tangents(&vertices, indices, &uvs, normals, &mut tangents);
                }
                Indices::U16(indices) => {
                    compute_tangents(&vertices, indices, &uvs, normals, &mut tangents)
                }
            },
        }
        // println!("vertices: {:?}", vertices);
        // println!("uvs: {:?}", uvs);
        // println!("normals: {:?}", normals);
        // println!("tangents: {:?}", tangents);
        self.attributes
            .insert(VertexAttribute::TANGENT, AttribContainer::new(&tangents));
    }
}

pub struct VertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

impl Mesh {
    #[rustfmt::skip]
    /// Creates a unit cube of side length 1 centered at the origin.
    pub fn cube(length: f32) -> Self {
        let mut attributes = VertexAttributes::default();
        let half = length * 0.5;
        // Vertex positions for a unit cube centered at the origin.
        let vertices: [[f32; 3]; 24] = [
            // front (0.0, 0.0, 0.5)
            [-half, -half, half], [half, -half, half], [half, half, half], [-half, half, half],
            // back (0.0, 0.0, -half)
            [-half, -half, -half], [half, -half, -half], [half, half, -half], [-half, half, -half],
            // right (half, 0.0, 0.0)
            [half, -half, -half], [half, half, -half], [half, half, half], [half, -half, half],
            // left (-half, 0.0, 0.0)
            [-half, -half, half], [-half, half, half], [-half, half, -half], [-half, -half, -half],
            // top (0.0, half, 0.0)
            [half, half, -half], [-half, half, -half], [-half, half, half], [half, half, half],
            // bottom (0.0, -half, 0.0)
            [half, -half, half], [-half, -half, half], [-half, -half, -half], [half, -half, -half],
        ];
        // Vertex normals for a unit cube centered at the origin. Per vertex normals.
        let normals: [[f32; 3]; 24] = [
            // front (0.0, 0.0, 1.0)
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            // back (0.0, 0.0, -1.0)
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            // right (1.0, 0.0, 0.0)
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            // left (-1.0, 0.0, 0.0)
            [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
            // top (0.0, 1.0, 0.0)
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            // bottom (0.0, -1.0, 0.0)
            [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
        ];
        // Vertex indices for a unit cube centered at the origin.
        let indices: Vec<u16> = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 7, 6, 4, 6, 5, // 6, 5, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom */
        ];
        // Vertex UVs for a unit cube centered at the origin.
        let uvs: Vec<[f32; 2]> = vec![
            // front
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // back
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // right
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // left
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // top
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // bottom
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ];
        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh {
            id: 0,
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U16(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        };
        mesh.update_tangents();
        mesh
    }

    #[rustfmt::skip]
    /// Creates a unit quad of side length 1 centered at the origin.
    pub fn plane(length: f32, align: Alignment) -> Self {
        let mut attributes = VertexAttributes::default();
        let half = length * 0.5;
        let (vertices, normals) = match align {
            Alignment::XY => {
                let vertices: [[f32; 3]; 4] = [
                    [-half, -half, 0.0], [half, -half, 0.0], [half, half, 0.0], [-half, half, 0.0],
                ];
                let normals: [[f32; 3]; 4] = [
                    [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
                ];
                (vertices, normals)
            }
            Alignment::XZ => {
                let vertices: [[f32; 3]; 4] = [
                    [-half, 0.0, -half], [half, 0.0, -half], [half, 0.0, half], [-half, 0.0, half],
                ];
                let normals: [[f32; 3]; 4] = [
                    [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
                ];
                (vertices, normals)
            }
            Alignment::YZ => {
                let vertices: [[f32; 3]; 4] = [
                    [0.0, -half, -half], [0.0, half, -half], [0.0, half, half], [0.0, -half, half],
                ];
                let normals: [[f32; 3]; 4] = [
                    [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
                ];
                (vertices, normals)
            }
        };
        // Vertex indices for a unit quad centered at the origin.
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        // Vertex UVs for a unit quad centered at the origin.
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh {
            id: 1,
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U16(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        };
        mesh.update_tangents();
        mesh
    }

    /// Creates a sphere of centered at the origin.
    ///
    /// # Arguments
    ///
    /// * `radius` - Radius of the sphere.
    /// * `segments` - Number of segments around the sphere.
    /// * `rings` - Number of rings from the top to the bottom of the sphere.
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        let mut attributes = VertexAttributes::default();
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices = Vec::new();

        // Create the vertices.
        for ring in 0..=rings {
            let v = ring as f32 / rings as f32;
            let theta = v * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for segment in 0..=segments {
                let u = segment as f32 / segments as f32;
                let phi = u * std::f32::consts::PI * 2.0;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                vertices.push([radius * x, radius * y, radius * z]);
                normals.push([x, y, z]);
                uvs.push([u, v]);
            }
        }

        // Create the indices.
        for ring in 0..rings {
            for segment in 0..segments {
                let next_segment = segment + 1;
                let next_ring = ring + 1;

                indices.push(ring * (segments + 1) + segment);
                indices.push(next_ring * (segments + 1) + next_segment);
                indices.push(next_ring * (segments + 1) + segment);

                indices.push(ring * (segments + 1) + segment);
                indices.push(ring * (segments + 1) + next_segment);
                indices.push(next_ring * (segments + 1) + next_segment);
            }
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));

        let mut mesh = Mesh {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U32(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        };
        mesh.update_tangents();
        mesh
    }

    /// Creates a triangle with user defined vertices.
    pub fn triangle(vertices: &[Vec3]) -> Mesh {
        assert_eq!(vertices.len(), 3, "Triangle must have 3 vertices.");
        let mut attributes = VertexAttributes::default();

        // Create the normals.
        let v0 = vertices[0];
        let v1 = vertices[1];
        let v2 = vertices[2];
        let u = v1 - v0;
        let v = v2 - v0;
        let normal = u.cross(v).normalize();

        let normals: Vec<[f32; 3]> = vec![normal.into(); 3];
        let vertices: Vec<[f32; 3]> = vertices.iter().map(|v| (*v).into()).collect();
        let indices = vec![0u32, 1, 2];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));

        let mut mesh = Mesh {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U32(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        };
        mesh.update_tangents();
        mesh
    }

    /// Loads a mesh from a wavefront obj file.
    pub fn load_from_obj<P: AsRef<Path> + Debug + Copy>(path: P) -> Self {
        log::debug!("Loading mesh from {}.", path.as_ref().display());
        let options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };
        let (models, materials) = tobj::load_obj(path, &options)
            .map_err(|err| {
                log::error!("Failed to load mesh from {:?}: {}", path, err);
            })
            .unwrap();
        let materials = materials.expect("Failed to load materials.");
        log::debug!("- Loaded {} models.", models.len());
        log::debug!("- Loaded {} materials.", materials.len());
        log::debug!("-- Loaded materials: {:?}", materials);
        let mut attributes = VertexAttributes::default();
        let mut vertices: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut uvs: Vec<f32> = Vec::new();
        let mut indices = Vec::new();

        // Classify the submeshes by material.
        let mut sub_meshes = Vec::new();
        let mut sub_meshes_by_material = FxHashMap::default();
        for model in models.iter() {
            let mesh = &model.mesh;
            sub_meshes_by_material
                .entry(mesh.material_id)
                .or_insert_with(Vec::new)
                .push(mesh);
        }

        let mut index_start = 0;
        for (material_id, meshes) in sub_meshes_by_material.iter() {
            let mut sub_mesh = SubMesh {
                range: index_start..index_start,
                material: material_id.map(|id| id as u32),
            };
            for mesh in meshes.iter() {
                let mut mesh_indices = mesh.indices.clone();
                let index_offset = vertices.len() as u32 / 3;
                for idx in mesh_indices.iter_mut() {
                    *idx += index_offset;
                }
                let index_count = mesh_indices.len() as u32;
                vertices.append(&mut mesh.positions.clone());
                normals.append(&mut mesh.normals.clone());
                uvs.append(&mut mesh.texcoords.clone());
                indices.append(&mut mesh_indices);
                index_start += index_count;
            }
            sub_mesh.range.end = index_start;
            sub_meshes.push(sub_mesh);
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));

        if !normals.is_empty() {
            attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        }

        if !uvs.is_empty() {
            attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        }

        let id = MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        log::debug!("- Loaded mesh with id: {}.", id);

        let materials = materials
            .iter()
            .map(|m| Material::from_tobj_material(m.clone(), &path.as_ref()))
            .collect();

        log::debug!("- Processed materials: {:?}", materials);
        log::debug!("- Loaded submeshes: {:?}", sub_meshes);

        let mut mesh = Mesh {
            id,
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: if !indices.is_empty() {
                Some(Indices::U32(indices))
            } else {
                None
            },
            sub_meshes: Some(sub_meshes),
            path: None,
            materials: Some(materials),
        };
        mesh.update_tangents();
        // println!(
        //     "tangents: {:?}",
        //     mesh.attributes
        //         .0
        //         .get(&VertexAttribute::TANGENT)
        //         .unwrap()
        //         .as_slice::<[f32; 4]>()
        // );
        // println!(
        //     "normals: {:?}",
        //     mesh.attributes
        //         .0
        //         .get(&VertexAttribute::NORMAL)
        //         .unwrap()
        //         .as_slice::<[f32; 3]>()
        // );
        mesh
    }
}

/// A mesh on the GPU.
pub struct GpuMesh {
    /// Unique id of the mesh from which it was created.
    pub mesh_id: u64,
    /// Path to the mesh file, if it's loaded from a file.
    pub mesh_path: Option<PathBuf>,
    /// Topology of the mesh primitive.
    pub topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    pub vertex_attribute_ranges: Vec<(VertexAttribute, Range<u64>)>,
    /// Vertex count of the mesh.
    pub vertex_count: u32,
    /// Index buffer format of the mesh.
    pub index_format: Option<wgpu::IndexFormat>,
    /// Index buffer range inside the mesh data buffer. 0..0 if no index buffer.
    pub index_range: Range<u64>,
    /// Number of indices in the index buffer.
    pub index_count: u32,
    /// Sub-meshes of the mesh.
    pub sub_meshes: Option<Vec<SubMesh>>,
}

impl Asset for GpuMesh {}

impl GpuMesh {
    /// Creates a new empty gpu mesh.
    pub fn empty(topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            mesh_id: u64::MAX,
            mesh_path: None,
            topology,
            vertex_attribute_ranges: Vec::new(),
            vertex_count: 0,
            index_format: None,
            index_range: 0..0,
            index_count: 0,
            sub_meshes: None,
        }
    }

    /// Returns the range of the vertex attribute in the mesh data buffer, if it
    /// exists.
    pub fn get_vertex_attribute_range(&self, attribute: VertexAttribute) -> Option<Range<u64>> {
        self.vertex_attribute_ranges
            .iter()
            .find_map(|(attrib, range)| (*attrib == attribute).then_some(range.clone()))
    }
}

mod util {
    /// Helper function to normalize a vector
    pub fn normalize(v: &[f32; 3]) -> [f32; 3] {
        let length = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
        [v[0] / length, v[1] / length, v[2] / length]
    }

    /// Helper function to calculate the dot product of two vectors
    pub fn dot(v1: &[f32; 3], v2: &[f32; 3]) -> f32 {
        v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
    }

    /// Helper function to calculate the cross product of two vectors
    pub fn cross(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
        [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ]
    }

    /// Helper function to calculate the rejection of a vector from
    /// another which is the perpendicular part of the decomposition
    /// of the first vector onto the second.
    pub fn rejection(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
        let b_norm = normalize(b);
        let dot = dot(a, &b_norm);
        [
            a[0] - dot * b_norm[0],
            a[1] - dot * b_norm[1],
            a[2] - dot * b_norm[2],
        ]
    }

    /// Helper function to sum two vectors.
    pub fn sum(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
        [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]]
    }

    pub fn sum_vec3_vec4(v1: &[f32; 3], v2: &[f32; 4]) -> [f32; 4] {
        [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2], v2[3]]
    }

    /// Helper function to subtract two vectors.
    pub fn sub(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
        [v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2]]
    }

    /// Helper function to multiply one vector by a scalar.
    pub fn mul_scalar(v: &[f32; 3], s: f32) -> [f32; 3] {
        [v[0] * s, v[1] * s, v[2] * s]
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MeshBundle {
    pub mesh: Handle<GpuMesh>,
    pub textures: Handle<TextureBundle>,
    pub materials: Handle<MaterialBundle>,
}

fn compute_tangents<T: IndexType>(
    positions: &[[f32; 3]],
    indices: &[T],
    uvs: &[[f32; 2]],
    normals: &[[f32; 3]],
    tangents: &mut [[f32; 4]],
) {
    let mut bitangents = vec![[0.0f32; 3]; positions.len()];
    let mut n_tris_per_vert = vec![0u32; positions.len()];
    for tri in indices.chunks(3) {
        let (tri0, tri1, tri2) = (tri[0].as_usize(), tri[1].as_usize(), tri[2].as_usize());
        let v0 = glam::Vec3::from(positions[tri0]);
        let v1 = glam::Vec3::from(positions[tri1]);
        let v2 = glam::Vec3::from(positions[tri2]);
        let uv0 = glam::Vec2::from(uvs[tri0]);
        let uv1 = glam::Vec2::from(uvs[tri1]);
        let uv2 = glam::Vec2::from(uvs[tri2]);

        // Calculate the edges of the triangle
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        // This will give us a direction to calculate the
        // tangent and bitangent
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        // Solving the following system of equations
        //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = ((e1 * delta_uv2.y - e2 * delta_uv1.y) * r).to_array();
        let bitangent = ((-e1 * delta_uv2.x + e2 * delta_uv1.x) * r).to_array();
        tangents[tri0] = util::sum_vec3_vec4(&tangent, &tangents[tri0]);
        tangents[tri1] = util::sum_vec3_vec4(&tangent, &tangents[tri1]);
        tangents[tri2] = util::sum_vec3_vec4(&tangent, &tangents[tri2]);
        bitangents[tri0] = util::sum(&bitangent, &bitangents[tri0]);
        bitangents[tri1] = util::sum(&bitangent, &bitangents[tri1]);
        bitangents[tri2] = util::sum(&bitangent, &bitangents[tri2]);
        n_tris_per_vert[tri0] += 1;
        n_tris_per_vert[tri1] += 1;
        n_tris_per_vert[tri2] += 1;
    }

    // Average the tangents and bitangents
    for i in 0..positions.len() {
        let denom = 1.0f32 / n_tris_per_vert[i] as f32;
        let tangent = [tangents[i][0], tangents[i][1], tangents[i][2]];
        let average = util::normalize(&util::mul_scalar(&tangent, denom));
        tangents[i][0] = average[0];
        tangents[i][1] = average[1];
        tangents[i][2] = average[2];
    }

    for i in 0..positions.len() {
        let tangent = tangents[i];
        let t = [tangent[0], tangent[1], tangent[2]];
        let b = bitangents[i];
        let n = normals[i];
        let t_perp = util::normalize(&util::rejection(&t, &n));
        tangents[i][0] = t_perp[0];
        tangents[i][1] = t_perp[1];
        tangents[i][2] = t_perp[2];
        tangents[i][3] = if util::dot(&util::cross(&t, &b), &n) > 0.0 {
            1.0
        } else {
            -1.0
        };
    }
}
