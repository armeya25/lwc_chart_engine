use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartCommand {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

impl ChartCommand {
    pub fn new(action: &str, chart_id: &str) -> Self {
        Self {
            action: action.to_string(),
            id: None,
            line_id: None,
            series_id: None,
            chart_id: Some(chart_id.to_string()),
            name: None,
            options: None,
            data: None,
            extra: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IndicatorType {
    Sma,
    Ema,
    Dema,
    Tema,
    Rsi,
    Macd,
    BollingerBands,
    Atr,
    Stochastic,
    Cci,
    Vwap,
    WilliamsR,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndicatorConfig {
    pub indicator_type: IndicatorType,
    pub target_series_id: String, // Principal series (e.g. MACD line or SMA)
    pub chart_id: String,         // The pane/chart where this indicator belongs
    #[serde(default)]
    pub extra_target_ids: std::collections::HashMap<String, String>, // Extra series (e.g. Signal, Hist, Upper, Lower)
    pub params: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorState {
    Sma,
    Ema(f64),
    Dema { ema1: f64, ema2: f64 },
    Tema { ema1: f64, ema2: f64, ema3: f64 },
    Rsi { avg_gain: f64, avg_loss: f64 },
    Macd { ema_fast: f64, ema_slow: f64, signal: f64 },
    BollingerBands,
    Atr(f64),
    Stochastic { k: f64, d: f64 },
    Cci,
    WilliamsR,
    Vwap { cum_tpv: f64, cum_vol: f64 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}
