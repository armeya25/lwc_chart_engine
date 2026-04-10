use std::collections::HashMap;
use serde_json::{json, Value};
#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;
use crate::drawings::DrawingTool;
use crate::trader::PaperTrader;
use crate::types::{ChartCommand, IndicatorConfig, IndicatorType, Point};
use crate::indicators;
use uuid::Uuid;
#[cfg(feature = "python-bridge")]
use pyo3_polars::PyDataFrame;
use polars::prelude::{DataFrame, DataType};
use crate::time_utils;

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone)]
pub struct Series {
    pub series_id: String,
    pub name: String,
    pub chart_id: String,
    pub options: Value,
    pub data: Vec<Point>,
    pub data_df: Option<DataFrame>,
    pub indicators: Vec<IndicatorConfig>,
    pub indicator_states: HashMap<String, crate::types::IndicatorState>,
    pub auto_volume_enabled: bool,
}

// Moved methods to consolidated #[pymethods] block below

impl Series {
    pub fn new(series_id: String, name: String, chart_id: String) -> Self {
        Self { 
            series_id, 
            name, 
            chart_id,
            options: json!({}), 
            data: Vec::new(), 
            data_df: None,
            indicators: Vec::new(),
            indicator_states: HashMap::new(),
            auto_volume_enabled: true,
        }
    }

    pub fn set_data(&mut self, df: DataFrame) -> Result<Vec<String>, String> {
        let df = time_utils::process_polars_data(df).map_err(|e| e.to_string())?;

        // 2. Validate required columns
        let required = ["time", "open", "high", "low", "close"];
        for col in &required {
            if !df.get_column_names().iter().any(|&s| s == *col) {
                return Err(format!("Missing required column: {}", col));
            }
        }

        self.data_df = Some(df.clone());
        
        let times = df.column("time").unwrap().i64().unwrap();
        let mut points = Vec::with_capacity(df.height());
        for i in 0..df.height() {
            let get_f64 = |name: &str, row: usize| -> f64 {
                let s = df.column(name).unwrap();
                if s.dtype() == &DataType::Float64 {
                    s.f64().unwrap().get(row).unwrap()
                } else {
                    s.cast(&DataType::Float64).unwrap().f64().unwrap().get(row).unwrap()
                }
            };

            points.push(Point {
                time: times.get(i).unwrap(),
                open: get_f64("open", i),
                high: get_f64("high", i),
                low: get_f64("low", i),
                close: get_f64("close", i),
                volume: df.column("volume").ok().and_then(|s| {
                    if s.dtype() == &DataType::Float64 {
                        s.f64().unwrap().get(i)
                    } else {
                        s.cast(&DataType::Float64).unwrap().f64().unwrap().get(i)
                    }
                }),
            });
        }
        self.data = points.clone();
        self.indicator_states.clear();
        
        let mut commands = Vec::new();
        
        let mut main_cmd = ChartCommand::new("set_series_data", &self.chart_id);
        main_cmd.series_id = Some(self.series_id.clone());
        main_cmd.data = Some(serde_json::to_value(&points).unwrap());
        commands.push(serde_json::to_string(&main_cmd).unwrap());
        
        for indicator in &self.indicators {
            if let Ok(ind_cmd) = indicators::calculate_batch(indicator, &df) {
                // Initialize state from the last calculated result if possible
                // (indicators::calculate_batch handles state extraction internally in future steps)
                commands.push(ind_cmd);
            }
        }

        // 3. Handle Auto-Volume
        if self.auto_volume_enabled {
            if let (Ok(vol_col), Ok(open_col), Ok(close_col)) = (df.column("volume"), df.column("open"), df.column("close")) {
                let vol_values = vol_col.f64().unwrap();
                let open_values = open_col.f64().unwrap();
                let close_values = close_col.f64().unwrap();
                
                let mut vol_data = Vec::with_capacity(df.height());
                for i in 0..df.height() {
                    if let (Some(v), Some(o), Some(c)) = (vol_values.get(i), open_values.get(i), close_values.get(i)) {
                        vol_data.push(json!({
                            "time": times.get(i).unwrap(),
                            "volume": v,
                            "open": o,
                            "close": c
                        }));
                    }
                }
                let mut vol_cmd = ChartCommand::new("set_volume_data", &self.chart_id);
                vol_cmd.series_id = Some(self.series_id.clone());
                vol_cmd.data = Some(serde_json::to_value(&vol_data).unwrap());
                commands.push(serde_json::to_string(&vol_cmd).unwrap());
            }
        }
        
        Ok(commands)
    }

