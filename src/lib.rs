use pyo3::prelude::*;

mod app;
mod input;
mod mesh;

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<app::AppState>()?;
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;
    module.add_class::<mesh::Mesh>()?;
    module.add_class::<input::Input>()?;
    module.add_class::<input::MouseButton>()?;
    module.add_class::<input::KeyCode>()?;

    // register_child_module(py, module, "window", |m| {
    //     m.add_class::<window::Window>()?;
    //     // m.add_class::<window::EventLoop>()?;
    //     Ok(())
    // })?;

    Ok(())
}

fn register_child_module(
    py: Python<'_>,
    parent: &PyModule,
    name: &str,
    add: impl FnOnce(&PyModule) -> PyResult<()>,
) -> PyResult<()> {
    let module = PyModule::new(py, name)?;
    add(module)?;
    parent.add_submodule(module)?;
    Ok(())
}
