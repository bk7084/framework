use crate::scene::transform::Transform;

/// A node in the scene graph.
#[derive(Debug, Default)]
pub struct Node {
    /// The index of the parent node in the nodes array, if any.
    pub parent: Option<usize>,
    /// The indices of the children nodes in the nodes array.
    pub children: Vec<usize>,

    local: Transform,
}
