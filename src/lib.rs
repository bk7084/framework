use pyo3::prelude::*;

// mod input;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(_py: Python, module: &PyModule) -> PyResult<()> {
    // module.add_class::<mesh::Mesh>()?;
    // // module.add_class::<input::InputState>()?;
    // module.add_class::<input::MouseButton>()?;
    // module.add_class::<input::KeyCode>()?;
    module.add_function(wrap_pyfunction!(sum_as_string, module)?)?;
    Ok(())
}
