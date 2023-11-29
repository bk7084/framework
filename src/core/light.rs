use crate::core::Color;
use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum Light {
    /// A directional light.
    Directional {
        /// The direction from which the light is coming (in world space, origin
        /// - position). But the shader will use the opposite direction
        /// during shading calculations. See
        /// [`crate::render::rpass::LightsBindGroup::update_lights`].
        direction: Vec3,
        color: Color,
    },
    Point {
        color: Color,
    },
}

impl Light {
    /// Returns true if the light is a directional source.
    #[inline]
    pub const fn is_directional(&self) -> bool {
        match self {
            Light::Directional { .. } => true,
            _ => false,
        }
    }

    /// Returns true if the light is a point source.
    #[inline]
    pub const fn is_point(&self) -> bool {
        match self {
            Light::Point { .. } => true,
            _ => false,
        }
    }
}
