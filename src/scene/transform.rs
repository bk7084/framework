#[derive(Debug, Default, PartialEq, Clone, Copy)]
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

    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(self.scale),
            self.orientation,
            self.position,
        )
    }
}
