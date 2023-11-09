mod node;
pub use node::*;

use crossbeam_channel::{Receiver, Sender};
use glam::Quat;
use std::fmt::{Debug, Formatter};

use crate::{
    app::command::Command,
    core::{
        assets::{MaterialAssets, MeshAssets},
        camera::{Camera, Projection},
        mesh::Mesh,
        Color, ConcatOrder,
    },
};
use legion::{storage::IntoComponentSource, World};

/// Entity in a scene.
#[derive(Clone, Copy, Debug)]
pub struct Entity {
    /// The entity ID.
    pub(crate) entity: legion::Entity,
    /// The node ID.
    pub(crate) node: NodeIdx,
}

/// Entity with a command sender.
#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct PyEntity {
    pub entity: Entity,
    pub cmd_sender: Sender<Command>,
}

/// Scene graph.
pub struct Scene {
    pub(crate) world: World,
    pub(crate) nodes: Nodes,
    pub(crate) meshes: MeshAssets,
    pub(crate) materials: MaterialAssets,
    /// Command sender for sending commands to the scene.
    pub(crate) cmd_sender: Sender<Command>,
    /// Command receiver serves as a buffer for commands to be executed.
    cmd_receiver: Receiver<Command>,
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
    pub fn new(device: &wgpu::Device) -> Self {
        let mesh_assets = MeshAssets::new(device);
        let material_assets = MaterialAssets::new(device);
        let (sender, receiver) = crossbeam_channel::unbounded::<Command>();
        Self {
            world: World::default(),
            nodes: Nodes::default(),
            meshes: mesh_assets,
            materials: material_assets,
            cmd_sender: sender,
            cmd_receiver: receiver,
        }
    }

    /// Spawns a new entity with the given components together with a new node.
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
        let node_id = self.nodes.push(Node::new(Some(parent)));

        // Add the node ID as a component to the entity.
        self.world.entry(entity).unwrap().add_component(node_id);

        Entity {
            entity,
            node: node_id,
        }
    }

    // TODO: avoid adding the mesh multiple times
    pub fn spawn_mesh(
        &mut self,
        parent: NodeIdx,
        mesh: &Mesh,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Entity {
        let mesh_handle = self.meshes.add(device, queue, mesh);
        self.spawn(parent, (mesh_handle,))
    }

    /// Processes all commands in the command receiver.
    pub fn process_commands(&mut self) {
        while let Ok(cmd) = self.cmd_receiver.try_recv() {
            match cmd {
                // Command::AddNode(parent) => {
                //     let node_id = self.nodes.push(Node::new(parent));
                //     self.world.entry(node_id).unwrap().add_component(node_id);
                // }
                // Command::RemoveNode(node) => {
                //     self.nodes.remove(node);
                //     self.world
                //         .entry(node)
                //         .unwrap()
                //         .remove_component::<NodeIdx>();
                // }
                // Command::AddComponent(entity, component) => {
                //     self.world.entry(entity).unwrap().add_component(component);
                // }
                // Command::RemoveComponent(entity, component) => {
                //     self.world
                //         .entry(entity)
                //         .unwrap()
                //         .remove_component::<NodeIdx>();
                // }
                Command::Translate {
                    entity,
                    translation,
                    order,
                } => {
                    let node = &mut self.nodes[entity.node];
                    match order {
                        ConcatOrder::Pre => {
                            node.transform_mut()
                                .pre_concat(&Transform::from_translation(translation));
                        }
                        ConcatOrder::Post => {
                            node.transform_mut()
                                .post_concat(&Transform::from_translation(translation));
                        }
                    }
                }
                Command::Rotate {
                    entity,
                    rotation,
                    order,
                } => {
                    let node = &mut self.nodes[entity.node];
                    match order {
                        ConcatOrder::Pre => {
                            node.transform_mut()
                                .pre_concat(&Transform::from_rotation(rotation));
                        }
                        ConcatOrder::Post => {
                            node.transform_mut()
                                .post_concat(&Transform::from_rotation(rotation));
                        }
                    }
                }
                Command::Scale {
                    entity,
                    scale,
                    order,
                } => {
                    let node = &mut self.nodes[entity.node];
                    match order {
                        ConcatOrder::Pre => {
                            node.transform_mut()
                                .pre_concat(&Transform::from_scale(scale));
                        }
                        ConcatOrder::Post => {
                            node.transform_mut()
                                .post_concat(&Transform::from_scale(scale));
                        }
                    }
                }
                Command::SetActive { entity, active } => {
                    self.nodes[entity.node].set_active(active);
                }
                Command::CameraOrbit {
                    entity,
                    rotation_x,
                    rotation_y,
                } => {
                    let node = &mut self.nodes[entity.node];
                    let x = node.transform().to_mat4().x_axis;
                    let y = node.transform().to_mat4().y_axis;
                    node.transform_mut().pre_concat(&Transform::from_rotation(
                        Quat::from_axis_angle(y.truncate(), rotation_y)
                            * Quat::from_axis_angle(x.truncate(), rotation_x),
                    ));
                }
            }
        }
    }

    pub fn node(&self, node: NodeIdx) -> &Node {
        &self.nodes[node]
    }

    pub fn node_mut(&mut self, node: NodeIdx) -> &mut Node {
        &mut self.nodes[node]
    }

    pub fn children(&self, node: NodeIdx) -> impl Iterator<Item = NodeIdx> + '_ {
        self.nodes.children(node)
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
