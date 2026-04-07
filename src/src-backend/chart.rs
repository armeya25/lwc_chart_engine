use std::collections::HashMap;
use serde_json::{json, Value};
#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;
use crate::drawings::DrawingTool;
use crate::trader::PaperTrader;
use crate::types::ChartCommand;
use uuid::Uuid;

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone)]
pub struct Series {
    pub series_id: String,
    pub name: String,
    pub options: Value,
}

impl Series {
    pub fn new(series_id: String, name: String) -> Self {
        Self { series_id, name, options: json!({}) }
    }

    pub fn set_data(&self, data_json: String) -> Result<String, String> {
        let data: Value = serde_json::from_str(&data_json).map_err(|e| e.to_string())?;
        let mut cmd = ChartCommand::new("set_series_data", "chart-0");
        cmd.series_id = Some(self.series_id.clone());
        cmd.data = Some(data);
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn update(&self, data_json: String) -> Result<String, String> {
        let data: Value = serde_json::from_str(&data_json).map_err(|e| e.to_string())?;
        let row = if data.is_array() {
            data.as_array().unwrap().get(0).cloned().unwrap_or(json!({}))
        } else {
            data
        };
        let mut cmd = ChartCommand::new("update_series_data", "chart-0");
        cmd.series_id = Some(self.series_id.clone());
        cmd.data = Some(row);
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn apply_options(&self, options_json: String) -> Result<String, String> {
        let options: Value = serde_json::from_str(&options_json).map_err(|e| e.to_string())?;
        let mut cmd = ChartCommand::new("update_series_options", "chart-0");
        cmd.series_id = Some(self.series_id.clone());
        cmd.options = Some(options);
        Ok(serde_json::to_string(&cmd).unwrap())
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl Series {
    #[getter]
    pub fn series_id(&self) -> String {
        self.series_id.clone()
    }

    #[pyo3(name = "set_data")]
    pub fn py_set_data(&self, data_json: String) -> PyResult<String> {
        self.set_data(data_json).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "update")]
    pub fn py_update(&self, data_json: String) -> PyResult<String> {
        self.update(data_json).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "apply_options")]
    pub fn py_apply_options(&self, options_json: String) -> PyResult<String> {
        self.apply_options(options_json).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }
}

// (Series::new logic already integrated above)

#[cfg_attr(feature = "python-bridge", pyclass)]
pub struct Chart {
    pub series: HashMap<String, Series>,
    pub toolbox: DrawingTool,
    pub trader: PaperTrader,
    pub layout: String,
    pub tooltip_enabled: bool,
}

impl Chart {
        Self { 
            series: series_map, 
            toolbox: DrawingTool::new(), 
            trader: PaperTrader::new(),
            layout: "single".to_string(),
            tooltip_enabled: false,
        }
    }

    pub fn set_layout(&mut self, layout: String) -> Result<String, String> {
        self.layout = layout.clone();
        let mut cmd = ChartCommand::new("set_layout", "chart-0");
        cmd.options = Some(json!({"layout": layout}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn create_line_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        let sid = format!("line_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone()));
        let mut cmd = ChartCommand::new("create_line_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn create_candlestick_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        let sid = format!("candle_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone()));
        let mut cmd = ChartCommand::new("create_candlestick_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn set_tooltip(&mut self, enabled: bool) -> Result<String, String> {
        self.tooltip_enabled = enabled;
        let mut cmd = ChartCommand::new("set_tooltip", "chart-0");
        cmd.data = Some(json!({"enabled": enabled}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl Chart {
    #[getter]
    pub fn series(&self) -> HashMap<String, Series> {
        self.series.clone()
    }

    #[getter]
    pub fn toolbox(&self) -> DrawingTool {
        self.toolbox.clone()
    }

    #[getter]
    pub fn trader(&self) -> PaperTrader {
        self.trader.clone()
    }

    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    #[pyo3(name = "set_layout")]
    pub fn py_set_layout(&mut self, layout: String) -> PyResult<String> {
        self.set_layout(layout).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "create_line_series")]
    pub fn py_create_line_series(&mut self, name: String, chart_id: String) -> PyResult<(String, String)> {
        self.create_line_series(name, chart_id).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "create_candlestick_series")]
    pub fn py_create_candlestick_series(&mut self, name: String, chart_id: String) -> PyResult<(String, String)> {
        self.create_candlestick_series(name, chart_id).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "set_tooltip")]
    pub fn py_set_tooltip(&mut self, enabled: bool) -> PyResult<String> {
        self.set_tooltip(enabled).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }
}
