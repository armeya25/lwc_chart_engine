use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};
use polars::prelude::*;
use serde_json::json;
use polars::series::ops::NullBehavior;
use polars::prelude::RollingOptionsFixedWindow;

// ── Batch (initial load) ─────────────────────────────────────────────────────

pub fn calculate_batch(config: &IndicatorConfig, df: &DataFrame) -> Result<String, String> {
    let period = config.params.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
    let df_lazy = df.clone().lazy();

    match config.indicator_type {
        IndicatorType::Sma => {
            let res_df = df_lazy
                .with_column(col("close").rolling_mean(RollingOptionsFixedWindow {
                    window_size: period, min_periods: period, ..Default::default()
                }).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Ema => {
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EWMOptions {
                    alpha: 2.0 / (period as f64 + 1.0), adjust: false, bias: false,
                    min_periods: period, ignore_nulls: true,
                }).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Rsi => {
            let res_df = df_lazy
                .with_column(col("close").diff(lit(1), NullBehavior::Ignore).alias("diff"))
                .with_columns([
                    when(col("diff").gt(0.0)).then(col("diff")).otherwise(0.0).alias("gain"),
                    when(col("diff").lt(0.0)).then(col("diff").abs()).otherwise(0.0).alias("loss"),
                ])
                .with_columns([
                    col("gain").ewm_mean(EWMOptions { alpha: 1.0 / (period as f64), adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("avg_gain"),
                    col("loss").ewm_mean(EWMOptions { alpha: 1.0 / (period as f64), adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("avg_loss"),
                ])
                .with_column((lit(100.0) - (lit(100.0) / (lit(1.0) + col("avg_gain") / col("avg_loss")))).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Macd => {
            let fast   = config.params.get("fast").and_then(|v| v.as_u64()).unwrap_or(12) as usize;
            let slow   = config.params.get("slow").and_then(|v| v.as_u64()).unwrap_or(26) as usize;
            let signal = config.params.get("signal").and_then(|v| v.as_u64()).unwrap_or(9) as usize;
            let res_df = df_lazy
                .with_columns([
                    col("close").ewm_mean(EWMOptions { alpha: 2.0 / (fast as f64 + 1.0), adjust: false, bias: false, min_periods: fast, ignore_nulls: true }).alias("ema_fast"),
                    col("close").ewm_mean(EWMOptions { alpha: 2.0 / (slow as f64 + 1.0), adjust: false, bias: false, min_periods: slow, ignore_nulls: true }).alias("ema_slow"),
                ])
                .with_column((col("ema_fast") - col("ema_slow")).alias("value"))
                .with_column(col("value").ewm_mean(EWMOptions { alpha: 2.0 / (signal as f64 + 1.0), adjust: false, bias: false, min_periods: signal, ignore_nulls: true }).alias("signal_val"))
                .with_column((col("value") - col("signal_val")).alias("hist_val"))
                .collect().map_err(|e| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)?];
            if let Some(sid) = config.extra_target_ids.get("signal") {
                let sig_df = res_df.clone().lazy().select([col("time"), col("signal_val").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&sig_df, sid, &config.chart_id)?);
            }
            if let Some(sid) = config.extra_target_ids.get("hist") {
                let hist_df = res_df.lazy().select([
                    col("time"), col("hist_val").alias("value"),
                    when(col("hist_val").gt_eq(0.0)).then(lit("rgba(38, 166, 154, 0.5)")).otherwise(lit("rgba(239, 83, 80, 0.5)")).alias("color")
                ]).collect().unwrap();
                commands.push(df_to_set_data_cmd_colored(&hist_df, sid, &config.chart_id)?);
            }
            Ok(commands.join("\n"))
        },

        IndicatorType::BollingerBands => {
            let std_dev = config.params.get("std_dev").and_then(|v| v.as_f64()).unwrap_or(2.0);
            let res_df = df_lazy
                .with_columns([
                    col("close").rolling_mean(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("value"),
                    col("close").rolling_std(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("std"),
                ])
                .with_columns([
                    (col("value") + lit(std_dev) * col("std")).alias("upper"),
                    (col("value") - lit(std_dev) * col("std")).alias("lower"),
                ])
                .collect().map_err(|e| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)?];
            if let Some(sid) = config.extra_target_ids.get("upper") {
                let up_df = res_df.clone().lazy().select([col("time"), col("upper").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&up_df, sid, &config.chart_id)?);
            }
            if let Some(sid) = config.extra_target_ids.get("lower") {
                let low_df = res_df.lazy().select([col("time"), col("lower").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&low_df, sid, &config.chart_id)?);
            }
            Ok(commands.join("\n"))
        },

        IndicatorType::Atr => {
            // Average True Range — True Range = max(H-L, |H-prev_close|, |L-prev_close|)
            let res_df = df_lazy
                .with_column(col("close").shift(lit(1)).alias("prev_close"))
                .with_columns([
                    (col("high") - col("low")).alias("hl"),
                    (col("high") - col("prev_close")).abs().alias("hpc"),
                    (col("low")  - col("prev_close")).abs().alias("lpc"),
                ])
                .with_column(
                    // TR = max of three components via nested when/otherwise
                    when(col("hl").gt_eq(col("hpc")).and(col("hl").gt_eq(col("lpc"))))
                        .then(col("hl"))
                        .when(col("hpc").gt_eq(col("lpc")))
                        .then(col("hpc"))
                        .otherwise(col("lpc"))
                        .alias("tr")
                )
                .with_column(col("tr").rolling_mean(RollingOptionsFixedWindow {
                    window_size: period, min_periods: period, ..Default::default()
                }).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Stochastic => {
            let smooth_k = config.params.get("smooth_k").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let smooth_d = config.params.get("smooth_d").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let res_df = df_lazy
                .with_columns([
                    col("low").rolling_min(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("lowest_low"),
                    col("high").rolling_max(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("highest_high"),
                ])
                .with_column(
                    (lit(100.0) * (col("close") - col("lowest_low")) / (col("highest_high") - col("lowest_low"))).alias("raw_k")
                )
                .with_column(
                    col("raw_k").rolling_mean(RollingOptionsFixedWindow { window_size: smooth_k, min_periods: smooth_k, ..Default::default() }).alias("value") // %K
                )
                .with_column(
                    col("value").rolling_mean(RollingOptionsFixedWindow { window_size: smooth_d, min_periods: smooth_d, ..Default::default() }).alias("d_line") // %D
                )
                .collect().map_err(|e| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)?];
            if let Some(sid) = config.extra_target_ids.get("d") {
                let d_df = res_df.lazy().select([col("time"), col("d_line").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&d_df, sid, &config.chart_id)?);
            }
            Ok(commands.join("\n"))
        },

        IndicatorType::Cci => {
            // Commodity Channel Index
            let res_df = df_lazy
                .with_column(((col("high") + col("low") + col("close")) / lit(3.0)).alias("tp"))
                .with_columns([
                    col("tp").rolling_mean(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("sma_tp"),
                    col("tp").rolling_std(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("std_tp"),
                ])
                .with_column(((col("tp") - col("sma_tp")) / (lit(0.015) * col("std_tp"))).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Vwap => {
            // Volume-weighted average price (cumulative, resets per session)
            let res_df = df_lazy
                .with_column(((col("high") + col("low") + col("close")) / lit(3.0)).alias("tp"))
                .with_columns([
                    (col("tp") * col("volume")).cum_sum(false).alias("cum_tpv"),
                    col("volume").cum_sum(false).alias("cum_vol"),
                ])
                .with_column((col("cum_tpv") / col("cum_vol")).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::WilliamsR => {
            // Williams %R oscillator (-100 to 0)
            let res_df = df_lazy
                .with_columns([
                    col("high").rolling_max(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("hh"),
                    col("low").rolling_min(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("ll"),
                ])
                .with_column((lit(-100.0) * (col("hh") - col("close")) / (col("hh") - col("ll"))).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Dema => {
            // Double EMA = 2*EMA - EMA(EMA)
            let alpha = 2.0 / (period as f64 + 1.0);
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EWMOptions { alpha, adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("ema1"))
                .with_column(col("ema1").ewm_mean(EWMOptions { alpha, adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("ema2"))
                .with_column((lit(2.0) * col("ema1") - col("ema2")).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },

        IndicatorType::Tema => {
            // Triple EMA = 3*EMA - 3*EMA(EMA) + EMA(EMA(EMA))
            let alpha = 2.0 / (period as f64 + 1.0);
            let res_df = df_lazy
                .with_column(col("close").ewm_mean(EWMOptions { alpha, adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("ema1"))
                .with_column(col("ema1").ewm_mean(EWMOptions { alpha, adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("ema2"))
                .with_column(col("ema2").ewm_mean(EWMOptions { alpha, adjust: false, bias: false, min_periods: period, ignore_nulls: true }).alias("ema3"))
                .with_column((lit(3.0) * col("ema1") - lit(3.0) * col("ema2") + col("ema3")).alias("value"))
                .collect().map_err(|e| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id)
        },
    }
}

// ── Real-time step (O(1) per tick) ──────────────────────────────────────────

pub fn calculate_step(
    config: &IndicatorConfig,
    data: &[Point],
    point: &Point,
    state: Option<IndicatorState>,
) -> Result<(String, Option<IndicatorState>), String> {
    let period = config.params.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
    if data.is_empty() { return Err("No data".to_string()); }

    match config.indicator_type {
        IndicatorType::Sma => {
            if data.len() < period { return Err("Insufficient data".to_string()); }
            let value = data.iter().rev().take(period).map(|p| p.close).sum::<f64>() / period as f64;
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Sma)))
        },

        IndicatorType::Ema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            let new_ema = match state {
                Some(IndicatorState::Ema(prev)) => point.close * alpha + prev * (1.0 - alpha),
                _ => { let mut e = data[0].close; for p in data { e = p.close * alpha + e * (1.0 - alpha); } e }
            };
            Ok((single_update_cmd(&config.target_series_id, point.time, new_ema, &config.chart_id), Some(IndicatorState::Ema(new_ema))))
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
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Dema { ema1, ema2 })))
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
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Tema { ema1, ema2, ema3 })))
        },

        IndicatorType::Rsi => {
            if data.len() < period + 1 { return Err("Insufficient RSI data".to_string()); }
            let alpha = 1.0 / (period as f64);
            let (new_gain, new_loss) = match state {
                Some(IndicatorState::Rsi { avg_gain, avg_loss }) => {
                    let diff = point.close - data[data.len()-2].close;
                    let g = if diff > 0.0 { diff } else { 0.0 };
                    let l = if diff < 0.0 { diff.abs() } else { 0.0 };
                    (g * alpha + avg_gain * (1.0 - alpha), l * alpha + avg_loss * (1.0 - alpha))
                },
                _ => {
                    let mut ag = 0.0f64; let mut al = 0.0f64;
                    for i in 1..data.len() {
                        let d = data[i].close - data[i-1].close;
                        let g = if d > 0.0 { d } else { 0.0 };
                        let l = if d < 0.0 { d.abs() } else { 0.0 };
                        ag = g * alpha + ag * (1.0 - alpha);
                        al = l * alpha + al * (1.0 - alpha);
                    }
                    (ag, al)
                }
            };
            let value = if new_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + new_gain / new_loss) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Rsi { avg_gain: new_gain, avg_loss: new_loss })))
        },

        IndicatorType::Macd => {
            let fast   = config.params.get("fast").and_then(|v| v.as_u64()).unwrap_or(12) as usize;
            let slow   = config.params.get("slow").and_then(|v| v.as_u64()).unwrap_or(26) as usize;
            let signal = config.params.get("signal").and_then(|v| v.as_u64()).unwrap_or(9) as usize;
            let af = 2.0 / (fast as f64 + 1.0);
            let as_ = 2.0 / (slow as f64 + 1.0);
            let asig = 2.0 / (signal as f64 + 1.0);
            let (ef, es, sig) = match state {
                Some(IndicatorState::Macd { ema_fast, ema_slow, signal: ps }) => {
                    let nf = point.close * af + ema_fast * (1.0 - af);
                    let ns = point.close * as_ + ema_slow * (1.0 - as_);
                    let nm = nf - ns;
                    (nf, ns, nm * asig + ps * (1.0 - asig))
                },
                _ => {
                    if data.len() < slow + signal { return Err("Warm-up MACD".to_string()); }
                    let mut ef = data[0].close; let mut es = data[0].close; let mut hist = vec![];
                    for p in data { ef = p.close * af + ef * (1.0 - af); es = p.close * as_ + es * (1.0 - as_); hist.push(ef - es); }
                    let mut s = hist[0]; for v in &hist { s = v * asig + s * (1.0 - asig); }
                    (ef, es, s)
                }
            };
            let macd_val = ef - es; let hist_val = macd_val - sig;
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, macd_val, &config.chart_id)];
            if let Some(sid) = config.extra_target_ids.get("signal") { cmds.push(single_update_cmd(sid, point.time, sig, &config.chart_id)); }
            if let Some(sid) = config.extra_target_ids.get("hist") {
                let color = if hist_val >= 0.0 { "rgba(38, 166, 154, 0.5)" } else { "rgba(239, 83, 80, 0.5)" };
                cmds.push(serde_json::to_string(&json!({"action":"update_series_data","chartId":&config.chart_id,"seriesId":sid,"data":{"time":point.time,"value":hist_val,"color":color}})).unwrap());
            }
            Ok((cmds.join("\n"), Some(IndicatorState::Macd { ema_fast: ef, ema_slow: es, signal: sig })))
        },

        IndicatorType::BollingerBands => {
            let std_mult = config.params.get("std_dev").and_then(|v| v.as_f64()).unwrap_or(2.0);
            if data.len() < period { return Err("Insufficient data".to_string()); }
            let closes: Vec<f64> = data.iter().rev().take(period).map(|p| p.close).collect();
            let mean = closes.iter().sum::<f64>() / period as f64;
            let std = (closes.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / period as f64).sqrt();
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, mean, &config.chart_id)];
            if let Some(sid) = config.extra_target_ids.get("upper") { cmds.push(single_update_cmd(sid, point.time, mean + std_mult * std, &config.chart_id)); }
            if let Some(sid) = config.extra_target_ids.get("lower") { cmds.push(single_update_cmd(sid, point.time, mean - std_mult * std, &config.chart_id)); }
            Ok((cmds.join("\n"), Some(IndicatorState::BollingerBands)))
        },

        IndicatorType::Atr => {
            if data.len() < 2 { return Err("Insufficient ATR data".to_string()); }
            let prev_close = data[data.len()-2].close;
            let tr = (point.high - point.low)
                .max((point.high - prev_close).abs())
                .max((point.low - prev_close).abs());
            let new_atr = match state {
                Some(IndicatorState::Atr(prev)) => (prev * (period as f64 - 1.0) + tr) / period as f64,
                _ => data.windows(2).rev().take(period).map(|w| {
                    let pc = w[0].close;
                    (w[1].high - w[1].low).max((w[1].high - pc).abs()).max((w[1].low - pc).abs())
                }).sum::<f64>() / period as f64,
            };
            Ok((single_update_cmd(&config.target_series_id, point.time, new_atr, &config.chart_id), Some(IndicatorState::Atr(new_atr))))
        },

        IndicatorType::Stochastic => {
            if data.len() < period { return Err("Insufficient Stochastic data".to_string()); }
            let smooth_k = config.params.get("smooth_k").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let window: Vec<&Point> = data.iter().rev().take(period).collect();
            let ll = window.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
            let hh = window.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
            let raw_k = if (hh - ll).abs() < 1e-10 { 50.0 } else { 100.0 * (point.close - ll) / (hh - ll) };
            let k = match state {
                Some(IndicatorState::Stochastic { k: pk, .. }) => (raw_k + pk * (smooth_k as f64 - 1.0)) / smooth_k as f64,
                _ => raw_k,
            };
            let d = match state {
                Some(IndicatorState::Stochastic { d: pd, k: pk }) => (pk + pd * 2.0) / 3.0,
                _ => k,
            };
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, k, &config.chart_id)];
            if let Some(sid) = config.extra_target_ids.get("d") { cmds.push(single_update_cmd(sid, point.time, d, &config.chart_id)); }
            Ok((cmds.join("\n"), Some(IndicatorState::Stochastic { k, d })))
        },

        IndicatorType::Cci => {
            if data.len() < period { return Err("Insufficient CCI data".to_string()); }
            let tps: Vec<f64> = data.iter().rev().take(period).map(|p| (p.high + p.low + p.close) / 3.0).collect();
            let tp = (point.high + point.low + point.close) / 3.0;
            let mean_tp = tps.iter().sum::<f64>() / period as f64;
            let mad = tps.iter().map(|&v| (v - mean_tp).abs()).sum::<f64>() / period as f64;
            let value = if mad < 1e-10 { 0.0 } else { (tp - mean_tp) / (0.015 * mad) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Cci)))
        },

        IndicatorType::WilliamsR => {
            if data.len() < period { return Err("Insufficient WR data".to_string()); }
            let window: Vec<&Point> = data.iter().rev().take(period).collect();
            let hh = window.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
            let value = if (hh - ll).abs() < 1e-10 { -50.0 } else { -100.0 * (hh - point.close) / (hh - ll) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::WilliamsR)))
        },

        IndicatorType::Vwap => {
            let (cum_tpv, cum_vol) = match state {
                Some(IndicatorState::Vwap { cum_tpv, cum_vol }) => (cum_tpv, cum_vol),
                _ => data.iter().fold((0.0f64, 0.0f64), |(tpv, vol), p| {
                    let tp = (p.high + p.low + p.close) / 3.0;
                    let v = p.volume.unwrap_or(1.0);
                    (tpv + tp * v, vol + v)
                }),
            };
            let tp = (point.high + point.low + point.close) / 3.0;
            let vol = point.volume.unwrap_or(1.0);
            let new_tpv = cum_tpv + tp * vol;
            let new_vol = cum_vol + vol;
            let value = if new_vol == 0.0 { tp } else { new_tpv / new_vol };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id), Some(IndicatorState::Vwap { cum_tpv: new_tpv, cum_vol: new_vol })))
        },
    }
}

pub fn get_all_metadata_json() -> String {
    json!({
        "sma": {"period": {"type": "int", "min": 1, "max": 200, "default": 20}},
        "ema": {"period": {"type": "int", "min": 1, "max": 200, "default": 20}},
        "rsi": {"period": {"type": "int", "min": 1, "max": 100, "default": 14}},
        "macd": {
            "fast": {"type": "int", "min": 1, "max": 100, "default": 12},
            "slow": {"type": "int", "min": 1, "max": 100, "default": 26},
            "signal": {"type": "int", "min": 1, "max": 50, "default": 9}
        },
        "bollingerbands": {
            "period": {"type": "int", "min": 1, "max": 200, "default": 20},
            "std_dev": {"type": "float", "min": 0.1, "max": 10.0, "step": 0.1, "default": 2.0}
        },
        "atr": {"period": {"type": "int", "min": 1, "max": 100, "default": 14}},
        "stochastic": {
            "period": {"type": "int", "min": 1, "max": 100, "default": 14},
            "smooth_k": {"type": "int", "min": 1, "max": 50, "default": 3},
            "smooth_d": {"type": "int", "min": 1, "max": 50, "default": 3}
        },
        "cci": {"period": {"type": "int", "min": 1, "max": 100, "default": 20}},
        "vwap": {},
        "williamsr": {"period": {"type": "int", "min": 1, "max": 100, "default": 14}},
        "dema": {"period": {"type": "int", "min": 1, "max": 200, "default": 20}},
        "tema": {"period": {"type": "int", "min": 1, "max": 200, "default": 20}}
    }).to_string()
}

pub fn get_indicator_params_schema(ind_type: IndicatorType) -> serde_json::Value {
    let all_json = get_all_metadata_json();
    let all: serde_json::Value = serde_json::from_str(&all_json).unwrap();
    let key = match ind_type {
        IndicatorType::Sma => "sma",
        IndicatorType::Ema => "ema",
        IndicatorType::Rsi => "rsi",
        IndicatorType::Macd => "macd",
        IndicatorType::BollingerBands => "bollingerbands",
        IndicatorType::Atr => "atr",
        IndicatorType::Stochastic => "stochastic",
        IndicatorType::Cci => "cci",
        IndicatorType::Vwap => "vwap",
        IndicatorType::WilliamsR => "williamsr",
        IndicatorType::Dema => "dema",
        IndicatorType::Tema => "tema",
    };
    all.get(key).cloned().unwrap_or(serde_json::json!({}))
}

pub fn get_sub_series_info(ind_type: IndicatorType) -> Vec<(&'static str, &'static str, &'static str)> {
    match ind_type {
        IndicatorType::Macd => vec![
            ("main", "MACD", "#2196F3"),
            ("signal", "Signal", "#FF5252"),
            ("hist", "Histogram", "rgba(31, 150, 243, 0.5)")
        ],
        IndicatorType::BollingerBands => vec![
            ("main", "Mid", "rgba(31, 150, 243, 0.4)"),
            ("upper", "Upper", "rgba(31, 150, 243, 0.4)"),
            ("lower", "Lower", "rgba(31, 150, 243, 0.4)")
        ],
        IndicatorType::Stochastic => vec![
            ("main", "%K", "#1565C0"),
            ("d", "%D", "#E53935")
        ],
        _ => vec![("main", "Value", "#2962FF")],
    }
}

pub fn is_oscillator(ind_type: IndicatorType) -> bool {
    match ind_type {
        IndicatorType::Rsi | IndicatorType::Macd | IndicatorType::Atr | 
        IndicatorType::Stochastic | IndicatorType::Cci | IndicatorType::WilliamsR => true,
        _ => false,
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn single_update_cmd(sid: &str, time: i64, value: f64, chart_id: &str) -> String {
    serde_json::to_string(&json!({
        "action": "update_series_data", "chartId": chart_id,
        "seriesId": sid, "data": { "time": time, "value": value }
    })).unwrap()
}

fn df_to_set_data_cmd(df: &DataFrame, target_id: &str, chart_id: &str) -> Result<String, String> {
    let times  = df.column("time").unwrap().i64().unwrap();
    let values = df.column("value").unwrap().f64().unwrap();
    let data: Vec<_> = (0..df.height()).filter_map(|i| {
        let v = values.get(i)?;
        if v.is_nan() { return None; }
        Some(json!({ "time": times.get(i).unwrap(), "value": v }))
    }).collect();
    Ok(serde_json::to_string(&json!({ "action": "set_series_data", "chartId": chart_id, "seriesId": target_id, "data": data })).unwrap())
}

fn df_to_set_data_cmd_colored(df: &DataFrame, target_id: &str, chart_id: &str) -> Result<String, String> {
    let times  = df.column("time").unwrap().i64().unwrap();
    let values = df.column("value").unwrap().f64().unwrap();
    let colors = df.column("color").unwrap().str().unwrap();
    let data: Vec<_> = (0..df.height()).filter_map(|i| {
        let v = values.get(i)?;
        if v.is_nan() { return None; }
        Some(json!({ "time": times.get(i).unwrap(), "value": v, "color": colors.get(i).unwrap() }))
    }).collect();
    Ok(serde_json::to_string(&json!({ "action": "set_series_data", "chartId": chart_id, "seriesId": target_id, "data": data })).unwrap())
}
