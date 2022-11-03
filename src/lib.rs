mod mesh;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn bk7084rs(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<mesh::Mesh>()?;
    Ok(())
}
