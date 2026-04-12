use polars::prelude::*;
use polars::prelude::EWMOptions as EwmOptions;
use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};
use crate::indicators::utils::{df_to_set_data_cmd, single_update_cmd};

pub fn calculate_batch(config: &IndicatorConfig, df_lazy: LazyFrame, period: usize) -> Result<String, String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
        IndicatorType::Sma => {
            let res_df = df_lazy
                .with_column(col("close").rolling_mean(RollingOptionsFixedWindow {
                    window_size: period, min_periods: period, ..Default::default()
                }).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Ema => {
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EwmOptions {
                    alpha: 2.0 / (period as f64 + 1.0), adjust: false,
                    min_periods: period, bias: false, ignore_nulls: true
                }).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Dema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("ema1"))
                .with_column(col("ema1").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("ema2"))
                .with_column((lit(2.0) * col("ema1") - col("ema2")).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Tema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("ema1"))
                .with_column(col("ema1").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("ema2"))
                .with_column(col("ema2").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("ema3"))
                .with_column((lit(3.0) * col("ema1") - lit(3.0) * col("ema2") + col("ema3")).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Wma => {
            let weight_sum = (period * (period + 1)) as f64 / 2.0;
            let mut wma_expr = lit(0.0);
            // Latest point (shift 0) gets highest weight 'period'
            for i in 0..period {
                wma_expr = wma_expr + col("close").shift(lit(i as i64)) * lit((period - i) as f64);
            }
            let res_df = df_lazy
                .with_column((wma_expr / lit(weight_sum)).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Hma => {
            let n2 = period / 2;
            let sqrt_n = (period as f64).sqrt() as usize;
            
            // 1. WMA(n/2)
            let mut wma_n2_expr = lit(0.0);
            let ws_n2 = (n2 * (n2 + 1)) as f64 / 2.0;
            for i in 0..n2 { wma_n2_expr = wma_n2_expr + col("close").shift(lit(i as i64)) * lit((n2 - i) as f64); }
            
            // 2. WMA(n)
            let mut wma_n_expr = lit(0.0);
            let ws_n = (period * (period + 1)) as f64 / 2.0;
            for i in 0..period { wma_n_expr = wma_n_expr + col("close").shift(lit(i as i64)) * lit((period - i) as f64); }
            
            // 3. raw_hma = 2*WMA(n/2) - WMA(n)
            let res_df = df_lazy
                .with_column((lit(2.0) * (wma_n2_expr / lit(ws_n2)) - (wma_n_expr / lit(ws_n))).alias("raw_hma"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            
            // 4. hma = WMA(raw_hma, sqrt(n))
            let mut hma_expr = lit(0.0);
            let ws_sqrt = (sqrt_n * (sqrt_n + 1)) as f64 / 2.0;
            for i in 0..sqrt_n { hma_expr = hma_expr + col("raw_hma").shift(lit(i as i64)) * lit((sqrt_n - i) as f64); }
            
            let final_df = res_df.lazy()
                .with_column((hma_expr / lit(ws_sqrt)).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&final_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        _ => Err(format!("Unsupported moving average: {:?}", config.indicator_type)),
    }
}

pub fn calculate_step(
    config: &IndicatorConfig,
    data: &[Point],
    point: &Point,
    state: Option<IndicatorState>,
    period: usize,
) -> Result<(String, Option<IndicatorState>), String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
        IndicatorType::Sma => {
            if data.len() < period { return Err("Insufficient data".to_string()); }
            let value = data.iter().rev().take(period).map(|p| p.close).sum::<f64>() / period as f64;
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Sma)))
        },
        IndicatorType::Ema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let new_ema = match state {
                Some(IndicatorState::Ema(prev)) => point.close * alpha + prev * (1.0 - alpha),
                _ => { let mut e = data[0].close; for p in data { e = p.close * alpha + e * (1.0 - alpha); } e }
            };
            Ok((single_update_cmd(&config.target_series_id, point.time, new_ema, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Ema(new_ema))))
        },
        IndicatorType::Dema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let (ema1, ema2) = match state {
                Some(IndicatorState::Dema { ema1: pe1, ema2: pe2 }) => {
                    let ne1 = point.close * alpha + pe1 * (1.0 - alpha);
                    let ne2 = ne1 * alpha + pe2 * (1.0 - alpha);
                    (ne1, ne2)
                },
                _ => {
                    let mut e1 = data[0].close; let mut e2 = data[0].close;
                    for p in data { e1 = p.close * alpha + e1 * (1.0 - alpha); e2 = e1 * alpha + e2 * (1.0 - alpha); }
                    (e1, e2)
                }
            };
            let value = 2.0 * ema1 - ema2;
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Dema { ema1, ema2 })))
        },
        IndicatorType::Tema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let (ema1, ema2, ema3) = match state {
                Some(IndicatorState::Tema { ema1: pe1, ema2: pe2, ema3: pe3 }) => {
                    let ne1 = point.close * alpha + pe1 * (1.0 - alpha);
                    let ne2 = ne1 * alpha + pe2 * (1.0 - alpha);
                    let ne3 = ne2 * alpha + pe3 * (1.0 - alpha);
                    (ne1, ne2, ne3)
                },
                _ => {
                    let mut e1 = data[0].close; let mut e2 = data[0].close; let mut e3 = data[0].close;
                    for p in data {
                        e1 = p.close * alpha + e1 * (1.0 - alpha);
                        e2 = e1 * alpha + e2 * (1.0 - alpha);
                        e3 = e2 * alpha + e3 * (1.0 - alpha);
                    }
                    (e1, e2, e3)
                }
            };
            let value = 3.0 * ema1 - 3.0 * ema2 + ema3;
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Tema { ema1, ema2, ema3 })))
        },
        IndicatorType::Wma => {
            if data.len() < period { return Err("Insufficient data".to_string()); }
            let weight_sum = (period * (period + 1)) as f64 / 2.0;
            let mut val = 0.0;
            for (i, p) in data.iter().rev().take(period).enumerate() {
                val += p.close * (period - i) as f64;
            }
            Ok((single_update_cmd(&config.target_series_id, point.time, val / weight_sum, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Wma)))
        },
        IndicatorType::Hma => {
            let n2 = period / 2;
            let sqrt_n = (period as f64).sqrt() as usize;
            if data.len() < period + sqrt_n { return Err("Insufficient data".to_string()); }
            
            // We need multiple WMAs to compute HMA at one point. 
            // The step is inefficient (O(period*sqrt_n)) but correct.
            let mut raw_hmas = Vec::with_capacity(sqrt_n);
            for offset in 0..sqrt_n {
                let sub_data = &data[..(data.len() - offset)];
                let wma_n2 = {
                    let ws = (n2 * (n2 + 1)) as f64 / 2.0;
                    sub_data.iter().rev().take(n2).enumerate().map(|(i, p)| p.close * (n2 - i) as f64).sum::<f64>() / ws
                };
                let wma_n = {
                    let ws = (period * (period + 1)) as f64 / 2.0;
                    sub_data.iter().rev().take(period).enumerate().map(|(i, p)| p.close * (period - i) as f64).sum::<f64>() / ws
                };
                raw_hmas.push(2.0 * wma_n2 - wma_n);
            }
            
            let ws_sqrt = (sqrt_n * (sqrt_n + 1)) as f64 / 2.0;
            let hma_val = raw_hmas.iter().enumerate().map(|(i, v)| v * (sqrt_n - i) as f64).sum::<f64>() / ws_sqrt;
            
            Ok((single_update_cmd(&config.target_series_id, point.time, hma_val, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Hma)))
        },
        _ => Err(format!("Unsupported moving average: {:?}", config.indicator_type)),
    }
}
