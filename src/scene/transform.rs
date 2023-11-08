use std::ops::Mul;

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

    /// Combines two transforms. The result is equivalent to applying `self` and
    /// then `other`.
    pub fn _mul(&self, other: &Self) -> Self {
        Self {
            scale: self.scale * other.scale,
            orientation: self.orientation * other.orientation,
            position: self.scale * (self.orientation * other.position) + self.position,
        }
    }
}

impl Mul<Transform> for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self._mul(&rhs)
    }
}

impl Mul<Transform> for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        self._mul(&rhs)
    }
}

impl Mul<&Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: &Transform) -> Self::Output {
        self._mul(rhs)
    }
}

impl Mul for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        self._mul(rhs)
    }
}
