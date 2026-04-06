use std::collections::HashMap;
use serde_json::{json, Value};
use pyo3::prelude::*;
use crate::drawings::DrawingTool;
use crate::types::ChartCommand;
use uuid::Uuid;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Series {
    #[pyo3(get)]
    pub series_id: String,
    pub name: String,
    pub options: Value,
}

#[pymethods]
impl Series {
    pub fn set_data(&self, data_json: String) -> PyResult<String> {
        // Bypassing the broken Polars-Arrow bridge with JSON
        let data: Value = serde_json::from_str(&data_json).unwrap();
        let mut cmd = ChartCommand::new("set_series_data", "chart-0");
        cmd.series_id = Some(self.series_id.clone());
        cmd.data = Some(data);
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn update(&self, data_json: String) -> PyResult<String> {
        let data: Value = serde_json::from_str(&data_json).unwrap();
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

    pub fn apply_options(&self, options_json: String) -> PyResult<String> {
        let options: Value = serde_json::from_str(&options_json).unwrap();
        let mut cmd = ChartCommand::new("update_series_options", "chart-0");
        cmd.series_id = Some(self.series_id.clone());
        cmd.options = Some(options);
        Ok(serde_json::to_string(&cmd).unwrap())
    }
}

impl Series {
    pub fn new(series_id: String, name: String) -> Self {
        Self { series_id, name, options: json!({}) }
    }
}

#[pyclass]
pub struct Chart {
    #[pyo3(get)]
    pub series: HashMap<String, Series>,
    #[pyo3(get)]
    pub toolbox: DrawingTool,
    pub layout: String,
}

#[pymethods]
impl Chart {
    #[new]
    pub fn new() -> Self {
        let mut series_map = HashMap::new();
        series_map.insert("main".to_string(), Series::new("main".to_string(), "Main".to_string()));
        Self { series: series_map, toolbox: DrawingTool::new(), layout: "single".to_string() }
    }

    pub fn set_layout(&mut self, layout: String) -> PyResult<String> {
        self.layout = layout.clone();
        let mut cmd = ChartCommand::new("set_layout", "chart-0");
        cmd.options = Some(json!({"layout": layout}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn create_line_series(&mut self, name: String, chart_id: String) -> PyResult<(String, String)> {
        let sid = format!("line_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone()));
        let mut cmd = ChartCommand::new("create_line_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn create_candlestick_series(&mut self, name: String, chart_id: String) -> PyResult<(String, String)> {
        let sid = format!("candle_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone()));
        let mut cmd = ChartCommand::new("create_candlestick_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }
}
