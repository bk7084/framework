use pyo3::{pyclass, pymethods};
use std::{
    any::Any,
    collections::BTreeMap,
    fmt::{Debug, Formatter},
    ops::{Deref, Range},
    sync::Arc,
};

mod attribute;
use crate::core::assets::{storage::GpuMeshStorage, Asset};
pub use attribute::*;

type Vec2 = [f32; 2];
type Vec3 = [f32; 3];
type Vec4 = [f32; 4];

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
            _ => unreachable!(),
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
#[derive(Clone)]
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

#[pyclass]
#[derive(Clone)]
pub struct Mesh {
    /// Topology of the mesh primitive.
    pub(crate) topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    pub(crate) attributes: VertexAttributes,
    /// Indices of the mesh.
    pub(crate) indices: Option<Indices>,
}

#[pymethods]
impl Mesh {
    #[new]
    pub fn new(topology: PyTopology) -> Self {
        let mut attributes = VertexAttributes::default();
        Self {
            topology: topology.into(),
            attributes,
            indices: None,
        }
    }

    #[staticmethod]
    #[pyo3(name = "create_cube")]
    pub fn new_cube_py() -> Self {
        Self::cube()
    }

    pub fn compute_normals(&mut self) {}
    pub fn compute_tangents(&mut self) {}
}

pub struct VertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

impl Mesh {
    /// Creates a unit cube of side length 1 centered at the origin.
    pub fn cube() -> Self {
        let mut attributes = VertexAttributes::default();
        // Vertex positions for a unit cube centered at the origin.
        let vertices = [
            // [0.0f32, 0.5, 0.0],
            // [-0.5, -0.5, 0.0],
            // [0.5, -0.5, 0.0],
            // front (0.0, 0.0, 0.5)
            [-0.5f32, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
            // back (0.0, 0.0, -0.5)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
            // right (0.5, 0.0, 0.0)
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // left (-0.5, 0.0, 0.0)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, -0.5],
            // top (0.0, 0.5, 0.0)
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            // bottom (0.0, -0.5, 0.0)
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
        ];
        // Vertex normals for a unit cube centered at the origin. Per vertex normals.
        let normals = [
            // front (0.0, 0.0, 1.0)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // back (0.0, 0.0, -1.0)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            // right (1.0, 0.0, 0.0)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // left (-1.0, 0.0, 0.0)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // top (0.0, 1.0, 0.0)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // bottom (0.0, -1.0, 0.0)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        // Vertex indices for a unit cube centered at the origin.
        let indices: Vec<u16> = vec![
            0u16, 1, 2, 2, 3, 0, // front
            4, 7, 6, 6, 5, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom */
        ];

        attributes.insert(VertexAttribute::POSITION, Arc::new(vertices));
        attributes.insert(VertexAttribute::NORMAL, Arc::new(normals));
        Mesh {
            topology: wgpu::PrimitiveTopology::TriangleList,
            attributes,
            indices: Some(Indices::U16(indices)),
        }
    }
}

/// A mesh on the GPU.
pub struct GpuMesh {
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
}

impl Asset for GpuMesh {}

impl GpuMesh {
    /// Creates a new empty gpu mesh.
    pub fn new(topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            topology,
            vertex_attribute_ranges: Vec::new(),
            vertex_count: 0,
            index_format: None,
            index_range: 0..0,
            index_count: 0,
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
