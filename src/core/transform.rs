use glam::{Affine3A, Mat4, Quat, Vec3};
use std::ops::Mul;

/// The order in which transforms are concatenated. The transformation
/// result is in the reverse order of concatenation.
#[pyo3::pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConcatOrder {
    /// The transform is concatenated before the current one.
    Pre,
    /// The transform is concatenated after the current one.
    Post,
}

/// Transform relative to the parent node or the reference frame if the node
/// has no parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
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
        let orientation = self.rotation.inverse();
        let position = -scale * (orientation * self.translation);
        Self {
            translation: position,
            scale,
            rotation: orientation,
        }
    }

    /// Returns the matrix representation of this transform.
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Creates transform from matrix.
    pub fn from_mat4(mat: Mat4) -> Self {
        let (scale, rotation, translation) = mat.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Sets the transform from a matrix.
    pub fn set_from_mat4(&mut self, mat: Mat4) {
        let (scale, rotation, translation) = mat.to_scale_rotation_translation();
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
    }

    /// Sets the translation component of the transform.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    /// Sets the rotation component of the transform.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Sets the scale component of the transform.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Look at a target position.
    pub fn looking_at(&mut self, target: Vec3, up: Vec3) {
        let affine = Affine3A::look_at_rh(self.translation, target, up);
        let (_, rot, _) = affine.inverse().to_scale_rotation_translation();
        self.rotation = rot;
    }

    /// Concatenates the transform before the current one (on the left). The
    /// result is equivalent to applying `self` and then `transform`.
    pub fn pre_concat(&mut self, transform: &Transform) {
        self.translation =
            transform.scale * (transform.rotation * self.translation) + transform.translation;
        self.rotation = transform.rotation * self.rotation;
        self.scale = transform.scale * self.scale;
    }

    /// Concatenates the transform after the current one. The result is
    /// equivalent to applying `other` and then `self`. This is the order in
    /// which transforms are concatenated not the order in which they are
    /// applied onto the object.
    pub fn post_concat(&mut self, transform: &Transform) {
        self.translation = self.scale * (self.rotation * transform.translation) + self.translation;
        self.rotation = self.rotation * transform.rotation;
        self.scale = self.scale * transform.scale;
    }

    /// Combines two transforms. The result is equivalent to applying `self` and
    /// then `other`.
    fn _mul(&self, other: &Self) -> Self {
        Self {
            scale: self.scale * other.scale,
            rotation: self.rotation * other.rotation,
            translation: self.scale * (self.rotation * other.translation) + self.translation,
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
