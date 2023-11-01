use crate::scene::transform::Transform;

/// A reference to a node in the scene graph.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeRef(u32);

/// A node in the scene graph.
#[derive(Debug, Default, PartialEq)]
pub struct Node {
    parent: NodeRef,
    local: Transform,
}
