use crate::scene::transform::Transform;
use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A node in the scene graph.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Node {
    /// The index of the parent node in the nodes array, if any.
    pub parent: Option<NodeIdx>,
    /// The local transform of this node.
    pub local: Transform,
}

impl Node {
    /// Constructs the root node.
    pub fn root() -> Self {
        Self {
            parent: None,
            local: Transform::identity(),
        }
    }

    // /// Returns the world transform of this node.
    // pub fn world(&self, nodes: &[Node]) -> Transform {
    //     match self.parent {
    //         Some(parent) => nodes[parent].world(nodes) * self.local,
    //         None => self.local,
    //     }
    // }
    //
    // /// Returns the inverse world transform of this node.
    // pub fn inverse_world(&self, nodes: &[Node]) -> Transform {
    //     match self.parent {
    //         Some(parent) => self.local.inverse() *
    // nodes[parent].inverse_world(nodes),         None => self.local.inverse(),
    //     }
    // }
}

/// The ID of a node in the scene graph.
///
/// This is a simple wrapper around a `usize` that is used to identify nodes in
/// the scene graph, it is the index of the node in the nodes array. The root
/// node has the ID `0`, see [`NodeIdx::root`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct NodeIdx(pub(crate) usize);

impl NodeIdx {
    /// Returns the root node ID.
    pub const fn root() -> Self {
        Self(0)
    }
}

impl Deref for NodeIdx {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeIdx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<NodeIdx> for Vec<Node> {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self[index.0]
    }
}

impl Index<NodeIdx> for &[Node] {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self[index]
    }
}

impl Index<NodeIdx> for &mut [Node] {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self[index]
    }
}

impl IndexMut<NodeIdx> for Vec<Node> {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl IndexMut<NodeIdx> for &mut [Node] {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        &mut self[index]
    }
}
