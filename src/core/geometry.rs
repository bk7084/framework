use crate::core::mesh::Mesh;

/// Helper type for creating shapes.
pub struct Geometry(pub(crate) Mesh);

impl Geometry {
    pub fn cube() -> Self {
        Self(Mesh::cube())
    }
}
