#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_trait_impl)]

mod app;
mod camera;
pub mod core;
mod input;
mod mesh;
mod renderer;
mod scene;

use crate::core::SmlString;

#[derive(Debug, Clone)]
pub struct Labeled<T> {
    /// The label of the object.
    pub label: Option<SmlString>,
    /// The object.
    pub inner: T,
}

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn bkfw(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<app::WindowBuilder>()?;
    module.add_class::<app::PyAppState>()?;
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;
    module.add_class::<mesh::Mesh>()?;
    module.add_class::<input::Input>()?;
    module.add_class::<input::MouseButton>()?;
    module.add_class::<input::KeyCode>()?;
    Ok(())
}
