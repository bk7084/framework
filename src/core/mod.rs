//! Core module
//!
//! This module contains the core types and functions of the framework.
pub mod camera;
mod color;
pub use color::*;
pub mod assets;
mod material;
pub use material::*;
mod light;
pub use light::*;
pub mod mesh;

mod transform;
pub use transform::*;

mod texture;
pub use texture::*;
mod typedef;
pub use typedef::*;

/// The alignment of a plane.
#[pyo3::pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Alignment {
    /// The XY plane.
    XY,
    /// The XZ plane.
    XZ,
    /// The YZ plane.
    YZ,
}
