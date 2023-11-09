//! Core module
//!
//! This module contains the core types and functions of the framework.
pub mod camera;
mod color;
pub use color::*;
pub mod assets;

pub mod geometry;
mod material;
pub use material::*;
mod light;
pub mod mesh;

mod transform;
pub use transform::*;

mod typedef;

pub use typedef::*;

/// A plane in 3D space.
#[pyo3::pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Plane {
    /// The XY plane.
    XY,
    /// The XZ plane.
    XZ,
    /// The YZ plane.
    YZ,
}