    pub fn update(&mut self, df: DataFrame) -> Result<Vec<String>, String> {
        if df.height() == 0 { return Ok(Vec::new()); }
        
        // Process data (to handle aliases like 'date' in update items)
        let df = time_utils::process_polars_data(df).map_err(|e| e.to_string())?;

        let get_f64_opt = |name: &str, row: usize| -> Option<f64> {
            df.column(name).ok().and_then(|s| {
                if s.dtype() == &DataType::Float64 {
                    s.f64().ok().and_then(|ca| ca.get(row))
                } else {
                    s.cast(&DataType::Float64).ok().and_then(|sc| sc.f64().ok().and_then(|ca| ca.get(row)))
                }
            })
        };

        let point = Point {
            time: df.column("time").unwrap().i64().map_err(|e| e.to_string())?.get(0).unwrap(),
            open: get_f64_opt("open", 0).unwrap_or(0.0),
            high: get_f64_opt("high", 0).unwrap_or(0.0),
            low: get_f64_opt("low", 0).unwrap_or(0.0),
            close: get_f64_opt("close", 0).unwrap_or(0.0),
            volume: get_f64_opt("volume", 0),
        };

        if let Some(last) = self.data.last_mut() {
            if last.time == point.time {
                *last = point.clone();
            } else {
                self.data.push(point.clone());
            }
        } else {
            self.data.push(point.clone());
        }

        let mut commands = Vec::new();
        let mut main_cmd = ChartCommand::new("update_series_data", &self.chart_id);
        main_cmd.series_id = Some(self.series_id.clone());
        main_cmd.data = Some(serde_json::to_value(&point).unwrap());
        commands.push(serde_json::to_string(&main_cmd).unwrap());

        for indicator in &self.indicators {
            // Get or create state for this specific indicator series
            let state = self.indicator_states.get(&indicator.target_series_id).cloned();
            
            if let Ok((ind_cmd, new_state)) = indicators::calculate_step(indicator, &self.data, &point, state) {
                commands.push(ind_cmd);
                if let Some(s) = new_state {
                    self.indicator_states.insert(indicator.target_series_id.clone(), s);
                }
            }
        }

        // 4. Handle Auto-Volume update
        if self.auto_volume_enabled {
            if let Some(v) = point.volume {
                let mut vol_cmd = ChartCommand::new("update_volume_data", &self.chart_id);
                vol_cmd.series_id = Some(self.series_id.clone());
                vol_cmd.data = Some(json!({
                    "time": point.time,
                    "volume": v,
                    "open": point.open,
                    "close": point.close
                }));
                commands.push(serde_json::to_string(&vol_cmd).unwrap());
            }
        }
        
        Ok(commands)
    }

