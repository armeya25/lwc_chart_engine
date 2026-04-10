use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::types::ChartCommand;

#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python-bridge", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
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
                        side: p.side.clone(),
                        qty: p.qty,
                        entry: p.entry,
                        exit: price,
                        pnl: p.pnl,
                        entry_time: p.time,
                        exit_time: None, // Could use current time here if available
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
            // Remove from back to front to avoid index shifting
            for &idx in to_remove.iter().rev() {
                self.positions.remove(idx);
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

    pub fn execute(&mut self, side: String, qty: f64, entry_price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>) -> Vec<ChartCommand> {
        let price = entry_price.unwrap_or(self.last_price);
        if price == 0.0 { return Vec::new(); }

        let pos = Position {
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
        let side = data.get("side").and_then(|v| v.as_str()).unwrap_or("buy").to_string();
        let qty = data.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let tp = data.get("tp").and_then(|v| v.as_f64());
        let sl = data.get("sl").and_then(|v| v.as_f64());
        
        self.execute(side, qty, None, tp, sl, None)
    }
}

#[cfg(feature = "python-bridge")]
#[pymethods]
impl Position {
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

    #[pyo3(name = "execute", signature = (side, qty, price=None, tp=None, sl=None, time=None))]
    pub fn py_execute(&mut self, side: String, qty: f64, price: Option<f64>, tp: Option<f64>, sl: Option<f64>, time: Option<i64>) -> Vec<String> {
        self.execute(side, qty, price, tp, sl, time).iter().map(|c| serde_json::to_string(c).unwrap()).collect()
    }

    #[pyo3(name = "handle_callback")]
    pub fn py_handle_callback(&mut self, data_json: String) -> Vec<String> {
        if let Ok(data) = serde_json::from_str(&data_json) {
            return self.handle_callback(data).iter().map(|c| serde_json::to_string(c).unwrap()).collect();
        }
        Vec::new()
    }
}
