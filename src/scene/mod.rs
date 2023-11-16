mod node;
pub use node::*;

use crossbeam_channel::Sender;
use glam::{Mat4, Quat, Vec3};
use std::fmt::{Debug, Formatter};

use crate::{
    app::command::{Command, CommandReceiver, CommandSender},
    core::ConcatOrder,
};
use legion::{storage::IntoComponentSource, World};
use numpy as np;
use pyo3::Python;

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

#[pyo3::pymethods]
impl PyEntity {
    pub fn draw(&self) {
        self.cmd_sender
            .send(Command::SetVisible {
                entity: self.entity,
                visible: true,
            })
            .unwrap();
    }

    pub fn set_visible(&self, visible: bool) {
        self.cmd_sender
            .send(Command::SetVisible {
                entity: self.entity,
                visible,
            })
            .unwrap();
    }

    pub fn set_transform(&self, mat4: &np::PyArray2<f32>) {
        Python::with_gil(|_py| {
            let mat = Mat4::from_cols_slice(mat4.readonly().as_slice().unwrap()).transpose();
            let (scale, rotation, translation) = mat.to_scale_rotation_translation();
            self.cmd_sender
                .send(Command::SetTransform {
                    entity: self.entity,
                    translation,
                    rotation,
                    scale,
                })
                .unwrap();
        });
    }
}

/// Scene graph.
pub struct Scene {
    /// Legion world for storing entities and components.
    pub(crate) world: World,
    /// Scene graph nodes.
    pub(crate) nodes: Nodes,
    /// Command sender for sending commands to the scene.
    cmd_sender: CommandSender,
    /// Command receiver serves as a buffer for commands to be executed.
    cmd_receiver: CommandReceiver,
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
    // TODO: separate GPU resources from scene graph
    /// Creates a new empty scene.
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Command>();
        Self {
            world: World::default(),
            nodes: Nodes::default(),
            cmd_sender: sender,
            cmd_receiver: receiver,
        }
    }

    /// Returns the command sender.
    pub fn cmd_sender(&self) -> &CommandSender {
        &self.cmd_sender
    }

    /// Returns the command receiver.
    pub fn cmd_receiver(&self) -> &CommandReceiver {
        &self.cmd_receiver
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
                Command::SetVisible { entity, visible } => {
                    self.nodes[entity.node].set_visible(visible);
                }
                Command::CameraOrbit {
                    entity,
                    rotation_x,
                    rotation_y,
                } => {
                    let node = &mut self.nodes[entity.node];
                    let x = node.transform().to_mat4().x_axis;
                    node.transform_mut().pre_concat(&Transform::from_rotation(
                        Quat::from_axis_angle(Vec3::Y, rotation_y)
                            * Quat::from_axis_angle(x.truncate(), rotation_x),
                    ));
                }
                Command::SetTransform {
                    entity,
                    translation,
                    rotation,
                    scale,
                } => {
                    let node = &mut self.nodes[entity.node];
                    node.transform_mut().translation = translation;
                    node.transform_mut().rotation = rotation;
                    node.transform_mut().scale = scale;
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
    #[test]
    fn entity_spawning() {
        use super::NodeIdx;

        let mut scene = super::Scene::new();
        let _entity = scene.spawn(NodeIdx::root(), ());
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[NodeIdx(1)].parent, Some(NodeIdx::root()));

        let _entity1 = scene.spawn(NodeIdx(1), ());
        assert_eq!(scene.nodes.len(), 3);
        assert_eq!(scene.nodes[NodeIdx(2)].parent, Some(NodeIdx(1)));

        let _entity2 = scene.spawn(NodeIdx(1), ());
        assert_eq!(scene.nodes.len(), 4);
        assert_eq!(scene.nodes[NodeIdx(3)].parent, Some(NodeIdx(1)));

        let _entity3 = scene.spawn(NodeIdx::root(), ());
        assert_eq!(scene.nodes.len(), 5);
        assert_eq!(scene.nodes[NodeIdx(4)].parent, Some(NodeIdx::root()));
    }

    #[test]
    #[should_panic]
    fn entity_spawning_failed() {
        let mut scene = super::Scene::new();
        let _ = scene.spawn(super::NodeIdx(1), ());
    }
}
