pub mod chart;
pub mod drawings;
pub mod time_utils;
pub mod types;

use pyo3::prelude::*;
pub use chart::{Chart, Series};
pub use drawings::{DrawingTool, PriceLine};
pub use time_utils::{py_ensure_timestamp, py_process_polars_data, py_set_backend_timezone};
pub use types::ChartCommand;

#[pymodule]
fn chart_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Chart>()?;
    m.add_class::<Series>()?;
    m.add_class::<DrawingTool>()?;
    m.add_class::<PriceLine>()?;
    m.add_function(wrap_pyfunction!(py_set_backend_timezone, m)?)?;
    m.add_function(wrap_pyfunction!(py_ensure_timestamp, m)?)?;
    m.add_function(wrap_pyfunction!(py_process_polars_data, m)?)?;
    Ok(())
}
