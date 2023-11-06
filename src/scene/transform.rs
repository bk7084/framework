#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: glam::Vec3,
    pub scale: f32,
    pub orientation: glam::Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: 1.0,
            orientation: glam::Quat::IDENTITY,
        }
    }
}

impl Transform {
    /// Identity transform.
    pub fn identity() -> Self {
        Self::default()
    }

    /// Returns the inverse of this transform.
    pub fn inverse(&self) -> Self {
        let scale = 1.0 / self.scale;
        let orientation = self.orientation.inverse();
        let position = -scale * (orientation * self.position);
        Self {
            position,
            scale,
            orientation,
        }
    }

    /// Returns the matrix representation of this transform.
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(self.scale),
            self.orientation,
            self.position,
        )
    }
}
