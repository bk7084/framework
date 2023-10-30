use pyo3::prelude::*;

mod app;
mod input;
mod mesh;
mod renderer;
mod typedefs;

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<app::AppState>()?;
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;
    module.add_class::<mesh::Mesh>()?;
    module.add_class::<input::Input>()?;
    module.add_class::<input::MouseButton>()?;
    module.add_class::<input::KeyCode>()?;
    Ok(())
}
