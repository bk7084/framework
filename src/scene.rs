mod node;
mod transform;

use node::*;

use legion::{Resources, World};

#[derive(Debug)]
pub struct Scene {
    pub world: World,
    pub nodes: Vec<Node>,
    // resources: Resources,
    // systems: Schedule,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            nodes: vec![Node::default()],
        }
    }
}
