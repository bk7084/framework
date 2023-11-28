use bytemuck::Pod;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct AttribContainer {
    pub(crate) data: Vec<u8>,
    pub(crate) n_bytes: usize,
}

impl AttribContainer {
    /// Creates a new attribute container by copying the given data.
    pub fn new<T: 'static + Pod>(data: &[T]) -> Self {
        let n_bytes = data.len() * std::mem::size_of::<T>();
        Self {
            data: bytemuck::cast_slice(data).to_vec(),
            n_bytes,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn n_bytes(&self) -> usize {
        self.n_bytes
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_slice<T: 'static + Pod>(&self) -> &[T] {
        bytemuck::cast_slice(&self.data)
    }

    pub fn as_slice_mut<T: 'static + Pod>(&mut self) -> &mut [T] {
        bytemuck::cast_slice_mut(&mut self.data)
    }
}

/// A vertex attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexAttribute {
    /// Name of the vertex attribute.
    pub name: &'static str,
    /// Format of the vertex attribute.
    pub format: wgpu::VertexFormat,
    /// Index of the vertex attribute in the shader.
    pub shader_location: u32,
    /// Size of the vertex attribute in bytes.
    pub size: usize,
}

impl PartialOrd for VertexAttribute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.shader_location.partial_cmp(&other.shader_location)
    }
}

impl Ord for VertexAttribute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.shader_location.cmp(&other.shader_location)
    }
}

impl VertexAttribute {
    /// Position attribute.
    pub const POSITION: Self = Self::new(
        "vertex_position",
        wgpu::VertexFormat::Float32x3,
        0,
        std::mem::size_of::<[f32; 3]>(),
    );
    /// Normal attribute.
    pub const NORMAL: Self = Self::new(
        "vertex_normal",
        wgpu::VertexFormat::Float32x3,
        1,
        std::mem::size_of::<[f32; 3]>(),
    );
    /// UV attribute.
    pub const UV: Self = Self::new(
        "vertex_uv0",
        wgpu::VertexFormat::Float32x2,
        2,
        std::mem::size_of::<[f32; 2]>(),
    );
    /// Tangent attribute.
    pub const TANGENT: Self = Self::new(
        "vertex_tangent",
        wgpu::VertexFormat::Float32x4,
        3,
        std::mem::size_of::<[f32; 4]>(),
    );
    /// Color attribute.
    pub const COLOR: Self = Self::new(
        "vertex_color",
        wgpu::VertexFormat::Float32x4,
        4,
        std::mem::size_of::<[f32; 4]>(),
    );

    pub const fn new(
        name: &'static str,
        format: wgpu::VertexFormat,
        shader_location: u32,
        size: usize,
    ) -> Self {
        Self {
            name,
            format,
            shader_location,
            size,
        }
    }
}

/// A collection of vertex attributes.
#[derive(Clone, Default)]
pub struct VertexAttributes(pub(crate) BTreeMap<VertexAttribute, AttribContainer>);

impl VertexAttributes {
    pub fn insert(&mut self, attrib: VertexAttribute, data: AttribContainer) {
        self.0.insert(attrib, data);
    }

    /// Returns the range of the vertex attribute in the mesh data buffer, if it
    /// exists.
    pub fn vertex_count(&self) -> usize {
        self.0
            .get(&VertexAttribute::POSITION)
            .map(|a| a.len())
            .unwrap_or(0)
    }
}
