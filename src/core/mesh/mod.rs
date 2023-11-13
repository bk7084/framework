use rustc_hash::{FxHashMap, FxHashSet};
use std::{fmt::Debug, ops::Range, path::PathBuf, sync::atomic::AtomicU64};

mod attribute;
use crate::{
    app::command::{Command, CommandSender},
    core::{
        assets::{Asset, Handle},
        Alignment, ArrVec, Material,
    },
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

/// Mesh id counter. 0 and 1 are reserved for the default cube and quad.
static MESH_ID: AtomicU64 = AtomicU64::new(2);

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

    /// Computes per vertex normals for the mesh.
    pub fn compute_per_vertex_normals(&mut self) {
        // If the mesh does not have positions, return.
        if !self.attributes.0.contains_key(&VertexAttribute::POSITION) {
            log::error!("Mesh does not have positions.");
            return;
        }

        // If the mesh already has normals, return.
        if self.attributes.0.contains_key(&VertexAttribute::NORMAL) {
            log::warn!("Mesh already has normals. Skipping normal computation.");
            return;
        }

        // Get the positions.
        let positions = self
            .attributes
            .0
            .get(&VertexAttribute::POSITION)
            .unwrap()
            .as_slice::<[f32; 3]>();

        // Get the indices.
        if self.indices.is_none() {
            log::error!("Cannot compute normals without indices.");
            return;
        }

        // let indices = match &self.indices {
        //     Some(Indices::U32(indices)) => indices.as_slice(),
        //     Some(Indices::U16(indices)) => indices.as_slice(),
        //     None => unreachable!("Cannot compute normals without indices."),
        // };
        for pos in positions.iter() {
            log::info!("{:?}", pos);
        }

        // Compute the normals.
        // let mut normals = Vec::with_capacity(positions.len());
        todo!("Compute normals.");
    }

    pub fn compute_tangents(&mut self) {}
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

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        Mesh {
            id: 0,
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U16(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        }
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
        // Vertex indices for a unit cube centered at the origin.
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        Mesh {
            id: 1,
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U16(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        }
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
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
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
                indices.push(next_ring * (segments + 1) + segment);
                indices.push(next_ring * (segments + 1) + next_segment);

                indices.push(ring * (segments + 1) + segment);
                indices.push(next_ring * (segments + 1) + next_segment);
                indices.push(ring * (segments + 1) + next_segment);
            }
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV0, AttribContainer::new(&uvs));

        Mesh {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U32(indices)),
            sub_meshes: None,
            path: None,
            materials: None,
        }
    }

    /// Loads a mesh from a wavefront obj file.
    pub fn load_from_obj(path: &str) -> Self {
        log::debug!("Loading mesh from {}.", path);
        let options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };
        let (models, materials) = tobj::load_obj(path, &options).unwrap();
        let materials = materials.expect("Failed to load materials.");
        log::debug!("- Loaded {} models.", models.len());
        log::debug!("- Loaded {} materials.", materials.len());
        let mut attributes = VertexAttributes::default();
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
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
                for index in mesh_indices.iter_mut() {
                    *index += index_offset;
                }
                vertices.append(&mut mesh.positions.clone());
                normals.append(&mut mesh.normals.clone());
                uvs.append(&mut mesh.texcoords.clone());
                indices.append(&mut mesh_indices);
                index_start += mesh_indices.len() as u32;
            }
            sub_mesh.range.end = index_start;
            sub_meshes.push(sub_mesh);
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));

        if !normals.is_empty() {
            attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        }

        if !uvs.is_empty() {
            attributes.insert(VertexAttribute::UV0, AttribContainer::new(&uvs));
        }

        Mesh {
            id: MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: if !indices.is_empty() {
                Some(Indices::U32(indices))
            } else {
                None
            },
            sub_meshes: Some(sub_meshes),
            path: None,
            materials: Some(
                materials
                    .iter()
                    .map(|m| Material::from_tobj_material(m.clone(), path.as_ref()))
                    .collect(),
            ),
        }
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

mod icosphere {
    // Helper function to calculate the midpoint of two vertices
    pub fn midpoint(v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
        [
            (v1[0] + v2[0]) / 2.0,
            (v1[1] + v2[1]) / 2.0,
            (v1[2] + v2[2]) / 2.0,
        ]
    }

    // Helper function to normalize a vector
    pub fn normalize(v: [f32; 3]) -> [f32; 3] {
        let length = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
        [v[0] / length, v[1] / length, v[2] / length]
    }

    // Helper function to calculate UV coordinates based on a vector
    pub fn uv_coordinates(v: [f32; 3]) -> [f32; 2] {
        let u = 0.5 + (v[2].atan2(v[0]) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (v[1].asin() / std::f32::consts::PI);
        [u, v]
    }
}
