use crate::core::Color;
use glam::Mat4;
use std::{fmt::Debug, ops::Range};

/// The type of projection for a camera.
#[pyo3::pyclass]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub enum ProjectionKind {
    /// An orthographic projection.
    Orthographic,
    /// A perspective projection.
    #[default]
    Perspective,
}

/// Describes the projection settings for a camera.
#[pyo3::pyclass]
#[derive(Clone, Copy)]
pub struct Projection {
    /// The type of projection.
    pub kind: ProjectionKind,
    /// The vertical field of view or vertical extent of this projection.
    pub fov_or_ext: VerticalFovOrExtent,
    /// The minimum depth(near plane) of this projection.
    pub min_depth: f32,
    /// The maximum depth(far plane) of this projection.
    pub max_depth: f32,
}

impl Default for Projection {
    fn default() -> Self {
        Self {
            kind: ProjectionKind::Perspective,
            fov_or_ext: VerticalFovOrExtent { fov: 60.0 },
            min_depth: 0.1,
            max_depth: f32::INFINITY,
        }
    }
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
                .field("min_depth", &self.min_depth)
                .field("max_depth", &self.max_depth)
                .finish(),
        }
    }
}

impl Projection {
    /// Returns the projection matrix for this projection.
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        match self.kind {
            ProjectionKind::Orthographic => {
                let extent_v = unsafe { self.fov_or_ext.extent };
                let half_extent_v = extent_v * 0.5;
                Mat4::orthographic_rh(
                    -half_extent_v,
                    half_extent_v,
                    -half_extent_v,
                    half_extent_v,
                    self.min_depth,
                    self.max_depth,
                )
            }
            ProjectionKind::Perspective => {
                let fov_v = unsafe { self.fov_or_ext.fov }.to_radians();
                if self.max_depth == f32::INFINITY {
                    Mat4::perspective_infinite_rh(fov_v, aspect, self.min_depth)
                } else {
                    Mat4::perspective_rh(fov_v, aspect, self.min_depth, self.max_depth)
                }
            }
        }
    }
}

#[pyo3::pymethods]
impl Projection {
    /// Creates a new perspective projection.
    #[staticmethod]
    pub fn orthographic(height: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            kind: ProjectionKind::Orthographic,
            fov_or_ext: VerticalFovOrExtent { extent: height },
            min_depth: z_near,
            max_depth: z_far,
        }
    }

    /// Creates a new perspective projection.
    #[staticmethod]
    pub fn perspective(fov: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            kind: ProjectionKind::Perspective,
            fov_or_ext: VerticalFovOrExtent { fov },
            min_depth: z_near,
            max_depth: z_far,
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
    /// Background color for this camera.
    pub background: Color,
    /// If this camera is the main camera.
    pub is_main: bool,
}

impl Camera {
    /// Creates a new camera with the given projection settings.
    pub fn new(proj: Projection, background: Color, main: bool) -> Self {
        Self {
            proj,
            background,
            is_main: main,
        }
    }

    /// Creates a new camera with the perspective projection settings.
    pub fn perspective(fov_v: f32, depth: Range<f32>, background: Color) -> Self {
        Self::new(
            Projection::perspective(fov_v, depth.start, depth.end),
            background,
            false,
        )
    }

    /// Creates a new camera with the orthographic projection settings.
    pub fn orthographic(height: f32, depth: Range<f32>, background: Color) -> Self {
        Self::new(
            Projection::orthographic(height, depth.start, depth.end),
            background,
            false,
        )
    }

    /// Returns the projection matrix for this camera.
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        self.proj.matrix(aspect)
    }
}
