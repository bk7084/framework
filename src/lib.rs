mod mesh;
mod app;
mod context;
mod renderer;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;
    module.add_class::<app::AppState>()?;
    module.add_class::<mesh::Mesh>()?;
    Ok(())
}