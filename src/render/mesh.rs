use pyo3::{pyclass, pymethods};
use std::{
    any::Any,
    collections::BTreeMap,
    fmt::{Debug, Formatter},
    ops::Deref,
    sync::Arc,
};

type Vec2 = [f32; 2];
type Vec3 = [f32; 3];
type Vec4 = [f32; 4];

pub trait AttribContainer {
    /// Number of elements in the container.
    fn len(&self) -> usize;
    /// Whether the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Convert the container to a byte slice.
    fn as_bytes(&self) -> &[u8];
    /// Reference to the container as an `Any`.
    fn as_any(&self) -> &dyn Any;
    /// Mutable reference to the container as an `Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static + bytemuck::Pod> AttribContainer for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    /// Name of the vertex attribute.
    pub name: &'static str,
    /// Format of the vertex attribute.
    pub format: wgpu::VertexFormat,
    /// Index of the vertex attribute in the shader.
    pub shader_location: u32,
}

impl VertexAttribute {
    /// Position attribute.
    pub const POSITION: Self = Self::new("vertex_position", wgpu::VertexFormat::Float32x3, 0);
    /// Normal attribute.
    pub const NORMAL: Self = Self::new("vertex_normal", wgpu::VertexFormat::Float32x3, 1);
    /// UV attribute.
    pub const UV0: Self = Self::new("vertex_uv0", wgpu::VertexFormat::Float32x2, 2);
    /// UV attribute.
    pub const UV1: Self = Self::new("vertex_uv1", wgpu::VertexFormat::Float32x2, 3);
    /// Tangent attribute.
    pub const TANGENT: Self = Self::new("vertex_tangent", wgpu::VertexFormat::Float32x4, 4);
    /// Color attribute.
    pub const COLOR: Self = Self::new("vertex_color", wgpu::VertexFormat::Float32x4, 5);

    pub const fn new(name: &'static str, format: wgpu::VertexFormat, shader_location: u32) -> Self {
        Self {
            name,
            format,
            shader_location,
        }
    }
}

type VertexAttributes = BTreeMap<VertexAttribute, Arc<dyn AttribContainer + Send + Sync + 'static>>;

/// Topology of a mesh primitive.
#[pyo3::pyclass]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct PyTopology(wgpu::PrimitiveTopology);

impl Deref for PyTopology {
    type Target = wgpu::PrimitiveTopology;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<wgpu::PrimitiveTopology> for PyTopology {
    fn from(topology: wgpu::PrimitiveTopology) -> Self {
        Self(topology)
    }
}

impl From<PyTopology> for wgpu::PrimitiveTopology {
    fn from(topology: PyTopology) -> Self {
        topology.0
    }
}

/// Indices of a mesh.
#[derive(Clone)]
pub enum Indices {
    U32(Vec<u32>),
    U16(Vec<u16>),
}

#[derive(Clone)]
#[pyclass]
pub struct Mesh {
    /// Topology of the mesh primitive.
    topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    attributes: VertexAttributes,
    /// Indices of the mesh.
    indices: Option<Indices>,
}

impl Mesh {
    pub const ATTRIB_POSITION: &'static str = "position";
}

#[pymethods]
impl Mesh {
    #[new]
    pub fn new(topology: PyTopology) -> Self {
        let mut attributes: VertexAttributes = BTreeMap::new();
        // vert_attribs.insert("position".to_string(), Arc::new(Vec::<Vec3>::new()));
        // vert_attribs.insert("normal".to_string(), Arc::new(Vec::<Vec3>::new()));
        // vert_attribs.insert("uv0".to_string(), Arc::new(Vec::<Vec2>::new()));
        // vert_attribs.insert("uv1".to_string(), Arc::new(Vec::<Vec2>::new()));
        // vert_attribs.insert("color0".to_string(), Arc::new(Vec::<Vec4>::new()));
        // vert_attribs.insert("color1".to_string(), Arc::new(Vec::<Vec4>::new()));
        // vert_attribs.insert("tangent".to_string(), Arc::new(Vec::<Vec3>::new()));
        // vert_attribs.insert("bitangent".to_string(), Arc::new(Vec::<Vec3>::new()));
        Self {
            topology: topology.into(),
            attributes,
            indices: None,
        }
    }

    // /// Validate that all vertex attributes have the same length.
    // pub fn validate(&self) -> Result<(), >

    pub fn compute_normals(&mut self) {}
    pub fn compute_tangents(&mut self) {}
}

// TODO: preallocate a giant buffer and suballocate from it
pub struct GpuMesh {
    pub topology: wgpu::PrimitiveTopology,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: Option<IndexBuffer>,
}

pub struct IndexBuffer {
    pub buffer: wgpu::Buffer,
    pub format: wgpu::IndexFormat,
    pub count: u32,
}

pub struct VertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}