    pub fn apply_options(&self, options_json: String) -> Result<String, String> {
        let options: Value = serde_json::from_str(&options_json).map_err(|e| e.to_string())?;
        let mut cmd = ChartCommand::new("update_series_options", &self.chart_id);
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
    pub fn py_set_data(&mut self, pydf: PyDataFrame) -> PyResult<Vec<String>> {
        self.set_data(pydf.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "update")]
    pub fn py_update(&mut self, pydf: PyDataFrame) -> PyResult<Vec<String>> {
        self.update(pydf.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "apply_options")]
    pub fn py_apply_options(&self, options_json: String) -> PyResult<String> {
        self.apply_options(options_json).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "add_indicator")]
    pub fn py_add_indicator(&mut self, id: String, ind_type: String, params_json: String, extra_ids_json: Option<String>) -> PyResult<()> {
        let ind_enum = match ind_type.to_lowercase().as_str() {
            "sma" => IndicatorType::Sma,
            "ema" => IndicatorType::Ema,
            "rsi" => IndicatorType::Rsi,
            "macd" => IndicatorType::Macd,
            "bbands" | "bollinger" => IndicatorType::BollingerBands,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Unknown indicator type")),
        };
        let params: std::collections::HashMap<String, Value> = serde_json::from_str(&params_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        let mut extra_target_ids = std::collections::HashMap::new();
        if let Some(json) = extra_ids_json {
            extra_target_ids = serde_json::from_str(&json)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        }

        self.indicators.push(IndicatorConfig {
            indicator_type: ind_enum,
            target_series_id: id,
            chart_id: self.chart_id.clone(),
            extra_target_ids,
            params,
        });
        Ok(())
    }

    #[pyo3(name = "set_auto_volume")]
    pub fn py_set_auto_volume(&mut self, enabled: bool) {
        self.auto_volume_enabled = enabled;
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
    pub layout_toolbar_enabled: bool,
}

impl Chart {
    pub fn new() -> Self {
        let mut series_map = HashMap::new();
        series_map.insert("main".to_string(), Series::new("main".to_string(), "Main".to_string(), "chart-0".to_string()));
        Self { 
            series: series_map, 
            toolbox: DrawingTool::new(), 
            trader: PaperTrader::new(),
            layout: "single".to_string(),
            tooltip_enabled: false,
            layout_toolbar_enabled: false,
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
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone(), chart_id.clone()));
        let mut cmd = ChartCommand::new("create_line_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.name = Some(name.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn create_candlestick_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        let sid = format!("candle_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone(), chart_id.clone()));
        let mut cmd = ChartCommand::new("create_candlestick_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.name = Some(name.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn create_histogram_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        let sid = format!("hist_{}", Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone(), chart_id.clone()));
        let mut cmd = ChartCommand::new("create_histogram_series", &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.name = Some(name.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }
    pub fn set_tooltip(&mut self, enabled: bool) -> Result<String, String> {
        self.tooltip_enabled = enabled;
        let mut cmd = ChartCommand::new("set_tooltip", "chart-0");
        cmd.data = Some(json!({"enabled": enabled}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn set_layout_toolbar_visibility(&mut self, visible: bool) -> Result<String, String> {
        self.layout_toolbar_enabled = visible;
        let mut cmd = ChartCommand::new("set_layout_toolbar_visibility", "chart-0");
        cmd.data = Some(json!({"visible": visible}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    pub fn add_indicator_v2(
        &mut self,
        source_sid: String,
        ind_type_str: String,
        params_json: String,
        chart_id: String,
    ) -> Result<String, String> {
        use crate::types::IndicatorType;

        let ind_type = match ind_type_str.to_lowercase().as_str() {
            "sma" => IndicatorType::Sma,
            "ema" => IndicatorType::Ema,
            "rsi" => IndicatorType::Rsi,
            "macd" => IndicatorType::Macd,
            "bollingerbands" | "bbands" => IndicatorType::BollingerBands,
            "atr" => IndicatorType::Atr,
            "stochastic" => IndicatorType::Stochastic,
            "cci" => IndicatorType::Cci,
            "vwap" => IndicatorType::Vwap,
            "williamsr" => IndicatorType::WilliamsR,
            "dema" => IndicatorType::Dema,
            "tema" => IndicatorType::Tema,
            _ => return Err(format!("Unknown indicator type: {}", ind_type_str)),
        };

        let params: HashMap<String, Value> = serde_json::from_str(&params_json).map_err(|e| e.to_string())?;

        let sub_info = indicators::get_sub_series_info(ind_type);
        let mut creations = Vec::new();
        let mut ids = HashMap::new();
        let mut main_id = String::new();

        let is_osc = indicators::is_oscillator(ind_type);
        let pane_id = if is_osc { Some(format!("pane_{}", Uuid::new_v4().to_string().split_at(8).0)) } else { None };

        // 1. Generate IDs first so we can use main_id for grouping
        for (role, label, _color) in &sub_info {
            let sid = format!("{}_{}", role, Uuid::new_v4().to_string().split_at(8).0);
            if *role == "main" { main_id = sid.clone(); }
            else { ids.insert(role.to_string(), sid.clone()); }
        }

        // Generate descriptive group name (e.g., SMA(14) or MACD(12, 26, 9))
        let type_label = ind_type_str.replace('"', "").to_uppercase();
        let group_name = if params.is_empty() {
            type_label.clone()
        } else {
            // Sort by key to have a deterministic order
            let mut keys: Vec<&String> = params.keys().collect();
            keys.sort();
            let p_vals: Vec<String> = keys.iter().map(|k| {
                let v = params.get(*k).unwrap();
                if v.is_string() { v.as_str().unwrap().to_string() }
                else { v.to_string() }
            }).collect();
            format!("{}({})", type_label, p_vals.join(", "))
        };

        // 2. Create commands with grouping
        for (role, label, color) in sub_info {
            let sid = if role == "main" { main_id.clone() } else { ids.get(role).unwrap().clone() };
            
            // Unified naming: Use role label if provided, else group name
            let series_label = if label.is_empty() || label == "Value" {
                group_name.clone()
            } else {
                label.to_string()
            };

            self.series.insert(sid.clone(), Series::new(sid.clone(), series_label.clone(), chart_id.clone()));

            let action = if role == "hist" { "create_histogram_series" } else { "create_line_series" };
            let mut cmd = ChartCommand::new(action, &chart_id);
            cmd.series_id = Some(sid.clone());
            cmd.name = Some(series_label.clone());
            
            // Add grouping metadata for the legend
            cmd.extra.insert("indicator".to_string(), json!(main_id));
            cmd.extra.insert("indicator_type_name".to_string(), json!(group_name));
            cmd.extra.insert("human_name".to_string(), json!(series_label));
            
            if role == "main" {
                cmd.extra.insert("indicatorParams".to_string(), json!(params));
                cmd.extra.insert("indicatorMetadata".to_string(), indicators::get_indicator_params_schema(ind_type));
            }

            let mut options = json!({
                "color": color,
                "lineWidth": if ind_type_str == "sma" || ind_type_str == "ema" { 2 } else { 1 },
                "priceLineVisible": false
            });

            if let Some(pid) = &pane_id {
                options["priceScaleId"] = json!(pid);
            }

            cmd.options = Some(options);
            creations.push(serde_json::to_string(&cmd).unwrap());
        }

        let mut data_cmds = Vec::new();
        {
            let source = self.series.get_mut(&source_sid).ok_or("Source series not found")?;
            let config = IndicatorConfig {
                indicator_type: ind_type,
                target_series_id: main_id.clone(),
                chart_id: chart_id.clone(),
                extra_target_ids: ids.clone(),
                params: params,
            };
            source.indicators.push(config.clone());

            if let Some(df) = &source.data_df {
                if let Ok(cmd) = indicators::calculate_batch(&config, df) {
                    data_cmds.push(cmd);
                }
            }
        }

        let res = json!({
            "mainId": main_id,
            "extraIds": ids,
            "commands": creations.into_iter().chain(data_cmds.into_iter()).collect::<Vec<_>>()
        });

        Ok(res.to_string())
    }

    pub fn set_series_data(&mut self, sid: String, df: DataFrame) -> Result<Vec<String>, String> {
        let s = self.series.get_mut(&sid).ok_or("Series not found")?;
        s.set_data(df)
    }

    pub fn update_series_data(&mut self, sid: String, df: DataFrame) -> Result<Vec<String>, String> {
        let s = self.series.get_mut(&sid).ok_or("Series not found")?;
        s.update(df)
    }

    pub fn set_series_auto_volume(&mut self, sid: String, enabled: bool) -> Result<(), String> {
        let s = self.series.get_mut(&sid).ok_or("Series not found")?;
        s.auto_volume_enabled = enabled;
        Ok(())
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

    #[pyo3(name = "set_layout_toolbar_visibility")]
    pub fn py_set_layout_toolbar_visibility(&mut self, visible: bool) -> PyResult<String> {
        self.set_layout_toolbar_visibility(visible).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "add_indicator_v2")]
    pub fn py_add_indicator_v2(
        &mut self,
        source_sid: String,
        ind_type: String,
        params_json: String,
        chart_id: String,
    ) -> PyResult<String> {
        self.add_indicator_v2(source_sid, ind_type, params_json, chart_id)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "set_series_data")]
    pub fn py_set_series_data(&mut self, sid: String, pydf: PyDataFrame) -> PyResult<Vec<String>> {
        self.set_series_data(sid, pydf.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "update_series_data")]
    pub fn py_update_series_data(&mut self, sid: String, pydf: PyDataFrame) -> PyResult<Vec<String>> {
        self.update_series_data(sid, pydf.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "set_series_auto_volume")]
    pub fn py_set_series_auto_volume(&mut self, sid: String, enabled: bool) -> PyResult<()> {
        self.set_series_auto_volume(sid, enabled).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pyo3(name = "create_histogram_series")]
    pub fn py_create_histogram_series(&mut self, name: String, chart_id: String) -> PyResult<(String, String)> {
        self.create_histogram_series(name, chart_id).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }
}
