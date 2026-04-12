use std::collections::{HashMap, HashSet};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::types::ChartCommand;
#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone)]
pub struct PriceLine {
    pub series_id: String,
    pub price: f64,
    pub color: String,
    pub width: i32,
    pub style: i32,
    pub text: String,
    pub axis_label_visible: bool,
    pub chart_id: String,
    pub line_id: String,
    pub is_visible: bool,
}

impl PriceLine {
    pub fn new(series_id: String, price: f64, color: String, chart_id: String) -> Self {
        Self {
            series_id,
            price,
            color,
            width: 1,
            style: 1,
            text: "".to_string(),
            axis_label_visible: true,
            chart_id,
            line_id: Uuid::new_v4().to_string(),
            is_visible: false,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<String> {
        let cmd = self.gen_update_command(price);
        cmd.map(|c| serde_json::to_string(&c).unwrap())
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl PriceLine {
    #[getter]
    pub fn series_id(&self) -> String { self.series_id.clone() }
    #[getter]
    pub fn price(&self) -> f64 { self.price }
    #[getter]
    pub fn color(&self) -> String { self.color.clone() }
    #[getter]
    pub fn width(&self) -> i32 { self.width }
    #[getter]
    pub fn style(&self) -> i32 { self.style }
    #[getter]
    pub fn text(&self) -> String { self.text.clone() }
    #[getter]
    pub fn axis_label_visible(&self) -> bool { self.axis_label_visible }
    #[getter]
    pub fn chart_id(&self) -> String { self.chart_id.clone() }
    #[getter]
    pub fn line_id(&self) -> String { self.line_id.clone() }
    #[getter]
    pub fn is_visible(&self) -> bool { self.is_visible }

    #[new]
    pub fn py_new(series_id: String, price: f64, color: String, chart_id: String) -> Self {
        Self::new(series_id, price, color, chart_id)
    }

    #[pyo3(name = "update")]
    pub fn py_update(&mut self, price: f64) -> Option<String> {
        self.update(price)
    }
}

impl PriceLine {
    pub fn gen_update_command(&mut self, price: f64) -> Option<ChartCommand> {
        if price == 0.0 {
            if self.is_visible {
                self.is_visible = false;
                let mut cmd = ChartCommand::new("remove_price_line", &self.chart_id);
                cmd.line_id = Some(self.line_id.clone());
                return Some(cmd);
            }
            return None;
        }

        if self.is_visible {
            let mut cmd = ChartCommand::new("update_price_line", &self.chart_id);
            cmd.line_id = Some(self.line_id.clone());
            cmd.options = Some(json!({"price": price}));
            Some(cmd)
        } else {
            self.is_visible = true;
            self.price = price;
            let mut cmd = ChartCommand::new("create_price_line", &self.chart_id);
            cmd.series_id = Some(self.series_id.clone());
            cmd.line_id = Some(self.line_id.clone());
            cmd.options = Some(json!({
                "price": price,
                "color": self.color,
                "lineWidth": self.width,
                "lineStyle": self.style,
                "title": self.text,
                "axisLabelVisible": self.axis_label_visible,
            }));
            Some(cmd)
        }
    }
}

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone)]
pub struct DrawingTool {
    pub markers: HashMap<String, Value>,
    pub lines: HashMap<String, PriceLine>,
    pub boxes: HashMap<String, Value>,
    pub positions: HashMap<String, Value>,
    pub line_tools: HashMap<String, Value>,
    pub chart_positions: HashMap<String, String>,
    pub category_index: HashMap<String, HashSet<String>>,
    pub last_position_state: String,
}

impl DrawingTool {
    pub fn new() -> Self {
        Self {
            markers: HashMap::new(),
            lines: HashMap::new(),
            boxes: HashMap::new(),
            positions: HashMap::new(),
            line_tools: HashMap::new(),
            chart_positions: HashMap::new(),
            category_index: HashMap::new(),
            last_position_state: "CLEARED".to_string(),
        }
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl DrawingTool {
    #[getter]
    pub fn lines(&self) -> HashMap<String, PriceLine> { self.lines.clone() }
    #[getter]
    pub fn chart_positions(&self) -> HashMap<String, String> { self.chart_positions.clone() }
    #[getter]
    pub fn last_position_state(&self) -> String { self.last_position_state.clone() }

    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    pub fn add_marker(&mut self, series_id: String, time: i64, position: String, color: String, shape: String, text: String, chart_id: String) -> (String, String) {
        let (marker_id, cmd) = self._add_marker(&series_id, time, &position, &color, &shape, &text, &chart_id);
        (marker_id, serde_json::to_string(&cmd).unwrap())
    }

    #[pyo3(signature = (start_time, start_price, end_time, end_price, color, border_color, text, category=None, chart_id="chart-0".to_string()))]
    pub fn create_box(&mut self, 
        start_time: i64, start_price: f64, 
        end_time: i64, end_price: f64, 
        color: String, border_color: String, 
        text: String, category: Option<String>, 
        chart_id: String) -> (String, Vec<String>) {
        
        let (box_id, cmds) = self._create_box(start_time, start_price, end_time, end_price, &color, &border_color, &text, category, &chart_id);
        let cmds_json = cmds.iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        (box_id, cmds_json)
    }

    pub fn remove_box(&mut self, box_id: String) -> String {
        let cmd = self._remove_box(&box_id);
        serde_json::to_string(&cmd).unwrap()
    }

    pub fn create_horizontal_line(&mut self, series_id: String, price: f64, color: String, chart_id: String) -> (String, Option<String>) {
        let (line_id, cmd) = self._create_horizontal_line(&series_id, price, &color, &chart_id);
        (line_id, cmd.map(|c| serde_json::to_string(&c).unwrap()))
    }

    #[pyo3(signature = (tool_type, start_time, start_price, end_time, end_price, color, width=1, style=0, visible=true, text="".to_string(), extended=false, chart_id="chart-0".to_string()))]
    pub fn create_line_tool(&mut self, 
        tool_type: String, 
        start_time: i64, start_price: f64, 
        end_time: i64, end_price: f64, 
        color: String, width: i32, style: i32, visible: bool, 
        text: String, extended: bool, chart_id: String) -> (String, String) {
        
        let tool_id = Uuid::new_v4().to_string();
        let tool_data = json!({
            "id": tool_id,
            "type": tool_type,
            "start_time": start_time,
            "start_price": start_price,
            "end_time": end_time,
            "end_price": end_price,
            "color": color,
            "width": width,
            "style": style,
            "visible": visible,
            "text": text,
            "extended": extended,
            "chart_id": chart_id,
        });

        self.line_tools.insert(tool_id.clone(), tool_data.clone());

        let mut cmd = ChartCommand::new("create_line_tool", &chart_id);
        cmd.id = Some(tool_id.clone());
        cmd.data = Some(tool_data);
        (tool_id, serde_json::to_string(&cmd).unwrap())
    }

    pub fn remove_line_tool(&mut self, tool_id: String) -> String {
        if let Some(data) = self.line_tools.remove(&tool_id) {
            if let Some(cid) = data.get("chart_id").and_then(|c| c.as_str()) {
                let mut cmd = ChartCommand::new("remove_line_tool", cid);
                cmd.id = Some(tool_id);
                return serde_json::to_string(&cmd).unwrap();
            }
        }
        let mut cmd = ChartCommand::new("remove_line_tool", "chart-0");
        cmd.id = Some(tool_id);
        serde_json::to_string(&cmd).unwrap()
    }

    pub fn clear_line_tools(&mut self) -> String {
        self.line_tools.clear();
        let cmd = ChartCommand::new("remove_all_line_tools", "chart-0");
        serde_json::to_string(&cmd).unwrap()
    }

    #[pyo3(signature = (chart_id=None))]
    pub fn clear_positions(&mut self, chart_id: Option<String>) -> Vec<String> {
        let cmds = self.gen_clear_positions_command(chart_id.as_deref());
        cmds.iter().map(|c| serde_json::to_string(c).unwrap()).collect()
    }

    #[pyo3(signature = (start_time, entry_price, sl_price, tp_price, end_time=None, visible=true, type_str="long".to_string(), quantity=1.0, text=None, chart_id="chart-0".to_string()))]
    pub fn create_position(&mut self, 
        start_time: i64, entry_price: f64, sl_price: f64, tp_price: f64, 
        end_time: Option<i64>, visible: bool, type_str: String, quantity: f64, text: Option<String>, chart_id: String) -> (String, Vec<String>) {
        
        let (pos_id, cmds) = self._create_position(start_time, entry_price, sl_price, tp_price, end_time, visible, &type_str, quantity, text, &chart_id);
        let cmds_json = cmds.iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        (pos_id, cmds_json)
    }

    pub fn remove_position(&mut self, pos_id: String) -> String {
        let cmd = self._remove_position(&pos_id);
        serde_json::to_string(&cmd).unwrap()
    }

    #[pyo3(signature = (is_opened, start_time=None, entry_price=None, sl_price=None, tp_price=None, pos_type=None, end_time=None, chart_id="chart-0".to_string()))]
    pub fn sync_active_position(&mut self, 
        is_opened: bool, 
        start_time: Option<i64>, 
        entry_price: Option<f64>, 
        sl_price: Option<f64>, 
        tp_price: Option<f64>, 
        pos_type: Option<String>, 
        end_time: Option<i64>, 
        chart_id: String) -> Vec<String> {
        
        if !is_opened {
            if self.last_position_state == "CLEARED" {
                return Vec::new();
            }
            let cmds = self.clear_positions(Some(chart_id));
            self.last_position_state = "CLEARED".to_string();
            return cmds;
        }

        if let (Some(st), Some(ep), Some(sl), Some(tp), Some(pt)) = (start_time, entry_price, sl_price, tp_price, pos_type) {
            let type_str = if pt == "buy" { "long".to_string() } else { "short".to_string() };
            
            // For now, simpler sync: remove and recreate if anything changed
            // Actually, we can just call create_position which handles removal
            let (_, cmds) = self.create_position(st, ep, sl, tp, end_time, true, type_str, 1.0, None, chart_id);
            self.last_position_state = "ACTIVE".to_string();
            return cmds;
        }
        
        Vec::new()
    }
}

impl DrawingTool {
    pub fn _add_marker(&mut self, series_id: &str, time: i64, position: &str, color: &str, shape: &str, text: &str, chart_id: &str) -> (String, ChartCommand) {
        let marker_id = format!("{}_{}", series_id, time);
        let data = json!({
            "id": marker_id,
            "time": time,
            "position": position,
            "color": color,
            "shape": shape,
            "text": text,
        });

        self.markers.insert(marker_id.clone(), data.clone());

        let mut cmd = ChartCommand::new("add_marker", chart_id);
        cmd.series_id = Some(series_id.to_string());
        cmd.data = Some(data);
        (marker_id, cmd)
    }

    pub fn _create_box(&mut self, 
        start_time: i64, start_price: f64, 
        end_time: i64, end_price: f64, 
        color: &str, border_color: &str, 
        text: &str, category: Option<String>, 
        chart_id: &str) -> (String, Vec<ChartCommand>) {
        
        let mut commands = Vec::new();
        let box_id = if let Some(ref cat) = category {
            format!("{}_{}_{}_{}", chart_id, cat, start_time, end_time)
        } else {
            format!("{}_{}_{}", chart_id, start_time, end_time)
        };

        if let Some(ref cat) = category {
            if let Some(ids) = self.category_index.get(cat) {
                for bid in ids {
                    if bid != &box_id {
                        let mut rm_cmd = ChartCommand::new("remove_box", chart_id);
                        rm_cmd.id = Some(bid.clone());
                        commands.push(rm_cmd);
                    }
                }
            }
        }

        if self.boxes.contains_key(&box_id) {
            return (box_id, commands);
        }

        let box_data = json!({
            "id": box_id,
            "start_time": start_time,
            "top_price": start_price,
            "end_time": end_time,
            "bottom_price": end_price,
            "color": color,
            "border_color": border_color,
            "text": text,
            "visible": true,
            "infinite": false,
            "category": category,
        });

        self.boxes.insert(box_id.clone(), box_data.clone());
        if let Some(cat) = category {
            self.category_index.entry(cat).or_insert_with(HashSet::new).insert(box_id.clone());
        }

        let mut cmd = ChartCommand::new("create_box", chart_id);
        cmd.id = Some(box_id.clone());
        cmd.data = Some(box_data);
        commands.push(cmd);

        (box_id, commands)
    }

    pub fn _remove_box(&mut self, box_id: &str) -> ChartCommand {
        if let Some(data) = self.boxes.remove(box_id) {
            if let Some(cat) = data.get("category").and_then(|c| c.as_str()) {
                if let Some(ids) = self.category_index.get_mut(cat) {
                    ids.remove(box_id);
                }
            }
        }
        let mut cmd = ChartCommand::new("remove_box", "chart-0");
        cmd.id = Some(box_id.to_string());
        cmd
    }

    pub fn _create_horizontal_line(&mut self, series_id: &str, price: f64, color: &str, chart_id: &str) -> (String, Option<ChartCommand>) {
        let mut line = PriceLine::new(series_id.to_string(), price, color.to_string(), chart_id.to_string());
        let cmd = line.gen_update_command(price);
        let line_id = line.line_id.clone();
        self.lines.insert(line_id.clone(), line);
        (line_id, cmd)
    }

    pub fn _create_position(&mut self, 
        start_time: i64, entry_price: f64, sl_price: f64, tp_price: f64, 
        end_time: Option<i64>, visible: bool, type_str: &str, quantity: f64, text: Option<String>, chart_id: &str) -> (String, Vec<ChartCommand>) {
        
        let mut commands = Vec::new();
        // Remove existing for this chart if exists
        if let Some(old_pid) = self.chart_positions.remove(chart_id) {
            self.positions.remove(&old_pid);
            let mut rm_cmd = ChartCommand::new("remove_position", chart_id);
            rm_cmd.id = Some(old_pid);
            commands.push(rm_cmd);
        }

        let pos_id = Uuid::new_v4().to_string();
        self.chart_positions.insert(chart_id.to_string(), pos_id.clone());

        let data = json!({
            "id": pos_id,
            "start_time": start_time,
            "end_time": end_time,
            "entry_price": entry_price,
            "sl_price": sl_price,
            "tp_price": tp_price,
            "visible": visible,
            "type": type_str,
            "quantity": quantity,
            "text": text,
            "chart_id": chart_id,
        });

        self.positions.insert(pos_id.clone(), data.clone());

        let mut cmd = ChartCommand::new("create_position", chart_id);
        cmd.id = Some(pos_id.clone());
        cmd.data = Some(data);
        commands.push(cmd);
        (pos_id, commands)
    }

    pub fn _remove_position(&mut self, pos_id: &str) -> ChartCommand {
        if let Some(data) = self.positions.remove(pos_id) {
            if let Some(cid) = data.get("chart_id").and_then(|c| c.as_str()) {
                if self.chart_positions.get(cid) == Some(&pos_id.to_string()) {
                    self.chart_positions.remove(cid);
                }
                let mut cmd = ChartCommand::new("remove_position", cid);
                cmd.id = Some(pos_id.to_string());
                return cmd;
            }
        }
        let mut cmd = ChartCommand::new("remove_position", "chart-0");
        cmd.id = Some(pos_id.to_string());
        cmd
    }

    pub fn gen_clear_positions_command(&mut self, chart_id: Option<&str>) -> Vec<ChartCommand> {
        let mut commands = Vec::new();
        if let Some(cid) = chart_id {
            if let Some(pid) = self.chart_positions.remove(cid) {
                self.positions.remove(&pid);
                let mut cmd = ChartCommand::new("remove_position", cid);
                cmd.id = Some(pid);
                commands.push(cmd);
            }
        } else {
            self.positions.clear();
            self.chart_positions.clear();
            commands.push(ChartCommand::new("remove_all_positions", "chart-0")); 
        }
        self.last_position_state = "CLEARED".to_string();
        commands
    }
}
