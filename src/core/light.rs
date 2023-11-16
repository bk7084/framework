use crate::core::Color;
use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum Light {
    /// A directional light.
    Directional {
        /// The direction from which the light is coming.
        /// But the shader will use the opposite direction.
        direction: Vec3,
        color: Color,
    },
    Point {
        color: Color,
    },
}
