pub use crate::scene::transform::Transform;

use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A node in the scene graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    /// The index of the parent node in the nodes array, if any.
    pub parent: Option<NodeIdx>,
    /// The local transform of this node.
    local: Transform,
}

impl Node {
    pub fn new(parent: Option<NodeIdx>) -> Self {
        Self {
            parent,
            local: Transform::identity(),
        }
    }

    /// Constructs the root node.
    pub fn root() -> Self {
        Self {
            parent: None,
            local: Transform::identity(),
        }
    }

    /// Returns the local transform of this node.
    pub fn transform(&self) -> &Transform {
        &self.local
    }

    /// Returns the local transform of this node.
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.local
    }
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

/// Container for all nodes in the scene graph.
#[derive(Clone, Debug)]
pub struct Nodes(Vec<Node>);

impl Nodes {
    /// Constructs a new empty scene graph with only the root node.
    pub fn new() -> Self {
        Self(vec![Node::root()])
    }

    /// Returns the world transform of this node.
    pub fn world(&self, node: NodeIdx) -> Transform {
        match self[node].parent {
            Some(parent) => self.world(parent) * self.0[node].local,
            None => self[node].local,
        }
    }

    /// Returns the inverse world transform of this node.
    pub fn inverse_world(&self, node: NodeIdx) -> Transform {
        self.world(node).inverse()
    }

    /// Pushes a new node to the scene graph and returns its ID.
    pub fn push(&mut self, node: Node) -> NodeIdx {
        let idx = NodeIdx(self.0.len());
        self.0.push(node);
        idx
    }
}

impl Default for Nodes {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Nodes {
    type Target = [Node];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Nodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<NodeIdx> for Nodes {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl Index<NodeIdx> for &Nodes {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl Index<NodeIdx> for &mut Nodes {
    type Output = Node;

    fn index(&self, index: NodeIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<NodeIdx> for Nodes {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl IndexMut<NodeIdx> for &mut Nodes {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}
