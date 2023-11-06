mod node;
pub use node::*;

mod transform;

use std::fmt::{Debug, Formatter};

use legion::{storage::IntoComponentSource, Entity, EntityStore, Resources, World};

#[pyo3::pyclass]
#[derive(Clone, Copy, Debug)]
pub struct PyEntity(Entity);

pub struct Scene {
    world: World,
    nodes: Vec<Node>,
    resources: Resources,
    // systems: Schedule,
}

impl Debug for Scene {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("world", &self.world)
            .field("nodes", &self.nodes)
            .finish()
    }
}

impl Scene {
    /// Creates a new empty scene.
    pub fn new() -> Self {
        Self {
            world: World::default(),
            nodes: vec![Node::root()],
            resources: Default::default(),
        }
    }

    /// Spawns a new entity with the given components.
    ///
    /// Together with the entity, a new node will be created and added to the
    /// scene graph. The entity will be parented to the given parent node. If
    /// the parent node is `None`, the entity will be parented to the root
    /// node.
    pub fn spawn<T>(&mut self, parent: NodeIdx, components: T) -> Entity
    where
        Option<T>: IntoComponentSource,
    {
        // Check if the parent node exists.
        assert!(
            parent.0 < self.nodes.len(),
            "Spawning entity with invalid parent node, parent node does not exist!"
        );

        // Spawn the entity and add the components to it.
        let entity = self.world.spawn(components);

        // Add a new node to the scene graph.
        let node_id = NodeIdx(self.nodes.len());
        self.nodes.push(Node {
            parent: Some(parent),
            ..Default::default()
        });

        // Add the node ID as a component to the entity.
        self.world.entry(entity).unwrap().add_component(node_id);
        entity
    }
}

mod tests {
    use crate::scene::node::NodeIdx;

    #[test]
    fn entity_spawning() {
        let mut scene = super::Scene::new();
        let entity = scene.spawn(NodeIdx::root(), ());
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[1].parent, Some(NodeIdx::root()));

        let entity1 = scene.spawn(NodeIdx(1), ());
        assert_eq!(scene.nodes.len(), 3);
        assert_eq!(scene.nodes[2].parent, Some(NodeIdx(1)));

        let entity2 = scene.spawn(NodeIdx(1), ());
        assert_eq!(scene.nodes.len(), 4);
        assert_eq!(scene.nodes[3].parent, Some(NodeIdx(1)));

        let entity3 = scene.spawn(NodeIdx::root(), ());
        assert_eq!(scene.nodes.len(), 5);
        assert_eq!(scene.nodes[4].parent, Some(NodeIdx::root()));
    }

    #[test]
    #[should_panic]
    fn entity_spawning_failed() {
        let mut scene = super::Scene::new();
        let _ = scene.spawn(NodeIdx(1), ());
    }
}
