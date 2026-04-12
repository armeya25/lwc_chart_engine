pub mod utils;
pub mod registry;
pub mod moving_averages;
pub mod oscillators;
pub mod volatility;
pub mod volume;

use polars::prelude::*;
use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};

#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;

// Re-exports
pub use registry::{get_indicator_params_schema, get_sub_series_info, is_oscillator};

#[cfg(feature = "python-bridge")]
#[pyfunction]
pub fn py_get_indicator_schemas() -> String {
    registry::get_indicator_schemas()
}

pub fn calculate_batch(config: &IndicatorConfig, df: &DataFrame) -> Result<String, String> {
    let period = config.params.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
    let df_lazy = df.clone().lazy();

    match config.indicator_type {
        IndicatorType::Sma | IndicatorType::Ema | IndicatorType::Dema | IndicatorType::Tema | IndicatorType::Wma | IndicatorType::Hma => {
            moving_averages::calculate_batch(config, df_lazy, period)
        },
        IndicatorType::Rsi | IndicatorType::Macd | IndicatorType::Stochastic | IndicatorType::Cci | IndicatorType::WilliamsR | IndicatorType::Mfi | IndicatorType::Roc => {
            oscillators::calculate_batch(config, df_lazy, period)
        },
        IndicatorType::BollingerBands | IndicatorType::Atr | IndicatorType::KeltnerChannels | IndicatorType::DonchianChannels => {
            volatility::calculate_batch(config, df_lazy, period)
        },
        IndicatorType::Vwap | IndicatorType::Obv | IndicatorType::Adl => {
            volume::calculate_batch(config, df_lazy)
        },
    }
}

pub fn calculate_step(
    config: &IndicatorConfig,
    data: &[Point],
    point: &Point,
    state: Option<IndicatorState>,
) -> Result<(String, Option<IndicatorState>), String> {
    let period = config.params.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
    if data.is_empty() { return Err("No data".to_string()); }

    match config.indicator_type {
        IndicatorType::Sma | IndicatorType::Ema | IndicatorType::Dema | IndicatorType::Tema | IndicatorType::Wma | IndicatorType::Hma => {
            moving_averages::calculate_step(config, data, point, state, period)
        },
        IndicatorType::Rsi | IndicatorType::Macd | IndicatorType::Stochastic | IndicatorType::Cci | IndicatorType::WilliamsR | IndicatorType::Mfi | IndicatorType::Roc => {
            oscillators::calculate_step(config, data, point, state, period)
        },
        IndicatorType::BollingerBands | IndicatorType::Atr | IndicatorType::KeltnerChannels | IndicatorType::DonchianChannels => {
            volatility::calculate_step(config, data, point, state, period)
        },
        IndicatorType::Vwap | IndicatorType::Obv | IndicatorType::Adl => {
            volume::calculate_step(config, data, point, state)
        },
    }
}
