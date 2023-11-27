pub mod app;
pub mod core;
pub mod render;
pub mod scene;

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
    module.add_class::<app::PyWindowBuilder>()?;
    module.add_class::<app::PyAppState>()?;
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;
    module.add_class::<app::Input>()?;
    module.add_class::<app::MouseButton>()?;
    module.add_class::<app::KeyCode>()?;
    module.add_class::<core::camera::Projection>()?;
    module.add_class::<core::camera::ProjectionKind>()?;
    module.add_class::<core::mesh::Mesh>()?;
    module.add_class::<core::mesh::SubMesh>()?;
    module.add_class::<core::mesh::PyTopology>()?;
    module.add_class::<core::Material>()?;
    module.add_class::<core::ConcatOrder>()?;
    module.add_class::<core::Alignment>()?;
    module.add_class::<core::Color>()?;
    module.add_class::<core::IllumModel>()?;
    Ok(())
}
