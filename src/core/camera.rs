use crate::core::Color;
use glam::Mat4;
use pyo3::prelude::*;
use std::{fmt::Debug, ops::Range};

/// The type of projection for a camera.
#[pyclass]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProjectionKind {
    /// An orthographic projection.
    Orthographic,
    /// A perspective projection.
    Perspective,
}

/// A projection for a camera.
#[pyclass]
#[derive(Clone, Copy)]
pub struct Projection {
    /// The type of projection.
    pub kind: ProjectionKind,
    /// The vertical field of view or vertical extent of this projection.
    pub fov_or_ext: VerticalFovOrExtent,
}

impl Debug for Projection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ProjectionKind::Orthographic => f
                .debug_struct("Projection")
                .field("kind", &self.kind)
                .field("ext", &unsafe { self.fov_or_ext.extent })
                .finish(),
            ProjectionKind::Perspective => f
                .debug_struct("Projection")
                .field("kind", &self.kind)
                .field("fov", &unsafe { self.fov_or_ext.fov })
                .finish(),
        }
    }
}

impl Projection {
    /// Returns the projection matrix for this projection.
    pub fn matrix(&self, near: f32, far: f32, aspect: f32) -> Mat4 {
        match self.kind {
            ProjectionKind::Orthographic => {
                let extent_v = unsafe { self.fov_or_ext.extent };
                let half_extent_v = extent_v * 0.5;
                Mat4::orthographic_rh(
                    -half_extent_v,
                    half_extent_v,
                    -half_extent_v,
                    half_extent_v,
                    near,
                    far,
                )
            }
            ProjectionKind::Perspective => {
                let fov_v = unsafe { self.fov_or_ext.fov }.to_radians();
                if far == f32::INFINITY {
                    Mat4::perspective_infinite_rh(fov_v, aspect, near)
                } else {
                    Mat4::perspective_rh(fov_v, aspect, near, far)
                }
            }
        }
    }
}

#[pymethods]
impl Projection {
    /// Creates a new perspective projection.
    #[staticmethod]
    pub fn orthographic(height: f32) -> Self {
        Self {
            kind: ProjectionKind::Orthographic,
            fov_or_ext: VerticalFovOrExtent { extent: height },
        }
    }

    /// Creates a new perspective projection.
    #[staticmethod]
    pub fn perspective(fov: f32) -> Self {
        Self {
            kind: ProjectionKind::Perspective,
            fov_or_ext: VerticalFovOrExtent { fov },
        }
    }
}

/// Union of the vertical field of view and vertical extent of a camera.
#[repr(C)]
#[derive(Clone, Copy)]
pub union VerticalFovOrExtent {
    /// Vertical field of view in degrees.
    fov: f32,
    /// Vertical extent in world units.
    extent: f32,
}

/// A camera component.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// The projection settings for this camera.
    pub proj: Projection,
    /// The near depth of this camera.
    pub near: f32,
    /// The far depth of this camera.
    pub far: f32,
    /// Background color for this camera.
    pub background: Color,
}

impl Camera {
    /// Creates a new camera with the given projection settings.
    pub fn new(proj: Projection, depth: Range<f32>, background: Color) -> Self {
        Self {
            proj,
            near: depth.start,
            far: depth.end,
            background,
        }
    }

    /// Creates a new camera with the perspective projection settings.
    pub fn perspective(fov_v: f32, depth: Range<f32>, background: Color) -> Self {
        Self::new(Projection::perspective(fov_v), depth, background)
    }

    /// Creates a new camera with the orthographic projection settings.
    pub fn orthographic(height: f32, depth: Range<f32>, background: Color) -> Self {
        Self::new(Projection::orthographic(height), depth, background)
    }

    /// Returns the projection matrix for this camera.
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        self.proj.matrix(self.near, self.far, aspect)
    }
}
