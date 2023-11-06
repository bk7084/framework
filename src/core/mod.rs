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
pub mod mesh;
mod typedef;

pub use typedef::*;
