use crate::rstar::RTree;
use pyo3::prelude::*;
mod rstar;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn rstar(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RTree>()?;
    Ok(())
}
