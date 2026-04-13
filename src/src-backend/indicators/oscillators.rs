use polars::prelude::*;
use polars::prelude::EWMOptions as EwmOptions;
use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};
use crate::indicators::utils::{df_to_set_data_cmd, df_to_set_data_cmd_colored, single_update_cmd};
use serde_json::json;

pub fn calculate_batch(config: &IndicatorConfig, df_lazy: LazyFrame, period: usize) -> Result<String, String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
        IndicatorType::Rsi => {
            let res_df = df_lazy
                .with_column((col("close") - col("close").shift(lit(1))).alias("diff"))
                .with_columns([
                    when(col("diff").gt(0.0)).then(col("diff")).otherwise(0.0).alias("gain"),
                    when(col("diff").lt(0.0)).then(col("diff").abs()).otherwise(0.0).alias("loss"),
                ])
                .with_columns([
                    col("gain").ewm_mean(EwmOptions { alpha: 1.0 / (period as f64), adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("avg_gain"),
                    col("loss").ewm_mean(EwmOptions { alpha: 1.0 / (period as f64), adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("avg_loss"),
                ])
                .with_column((lit(100.0) - (lit(100.0) / (lit(1.0) + col("avg_gain") / col("avg_loss")))).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Macd => {
            let fast   = config.params.get("fast").and_then(|v| v.as_u64()).unwrap_or(12) as usize;
            let slow   = config.params.get("slow").and_then(|v| v.as_u64()).unwrap_or(26) as usize;
            let signal = config.params.get("signal").and_then(|v| v.as_u64()).unwrap_or(9) as usize;
            let res_df = df_lazy
                .with_columns([
                    col("close").ewm_mean(EwmOptions { alpha: 2.0 / (fast as f64 + 1.0), adjust: false, min_periods: fast, bias: false, ignore_nulls: true }).alias("ema_fast"),
                    col("close").ewm_mean(EwmOptions { alpha: 2.0 / (slow as f64 + 1.0), adjust: false, min_periods: slow, bias: false, ignore_nulls: true }).alias("ema_slow"),
                ])
                .with_column((col("ema_fast") - col("ema_slow")).alias("value"))
                .with_column(col("value").ewm_mean(EwmOptions { alpha: 2.0 / (signal as f64 + 1.0), adjust: false, min_periods: signal, bias: false, ignore_nulls: true }).alias("signal_val"))
                .with_column((col("value") - col("signal_val")).alias("hist_val"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)?];
            if let Some(sid) = config.extra_target_ids.get("signal") {
                let sig_df = res_df.clone().lazy().select([col("time"), col("signal_val").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&sig_df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            if let Some(sid) = config.extra_target_ids.get("hist") {
                let hist_df = res_df.lazy().select([
                    col("time"), col("hist_val").alias("value"),
                    when(col("hist_val").gt_eq(0.0)).then(lit("rgba(38, 166, 154, 0.5)")).otherwise(lit( "rgba(239, 83, 80, 0.5)")).alias("color")
                ]).collect().unwrap();
                commands.push(df_to_set_data_cmd_colored(&hist_df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            Ok(commands.join("\n"))
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
                    when(col("highest_high").eq(col("lowest_low")))
                    .then(lit(50.0))
                    .otherwise(lit(100.0) * (col("close") - col("lowest_low")) / (col("highest_high") - col("lowest_low")))
                    .alias("raw_k")
                )
                .with_column(
                    col("raw_k").rolling_mean(RollingOptionsFixedWindow { window_size: smooth_k, min_periods: smooth_k, ..Default::default() }).alias("value") // %K
                )
                .with_column(
                    col("value").rolling_mean(RollingOptionsFixedWindow { window_size: smooth_d, min_periods: smooth_d, ..Default::default() }).alias("d_line") // %D
                )
                .collect().map_err(|e: PolarsError| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)?];
            if let Some(sid) = config.extra_target_ids.get("d") {
                let d_df = res_df.lazy().select([col("time"), col("d_line").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&d_df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            Ok(commands.join("\n"))
        },
        IndicatorType::Cci => {
            let res_df = df_lazy
                .with_column(((col("high") + col("low") + col("close")) / lit(3.0)).alias("tp"))
                .with_columns([
                    col("tp").rolling_mean(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("sma_tp"),
                    col("tp").rolling_std(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("std_tp"),
                ])
                .with_column(
                    when(col("std_tp").eq(0.0))
                    .then(lit(0.0))
                    .otherwise((col("tp") - col("sma_tp")) / (lit(0.015) * col("std_tp")))
                    .alias("value")
                )
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::WilliamsR => {
            let res_df = df_lazy
                .with_columns([
                    col("high").rolling_max(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("hh"),
                    col("low").rolling_min(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("ll"),
                ])
                .with_column(
                    when(col("hh").eq(col("ll")))
                    .then(lit(-50.0))
                    .otherwise(lit(-100.0) * (col("hh") - col("close")) / (col("hh") - col("ll")))
                    .alias("value")
                )
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Roc => {
            let res_df = df_lazy
                .with_column(((col("close") - col("close").shift(lit(period as i64))) / col("close").shift(lit(period as i64)) * lit(100.0)).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Mfi => {
            let res_df = df_lazy
                .with_column(((col("high") + col("low") + col("close")) / lit(3.0)).alias("tp"))
                .with_column((col("tp") * col("volume")).alias("rmf"))
                .with_column((col("tp") - col("tp").shift(lit(1))).alias("tp_diff"))
                .with_columns([
                    when(col("tp_diff").gt(0.0)).then(col("rmf")).otherwise(0.0).alias("pmf"),
                    when(col("tp_diff").lt(0.0)).then(col("rmf")).otherwise(0.0).alias("nmf"),
                ])
                .with_columns([
                    col("pmf").rolling_sum(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("pmf_sum"),
                    col("nmf").rolling_sum(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("nmf_sum"),
                ])
                .with_column((lit(100.0) - (lit(100.0) / (lit(1.0) + col("pmf_sum") / col("nmf_sum")))).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        _ => Err(format!("Unsupported oscillator: {:?}", config.indicator_type)),
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
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Rsi { avg_gain: new_gain, avg_loss: new_loss })))
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
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, macd_val, &config.chart_id, Some(&config.target_series_id), ind_type_str)];
            if let Some(sid) = config.extra_target_ids.get("signal") { cmds.push(single_update_cmd(sid, point.time, sig, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            if let Some(sid) = config.extra_target_ids.get("hist") {
                let color = if hist_val >= 0.0 { "rgba(38, 166, 154, 0.5)" } else { "rgba(239, 83, 80, 0.5)" };
                cmds.push(serde_json::to_string(&json!({"action":"update_series_data","chartId":&config.chart_id,"seriesId":sid,"indicator":&config.target_series_id,"indicatorType": ind_type_str, "data":{"time":point.time,"value":hist_val,"color":color}})).unwrap());
            }
            Ok((cmds.join("\n"), Some(IndicatorState::Macd { ema_fast: ef, ema_slow: es, signal: sig })))
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
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, k, &config.chart_id, Some(&config.target_series_id), ind_type_str)];
            if let Some(sid) = config.extra_target_ids.get("d") { cmds.push(single_update_cmd(sid, point.time, d, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            Ok((cmds.join("\n"), Some(IndicatorState::Stochastic { k, d })))
        },
        IndicatorType::Cci => {
            if data.len() < period { return Err("Insufficient CCI data".to_string()); }
            let tps: Vec<f64> = data.iter().rev().take(period).map(|p| (p.high + p.low + p.close) / 3.0).collect();
            let tp = (point.high + point.low + point.close) / 3.0;
            let mean_tp = tps.iter().sum::<f64>() / period as f64;
            let mad = tps.iter().map(|&v| (v - mean_tp).abs()).sum::<f64>() / period as f64;
            let value = if mad < 1e-10 { 0.0 } else { (tp - mean_tp) / (0.015 * mad) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Cci)))
        },
        IndicatorType::WilliamsR => {
            if data.len() < period { return Err("Insufficient WR data".to_string()); }
            let window: Vec<&Point> = data.iter().rev().take(period).collect();
            let hh = window.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
            let value = if (hh - ll).abs() < 1e-10 { -50.0 } else { -100.0 * (hh - point.close) / (hh - ll) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::WilliamsR)))
        },
        IndicatorType::Roc => {
            if data.len() <= period { return Err("Insufficient ROC data".to_string()); }
            let past_close = data[data.len() - 1 - period].close;
            let value = (point.close - past_close) / past_close * 100.0;
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Roc)))
        },
        IndicatorType::Mfi => {
            if data.len() < period + 1 { return Err("Insufficient MFI data".to_string()); }
            let mut pmf_sum = 0.0;
            let mut nmf_sum = 0.0;
            for i in (data.len() - period)..data.len() {
                let curr = &data[i];
                let prev = &data[i-1];
                let tp_curr = (curr.high + curr.low + curr.close) / 3.0;
                let tp_prev = (prev.high + prev.low + prev.close) / 3.0;
                let rmf = tp_curr * curr.volume.unwrap_or(1.0);
                if tp_curr > tp_prev { pmf_sum += rmf; }
                else if tp_curr < tp_prev { nmf_sum += rmf; }
            }
            let value = if nmf_sum == 0.0 { 100.0 } else { 100.0 - (100.0 / (1.0 + pmf_sum / nmf_sum)) };
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Mfi)))
        },
        _ => Err(format!("Unsupported oscillator: {:?}", config.indicator_type)),
    }
}
