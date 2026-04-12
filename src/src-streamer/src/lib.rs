pub mod modules;

use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use pyo3_polars::PyDataFrame;
use crate::modules::streamer::polars_basic::Streamer;

#[pyclass(subclass)]
pub struct PyStreamer {
    inner: Streamer,
}

#[pymethods]
impl PyStreamer {
    #[new]
    fn new() -> Self {
        PyStreamer {
            inner: Streamer::new(),
        }
    }

    fn set_stream_data(&mut self, file_path: &str, slice: i64, tf: &str) -> PyResult<bool> {
        match self.inner.set_stream_data(file_path, slice, tf) {
            Ok(_) => Ok(true),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e))),
        }
    }

    fn next_row(&mut self) -> bool {
        let streamer = &mut self.inner;
        if streamer.current_dt.is_none() {
            if streamer.total > 0 {
                streamer.current_idx = 0;
                streamer.current_dt = Some(streamer.dates[0]);
                streamer.candle_closed.current_dt = streamer.current_dt;
                return true;
            }
            return false;
        }

        let next_idx = streamer.current_idx + 1;
        if next_idx < streamer.total {
            streamer.current_idx = next_idx;
            streamer.current_dt = Some(streamer.dates[next_idx]);
            streamer.candle_closed.current_dt = streamer.current_dt;
            return true;
        }
        false
    }

    fn is_closed(&mut self, tf: &str) -> bool {
        match self.inner.is_closed(tf) {
            Ok(closed) => closed,
            Err(_) => false,
        }
    }

    fn get_chart_data<'py>(&self, tf: &str, py: Python<'py>) -> PyResult<PyObject> {
        match self.inner.get_chart_data(tf) {
            Ok(df) => {
                let pydf = PyDataFrame(df);
                match pydf.into_py_any(py) {
                    Ok(obj) => Ok(obj),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e))),
                }
            },
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
        }
    }

    fn get_forming_candle<'py>(&mut self, tf: &str, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        // Get period start date to include in the dictionary
        let idx = self.inner.current_idx;
        let dt = if idx < self.inner.total {
            self.inner.dates[idx]
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err("Current index out of bounds"));
        };

        let p_start = self.inner.candle_closed.get_period_start(dt, tf)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;

        match self.inner.get_forming_candle(tf) {
            Ok(res) => {
                let dict = pyo3::types::PyDict::new(py);
                for (k, v) in res {
                    dict.set_item(k, v)?;
                }
                // Add the date as a string
                let dt_str = p_start.format("%Y-%m-%dT%H:%M:%S").to_string();
                dict.set_item("date", dt_str)?;
                Ok(dict)
            },
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e)),
        }
    }


    fn get_current_dt_str(&self) -> Option<String> {
        self.inner.current_dt.as_ref().map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
    }
    fn is_stream_ended(&self) -> bool {
        self.inner.is_stream_ended()
    }
}

#[pymodule]
fn streamer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStreamer>()?;
    Ok(())
}
