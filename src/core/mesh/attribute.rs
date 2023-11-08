use std::{
    any::Any,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub trait AttribContainer {
    /// Number of elements in the container.
    fn len(&self) -> usize;
    /// Whether the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Number of bytes in the container.
    fn n_bytes(&self) -> usize;
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

    fn n_bytes(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
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

impl<T: 'static + bytemuck::Pod, const N: usize> AttribContainer for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn n_bytes(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
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

/// A vertex attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexAttribute {
    /// Name of the vertex attribute.
    pub name: &'static str,
    /// Format of the vertex attribute.
    pub format: wgpu::VertexFormat,
    /// Index of the vertex attribute in the shader.
    pub shader_location: u32,
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

/// A collection of vertex attributes.
#[derive(Clone, Default)]
pub struct VertexAttributes(
    pub(crate) BTreeMap<VertexAttribute, Arc<dyn AttribContainer + Send + Sync + 'static>>,
);

impl VertexAttributes {
    pub fn insert(
        &mut self,
        attrib: VertexAttribute,
        data: Arc<dyn AttribContainer + Send + Sync + 'static>,
    ) {
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
