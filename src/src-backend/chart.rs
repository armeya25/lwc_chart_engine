use std::collections::HashMap;
use serde_json::{json, Value};
#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;
use crate::drawings::DrawingTool;
use crate::trader::{PaperTrader, Position};
use crate::types::{ChartCommand, IndicatorConfig, Point, IndicatorType};
use crate::indicators;
use uuid::Uuid;
#[cfg(feature = "python-bridge")]
use pyo3_polars::PyDataFrame;
use polars::prelude::{DataFrame, DataType, PolarsResult, Series as PolarsSeries, NamedFrom};
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
        
        self.data = df_to_points(&df);
        self.indicator_states.clear();
        
        let mut commands = Vec::new();
        
        let mut main_cmd = ChartCommand::new("set_series_data", &self.chart_id);
        main_cmd.series_id = Some(self.series_id.clone());
        main_cmd.data = Some(serde_json::to_value(&self.data).unwrap());
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
            if let (Ok(vol_col), Ok(open_col), Ok(close_col), Ok(time_col)) = (df.column("volume"), df.column("open"), df.column("close"), df.column("time")) {
                let vol_values = vol_col.f64().unwrap();
                let open_values = open_col.f64().unwrap();
                let close_values = close_col.f64().unwrap();
                let times = time_col.i64().unwrap();
                
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

        let point = Point {
            time: df.column("time").unwrap().i64().map_err(|e| e.to_string())?.get(0).unwrap(),
            open: get_df_f64(&df, "open", 0).unwrap_or(0.0),
            high: get_df_f64(&df, "high", 0).unwrap_or(0.0),
            low: get_df_f64(&df, "low", 0).unwrap_or(0.0),
            close: get_df_f64(&df, "close", 0).unwrap_or(0.0),
            volume: get_df_f64(&df, "volume", 0),
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

    pub fn remove_indicator(&mut self, target_sid: &str) -> Vec<String> {
        let mut removed = Vec::new();
        let mut to_keep = Vec::new();
        
        for config in self.indicators.drain(..) {
            if config.target_series_id == target_sid {
                removed.push(config.target_series_id.clone());
                for id in config.extra_target_ids.values() {
                    removed.push(id.clone());
                }
            } else {
                to_keep.push(config);
            }
        }
        self.indicators = to_keep;

        for id in &removed {
            self.indicator_states.remove(id);
        }
        removed
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

    #[pyo3(name = "set_auto_volume")]
    pub fn py_set_auto_volume(&mut self, enabled: bool) {
        self.auto_volume_enabled = enabled;
    }

    #[pyo3(name = "remove_indicator")]
    pub fn py_remove_indicator(&mut self, target_sid: String) -> Vec<String> {
        self.remove_indicator(&target_sid)
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
    pub fn new() -> Self {
        let mut series_map = HashMap::new();
        series_map.insert("main".to_string(), Series::new("main".to_string(), "Main".to_string(), "chart-0".to_string()));
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

    fn _create_series(&mut self, action: &str, prefix: &str, name: String, chart_id: String) -> Result<(String, String), String> {
        let sid = format!("{}_{}", prefix, Uuid::new_v4().to_string().split_at(8).0);
        self.series.insert(sid.clone(), Series::new(sid.clone(), name.clone(), chart_id.clone()));
        let mut cmd = ChartCommand::new(action, &chart_id);
        cmd.series_id = Some(sid.clone());
        cmd.name = Some(name.clone());
        cmd.options = Some(json!({"name": name}));
        Ok((sid, serde_json::to_string(&cmd).unwrap()))
    }

    pub fn create_line_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        self._create_series("create_line_series", "line", name, chart_id)
    }

    pub fn create_candlestick_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        self._create_series("create_candlestick_series", "candle", name, chart_id)
    }

    pub fn create_histogram_series(&mut self, name: String, chart_id: String) -> Result<(String, String), String> {
        self._create_series("create_histogram_series", "hist", name, chart_id)
    }
    pub fn set_tooltip(&mut self, enabled: bool) -> Result<String, String> {
        self.tooltip_enabled = enabled;
        let mut cmd = ChartCommand::new("set_tooltip", "chart-0");
        cmd.data = Some(json!({"enabled": enabled}));
        Ok(serde_json::to_string(&cmd).unwrap())
    }

    fn _parse_indicator_type(ind_type_str: &str) -> Result<IndicatorType, String> {
        match ind_type_str.to_lowercase().as_str() {
            "sma" => Ok(IndicatorType::Sma),
            "ema" => Ok(IndicatorType::Ema),
            "rsi" => Ok(IndicatorType::Rsi),
            "macd" => Ok(IndicatorType::Macd),
            "bollingerbands" | "bbands" | "bollinger" => Ok(IndicatorType::BollingerBands),
            "atr" => Ok(IndicatorType::Atr),
            "stochastic" | "stoch" => Ok(IndicatorType::Stochastic),
            "cci" => Ok(IndicatorType::Cci),
            "vwap" => Ok(IndicatorType::Vwap),
            "williamsr" | "williams" => Ok(IndicatorType::WilliamsR),
            "dema" => Ok(IndicatorType::Dema),
            "tema" => Ok(IndicatorType::Tema),
            "wma" => Ok(IndicatorType::Wma),
            "hma" => Ok(IndicatorType::Hma),
            "mfi" => Ok(IndicatorType::Mfi),
            "roc" => Ok(IndicatorType::Roc),
            "keltnerchannels" | "keltner" => Ok(IndicatorType::KeltnerChannels),
            "donchianchannels" | "donchian" => Ok(IndicatorType::DonchianChannels),
            "obv" => Ok(IndicatorType::Obv),
            "adl" => Ok(IndicatorType::Adl),
            _ => Err(format!("Unknown indicator type: {}", ind_type_str)),
        }
    }

    fn _generate_display_name(ind_type_str: &str) -> String {
        let clean_type = ind_type_str.replace('"', "").to_lowercase();
        match clean_type.as_str() {
            "sma" => "SMA".to_string(),
            "ema" => "EMA".to_string(),
            "rsi" => "RSI".to_string(),
            "macd" => "MACD".to_string(),
            "bbands" | "bollinger" | "bollingerbands" => "Bollinger Bands".to_string(),
            "atr" => "ATR".to_string(),
            "stoch" | "stochastic" => "Stochastic".to_string(),
            "cci" => "CCI".to_string(),
            "vwap" => "VWAP".to_string(),
            "williamsr" | "williams" => "Williams %R".to_string(),
            "dema" => "Double EMA".to_string(),
            "tema" => "Triple EMA".to_string(),
            "wma" => "WMA".to_string(),
            "hma" => "HMA".to_string(),
            "obv" => "OBV".to_string(),
            "adl" => "ADL".to_string(),
            "keltnerchannels" | "keltner" => "Keltner Channels".to_string(),
            "donchianchannels" | "donchian" => "Donchian Channels".to_string(),
            "mfi" => "MFI".to_string(),
            "roc" => "ROC".to_string(),
            _ => clean_type.to_uppercase(),
        }
    }

    fn _generate_group_name(ind_type_str: &str, params: &HashMap<String, Value>) -> String {
        let display_type = Self::_generate_display_name(ind_type_str);
        if params.is_empty() {
            display_type
        } else {
            let mut keys: Vec<&String> = params.keys()
                .filter(|k| *k != "color" && *k != "owner_id")
                .collect();
            keys.sort();
            let p_vals: Vec<String> = keys.iter().map(|k| {
                let v = params.get(*k).unwrap();
                if v.is_string() { v.as_str().unwrap().to_string() }
                else { v.to_string() }
            }).collect();
            if p_vals.is_empty() {
                display_type
            } else {
                format!("{}({})", display_type, p_vals.join(", "))
            }
        }
    }

    fn _resolve_indicator_color(&self, source_sid: &str, role: &str, default_color: &str, params: &HashMap<String, Value>) -> String {
        let palette = [
            "#2196F3", "#FF9800", "#E91E63", "#4CAF50", "#9C27B0",
            "#00BCD4", "#FFC107", "#009688", "#673AB7", "#3F51B5",
            "#8BC34A", "#FF5722", "#607D8B", "#F44336", "#03A9F4",
            "#CDDC39", "#795548", "#9E9E9E", "#FF4081", "#00E676",
            "#651FFF", "#AEEA00", "#FFD600", "#FF6E40", "#18FFFF",
            "#76FF03", "#D4E157", "#FFA726", "#26C6DA", "#AB47BC",
            "#FF7043", "#5C6BC0", "#26A69A", "#D4E157", "#FFEE58",
            "#BDBDBD", "#90A4AE", "#ec407a", "#7e57c2", "#26c6da"
        ];
        
        if let Some(c) = params.get("color").and_then(|v| v.as_str()) {
            c.to_string()
        } else if role == "main" {
            let count = self.series.get(source_sid).map(|s| s.indicators.len()).unwrap_or(0);
            palette[count % palette.len()].to_string()
        } else {
            default_color.to_string()
        }
    }


    pub fn add_indicator_v2(
        &mut self,
        mut source_sid: String,
        ind_type_str: String,
        params_json: String,
        chart_id: String,
    ) -> Result<String, String> {

        source_sid = self.resolve_source_series(&source_sid);
        let ind_type = Self::_parse_indicator_type(&ind_type_str)?;

        let params: HashMap<String, Value> = serde_json::from_str(&params_json).map_err(|e| e.to_string())?;

        let sub_info = indicators::get_sub_series_info(ind_type);
        let mut creations = Vec::new();
        let mut ids = HashMap::new();
        let mut main_id = String::new();

        let is_osc = indicators::is_oscillator(ind_type);
        let pane_id = if is_osc { Some(format!("pane_{}", Uuid::new_v4().to_string().split_at(8).0)) } else { None };

        // 1. Generate IDs first so we can use main_id for grouping
        for (role, _label, _color) in &sub_info {
            let sid = format!("{}_{}", role, Uuid::new_v4().to_string().split_at(8).0);
            if *role == "main" { main_id = sid.clone(); }
            else { ids.insert(role.to_string(), sid.clone()); }
        }

        let group_name = Self::_generate_group_name(&ind_type_str, &params);

        // 2. Create series and commands with grouping
        for (role, label, color) in sub_info {
            let sid = if role == "main" { main_id.clone() } else { ids.get(role).unwrap().clone() };
            let series_label = if label.is_empty() || label == "Value" { group_name.clone() } else { label.to_string() };

            self.series.insert(sid.clone(), Series::new(sid.clone(), series_label.clone(), chart_id.clone()));

            let action = if role == "hist" { "create_histogram_series" } else { "create_line_series" };
            let mut cmd = ChartCommand::new(action, &chart_id);
            cmd.series_id = Some(sid.clone());
            cmd.name = Some(series_label.clone());
            
            // Add grouping metadata for the legend
            cmd.extra.extend([
                ("indicator".to_string(), json!(main_id)),
                ("indicatorTypeName".to_string(), json!(group_name)),
                ("humanName".to_string(), json!(series_label)),
            ]);
            
            if role == "main" {
                cmd.extra.extend([
                    ("indicatorParams".to_string(), json!(params)),
                    ("indicatorMetadata".to_string(), indicators::get_indicator_params_schema(ind_type)),
                    ("ind_type".to_string(), json!(ind_type_str)),
                    ("owner_id".to_string(), json!(source_sid)),
                ]);
            }

            let final_color = self._resolve_indicator_color(&source_sid, role, color, &params);
            let mut options = json!({
                "color": final_color,
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
            let source = self.series.get_mut(&source_sid).ok_or(format!("Source series {} not found", source_sid))?;
            let config = IndicatorConfig {
                indicator_type: ind_type,
                target_series_id: main_id.clone(),
                chart_id: chart_id.clone(),
                extra_target_ids: ids.clone(),
                params: params,
            };
            source.indicators.push(config.clone());

            // --- 2. Historical Fallback ---
            // If the dataframe cache is missing (e.g. after updates), reconstruct it from buffer
            let df_to_use = if let Some(df) = &source.data_df {
                Some(df.clone())
            } else if !source.data.is_empty() {
                points_to_df(&source.data).ok()
            } else {
                None
            };

            if let Some(df) = df_to_use {
                if let Ok(cmd) = indicators::calculate_batch(&config, &df) {
                    data_cmds.push(cmd);
                }
            }
        }

        let mut final_cmds = creations;
        for cmd_str in data_cmds {
            for single_cmd in cmd_str.split('\n') {
                if !single_cmd.is_empty() {
                    final_cmds.push(single_cmd.to_string());
                }
            }
        }

        let res = json!({
            "mainId": main_id,
            "extraIds": ids,
            "commands": final_cmds
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

    pub fn remove_series(&mut self, sid: &str) {
        self.series.remove(sid);
    }

    pub fn remove_indicator(&mut self, target_sid: &str) -> Vec<String> {
        let mut all_removed = Vec::new();
        for s in self.series.values_mut() {
            all_removed.extend(s.remove_indicator(target_sid));
        }
        
        for id in &all_removed {
            self.series.remove(id);
        }
        all_removed
    }

    fn resolve_source_series(&self, source_sid: &str) -> String {
        if self.series.contains_key(source_sid) && !self.series.get(source_sid).unwrap().data.is_empty() {
             return source_sid.to_string();
        }
        let found_sid = self.series.iter()
            .filter(|(_, s)| !s.data.is_empty())
            .map(|(id, _)| id.clone())
            .next();
            
        found_sid.unwrap_or_else(|| source_sid.to_string())
    }

    pub fn trader_update_price(&mut self, price: f64) -> Vec<String> {
        let mut cmds = self.trader.update_price(price).iter().map(|c| serde_json::to_string(c).unwrap()).collect::<Vec<_>>();
        
        let positions = &self.trader.positions;
        if positions.is_empty() {
            if self.toolbox.last_position_state != "CLEARED" {
                cmds.extend(self.toolbox.sync_active_position(false, None, None, None, None, None, None, "chart-0".to_string()));
            }
        } else {
            let p = &positions[0];
            cmds.extend(self.toolbox.sync_active_position(
                true, 
                p.time, 
                Some(p.entry), 
                p.sl, 
                p.tp, 
                Some(p.side.clone()), 
                None, 
                "chart-0".to_string()
            ));
        }
        cmds
    }

    pub fn create_position(&mut self, 
        start_time: i64, entry_price: f64, sl_price: f64, tp_price: f64, 
        end_time: Option<i64>, visible: bool, side: String, quantity: f64, text: Option<String>, chart_id: String) -> (String, Vec<String>) {
        
        // 1. Create visual tool in toolbox
        let (pos_id, cmds) = self.toolbox._create_position(start_time, entry_price, sl_price, tp_price, end_time, visible, &side, quantity, text, &chart_id);
        
        // 2. Register in trader
        let trader_side = if side.to_lowercase() == "long" { "buy".to_string() } else { "sell".to_string() };
        let pos = Position {
            id: pos_id.clone(),
            side: trader_side,
            qty: quantity,
            entry: entry_price,
            price: entry_price,
            tp: Some(tp_price),
            sl: Some(sl_price),
            pnl: 0.0,
            time: Some(start_time),
        };
        self.trader.add_position(pos);
        
        let cmds_json = cmds.iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        (pos_id, cmds_json)
    }

    pub fn trader_execute(&mut self, side: String, qty: f64, price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>, series_id: Option<String>) -> Vec<String> {
        let exec_price = price.unwrap_or(self.trader.last_price);
        let pos_id = Uuid::new_v4().to_string();
        let mut cmds = self.trader.execute(pos_id, side.clone(), qty, price, tp, sl, time).iter().map(|c| serde_json::to_string(c).unwrap()).collect::<Vec<_>>();
        
        if let Some(sid) = series_id {
            if exec_price > 0.0 {
                let is_buy = side.to_lowercase() == "buy";
                let text = format!("{} @ {:.2}", side.to_uppercase(), exec_price);
                let (pos, shape, color) = if is_buy {
                    ("belowBar", "arrowUp", "#00e676")
                } else {
                    ("aboveBar", "arrowDown", "#ff5252")
                };
                
                let time_val = time.unwrap_or(0); // If time is 0, frontend might handle it as "now" if possible, or we should use last point time
                
                let (_, marker_cmd) = self.toolbox._add_marker(&sid, time_val, pos, color, shape, &text, "chart-0");
                cmds.push(serde_json::to_string(&marker_cmd).unwrap());
            }
        }
        cmds
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

    #[pyo3(name = "remove_series")]
    pub fn py_remove_series(&mut self, sid: String) {
        self.remove_series(&sid);
    }

    #[pyo3(name = "remove_indicator")]
    pub fn py_remove_indicator(&mut self, target_sid: String) -> Vec<String> {
        self.remove_indicator(&target_sid)
    }

    #[pyo3(name = "trader_update_price")]
    pub fn py_trader_update_price(&mut self, price: f64) -> Vec<String> {
        self.trader_update_price(price)
    }

    #[pyo3(name = "trader_execute", signature = (side, qty, price=None, tp=None, sl=None, time=None, series_id=None))]
    pub fn py_trader_execute(&mut self, side: String, qty: f64, price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>, series_id: Option<String>) -> Vec<String> {
        self.trader_execute(side, qty, price, tp, sl, time, series_id)
    }

    #[pyo3(name = "create_position", signature = (start_time, entry_price, sl_price, tp_price, end_time=None, visible=true, side="long".to_string(), quantity=1.0, text=None, chart_id="chart-0".to_string()))]
    pub fn py_create_position(&mut self, 
        start_time: i64, entry_price: f64, sl_price: f64, tp_price: f64, 
        end_time: Option<i64>, visible: bool, side: String, quantity: f64, text: Option<String>, chart_id: String) -> (String, Vec<String>) {
        self.create_position(start_time, entry_price, sl_price, tp_price, end_time, visible, side, quantity, text, chart_id)
    }
}

fn points_to_df(points: &[Point]) -> PolarsResult<DataFrame> {
    let times: Vec<i64> = points.iter().map(|p| p.time).collect();
    let opens: Vec<f64> = points.iter().map(|p| p.open).collect();
    let highs: Vec<f64> = points.iter().map(|p| p.high).collect();
    let lows: Vec<f64> = points.iter().map(|p| p.low).collect();
    let closes: Vec<f64> = points.iter().map(|p| p.close).collect();
    // Some points might not have volume, handle gracefully
    let volumes: Vec<f64> = points.iter().map(|p| p.volume.unwrap_or(0.0)).collect();

    let df = DataFrame::new(vec![
        PolarsSeries::new("time".into(), times).into(),
        PolarsSeries::new("open".into(), opens).into(),
        PolarsSeries::new("high".into(), highs).into(),
        PolarsSeries::new("low".into(), lows).into(),
        PolarsSeries::new("close".into(), closes).into(),
        PolarsSeries::new("volume".into(), volumes).into(),
    ])?;
    
    Ok(df)
}

fn df_to_points(df: &DataFrame) -> Vec<Point> {
    let times = df.column("time").unwrap().i64().unwrap();
    let mut points = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        points.push(Point {
            time: times.get(i).unwrap(),
            open: get_df_f64(df, "open", i).unwrap_or(0.0),
            high: get_df_f64(df, "high", i).unwrap_or(0.0),
            low: get_df_f64(df, "low", i).unwrap_or(0.0),
            close: get_df_f64(df, "close", i).unwrap_or(0.0),
            volume: get_df_f64(df, "volume", i),
        });
    }
    points
}

fn get_df_f64(df: &DataFrame, name: &str, row: usize) -> Option<f64> {
    df.column(name).ok().and_then(|s| {
        if s.dtype() == &DataType::Float64 {
            s.f64().ok().and_then(|ca| ca.get(row))
        } else {
            s.cast(&DataType::Float64).ok().and_then(|sc| sc.f64().ok().and_then(|ca| ca.get(row)))
        }
    })
}
