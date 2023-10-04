use pyo3::prelude::*;

mod input;
mod app;
mod mesh;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<app::AppState>()?;
    module.add_class::<mesh::Mesh>()?;
    module.add_class::<input::InputState>()?;
    module.add_class::<input::MouseButton>()?;
    module.add_class::<input::KeyCode>()?;
    module.add_function(wrap_pyfunction!(sum_as_string, module)?)?;
    module.add_function(wrap_pyfunction!(app::run_main_loop, module)?)?;

    // register_child_module(py, module, "app", |m| {
    //
    // });

    Ok(())
}

fn register_child_module(py: Python<'_>, parent: &PyModule, name: &str, add: impl FnOnce(&PyModule) -> PyResult<()>) -> PyResult<()> {
    let module = PyModule::new(py, name)?;
    add(module)?;
    parent.add_submodule(module.clone())?;
    Ok(())
}
