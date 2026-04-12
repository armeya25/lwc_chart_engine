use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::types::ChartCommand;


#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub side: String,
    pub qty: f64,
    pub entry: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub pnl: f64,
    pub time: Option<i64>,
}

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPosition {
    pub id: String,
    pub side: String,
    pub qty: f64,
    pub entry: f64,
    pub exit: f64,
    pub pnl: f64,
    pub entry_time: Option<i64>,
    pub exit_time: Option<i64>,
}

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone)]
pub struct PaperTrader {
    pub positions: Vec<Position>,
    pub history: Vec<ClosedPosition>,
    pub last_price: f64,
}

impl PaperTrader {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            history: Vec::new(),
            last_price: 0.0,
        }
    }

    pub fn add_position(&mut self, pos: Position) {
        self.positions.push(pos);
    }

    pub fn remove_position_by_id(&mut self, id: &str) -> Option<Position> {
        if let Some(idx) = self.positions.iter().position(|p| p.id == id) {
            Some(self.positions.remove(idx))
        } else {
            None
        }
    }

    pub fn update_price(&mut self, price: f64) -> Vec<ChartCommand> {
        self.last_price = price;
        let mut to_remove = Vec::new();
        let mut commands = Vec::new();

        for (idx, p) in self.positions.iter_mut().enumerate() {
            p.price = price;
            let is_buy = p.side == "buy";
            let diff = if is_buy { price - p.entry } else { p.entry - price };
            p.pnl = diff * p.qty;

            if let Some(tp) = p.tp {
                if (is_buy && price >= tp) || (!is_buy && price <= tp) {
                    let mut cmd = ChartCommand::new("show_notification", "chart-0");
                    cmd.data = Some(json!({
                        "message": format!("TP Hit! Closed {} at {:.2}", p.side, price),
                        "type": "success"
                    }));
                    commands.push(cmd);
                    
                    self.history.push(ClosedPosition {
                        id: p.id.clone(),
                        side: p.side.clone(),
                        qty: p.qty,
                        entry: p.entry,
                        exit: price,
                        pnl: p.pnl,
                        entry_time: p.time,
                        exit_time: None,
                    });
                    
                    to_remove.push(idx);
                    continue;
                }
            }

            if let Some(sl) = p.sl {
                if (is_buy && price <= sl) || (!is_buy && price >= sl) {
                    let mut cmd = ChartCommand::new("show_notification", "chart-0");
                    cmd.data = Some(json!({
                        "message": format!("SL Hit! Closed {} at {:.2}", p.side, price),
                        "type": "error"
                    }));
                    commands.push(cmd);
                    
                    self.history.push(ClosedPosition {
                        id: p.id.clone(),
                        side: p.side.clone(),
                        qty: p.qty,
                        entry: p.entry,
                        exit: price,
                        pnl: p.pnl,
                        entry_time: p.time,
                        exit_time: None,
                    });
                    
                    to_remove.push(idx);
                }
            }
        }

        if !to_remove.is_empty() {
            for &idx in to_remove.iter().rev() {
                let p = self.positions.remove(idx);
                let mut remove_cmd = ChartCommand::new("remove_position", "chart-0");
                remove_cmd.data = Some(json!({"id": p.id}));
                commands.push(remove_cmd);
            }
        }

        if !to_remove.is_empty() || !self.positions.is_empty() {
            let mut sync_cmd = ChartCommand::new("update_positions", "chart-0");
            sync_cmd.data = Some(json!(self.positions));
            commands.push(sync_cmd);
            
            if !to_remove.is_empty() {
                let mut hist_cmd = ChartCommand::new("update_history", "chart-0");
                hist_cmd.data = Some(json!(self.history));
                commands.push(hist_cmd);
            }
        }

        commands
    }

    pub fn execute(&mut self, id: String, side: String, qty: f64, entry_price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>) -> Vec<ChartCommand> {
        let price = entry_price.unwrap_or(self.last_price);
        if price == 0.0 { 
            let mut cmd = ChartCommand::new("show_notification", "chart-0");
            cmd.data = Some(json!({
                "message": "Execution Failed: Market price not available (Wait for chart to start ticking)",
                "type": "error"
            }));
            return vec![cmd];
        }

        let pos = Position {
            id,
            side: side.to_lowercase(),
            qty,
            entry: price,
            price,
            tp,
            sl,
            pnl: 0.0,
            time,
        };

        self.positions.push(pos);
        
        let mut sync_cmd = ChartCommand::new("update_positions", "chart-0");
        sync_cmd.data = Some(json!(self.positions));
        vec![sync_cmd]
    }

    pub fn handle_callback(&mut self, data: Value) -> Vec<ChartCommand> {
        let id = data.get("id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
        let side = data.get("side").and_then(|v| v.as_str()).unwrap_or("buy").to_string();
        let qty = data.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let tp = data.get("tp").and_then(|v| v.as_f64());
        let sl = data.get("sl").and_then(|v| v.as_f64());
        
        self.execute(id, side, qty, None, tp, sl, None)
    }

    pub fn close_position(&mut self, side: String, qty: f64, entry: f64, id: Option<String>) -> Vec<ChartCommand> {
        let mut commands = Vec::new();
        let found_idx = self.positions.iter().position(|p| {
            if let Some(target_id) = &id {
                p.id == *target_id
            } else {
                p.side == side && (p.qty - qty).abs() < 0.0001 && (p.entry - entry).abs() < 0.0001
            }
        });

        if let Some(idx) = found_idx {
            let p = self.positions.remove(idx);
            let exit_price = if self.last_price > 0.0 { self.last_price } else { p.entry };
            
            let is_buy = p.side == "buy";
            let pnl = if is_buy { exit_price - p.entry } else { p.entry - exit_price } * p.qty;
            
            let pos_id = p.id.clone();

            self.history.push(ClosedPosition {
                id: pos_id.clone(),
                side: p.side,
                qty: p.qty,
                entry: p.entry,
                exit: exit_price,
                pnl,
                entry_time: p.time,
                exit_time: None,
            });
            
            let mut remove_cmd = ChartCommand::new("remove_position", "chart-0");
            remove_cmd.data = Some(json!({"id": pos_id}));
            commands.push(remove_cmd);

            let mut sync_cmd = ChartCommand::new("update_positions", "chart-0");
            sync_cmd.data = Some(json!(self.positions));
            commands.push(sync_cmd);
            
            let mut hist_cmd = ChartCommand::new("update_history", "chart-0");
            hist_cmd.data = Some(json!(self.history));
            commands.push(hist_cmd);
            
            let mut notify = ChartCommand::new("show_notification", "chart-0");
            notify.data = Some(json!({
                "message": format!("Closed {} {} at {:.2} (P/L: {:.2})", side.to_uppercase(), qty, exit_price, pnl),
                "type": "info"
            }));
            commands.push(notify);
        }
        
        commands
    }

    pub fn handle_close_callback(&mut self, data: Value) -> Vec<ChartCommand> {
        let side = data.get("side").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let qty = data.get("qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let entry = data.get("entry").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let id = data.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        self.close_position(side, qty, entry, id)
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl Position {
    #[new] #[pyo3(signature = (id, side, qty, entry, price, tp=None, sl=None, time=None))] pub fn py_new(id: String, side: String, qty: f64, entry: f64, price: f64, tp: Option<f64>, sl: Option<f64>, time: Option<i64>) -> Self {
        Self { id, side, qty, entry, price, tp, sl, pnl: 0.0, time }
    }
    #[getter] pub fn id(&self) -> String { self.id.clone() }
    #[getter] pub fn side(&self) -> String { self.side.clone() }
    #[getter] pub fn qty(&self) -> f64 { self.qty }
    #[getter] pub fn entry(&self) -> f64 { self.entry }
    #[getter] pub fn price(&self) -> f64 { self.price }
    #[getter] pub fn tp(&self) -> Option<f64> { self.tp }
    #[getter] pub fn sl(&self) -> Option<f64> { self.sl }
    #[getter] pub fn pnl(&self) -> f64 { self.pnl }
    #[getter] pub fn time(&self) -> Option<i64> { self.time }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl ClosedPosition {
    #[getter] pub fn id(&self) -> String { self.id.clone() }
    #[getter] pub fn side(&self) -> String { self.side.clone() }
    #[getter] pub fn qty(&self) -> f64 { self.qty }
    #[getter] pub fn entry(&self) -> f64 { self.entry }
    #[getter] pub fn exit(&self) -> f64 { self.exit }
    #[getter] pub fn pnl(&self) -> f64 { self.pnl }
    #[getter] pub fn entry_time(&self) -> Option<i64> { self.entry_time }
    #[getter] pub fn exit_time(&self) -> Option<i64> { self.exit_time }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl PaperTrader {
    #[getter]
    pub fn positions(&self) -> Vec<Position> { self.positions.clone() }

    #[getter]
    pub fn last_price(&self) -> f64 { self.last_price }

    #[getter]
    pub fn history(&self) -> Vec<ClosedPosition> { self.history.clone() }

    #[pyo3(name = "update_price")]
    pub fn py_update_price(&mut self, price: f64) -> Vec<String> {
        self.update_price(price).iter().map(|c| serde_json::to_string(c).unwrap()).collect()
    }

    #[pyo3(name = "execute", signature = (side, qty, id=None, price=None, tp=None, sl=None, time=None))]
    pub fn py_execute(&mut self, side: String, qty: f64, id: Option<String>, price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>) -> Vec<String> {
        let actual_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        self.execute(actual_id, side, qty, price, tp, sl, time).iter().map(|c| serde_json::to_string(c).unwrap()).collect()
    }

    #[pyo3(name = "handle_callback")]
    pub fn py_handle_callback(&mut self, data_json: String) -> Vec<String> {
        if let Ok(data) = serde_json::from_str(&data_json) {
            return self.handle_callback(data).iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        }
        Vec::new()
    }

    #[pyo3(name = "close_position", signature = (side, qty, entry, id=None))]
    pub fn py_close_position(&mut self, side: String, qty: f64, entry: f64, id: Option<String>) -> Vec<String> {
        self.close_position(side, qty, entry, id).iter().map(|c| serde_json::to_string(c).unwrap()).collect()
    }

    #[pyo3(name = "handle_close_callback")]
    pub fn py_handle_close_callback(&mut self, data_json: String) -> Vec<String> {
        if let Ok(data) = serde_json::from_str(&data_json) {
            return self.handle_close_callback(data).iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        }
        Vec::new()
    }
}
