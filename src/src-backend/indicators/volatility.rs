use polars::prelude::*;
use polars::prelude::EWMOptions as EwmOptions;
use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};
use crate::indicators::utils::{df_to_set_data_cmd, single_update_cmd};

pub fn calculate_batch(config: &IndicatorConfig, df_lazy: LazyFrame, period: usize) -> Result<String, String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
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
                .collect().map_err(|e: PolarsError| e.to_string())?;
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)?];
            if let Some(sid) = config.extra_target_ids.get("upper") {
                let up_df = res_df.clone().lazy().select([col("time"), col("upper").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&up_df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            if let Some(sid) = config.extra_target_ids.get("lower") {
                let low_df = res_df.lazy().select([col("time"), col("lower").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&low_df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            Ok(commands.join("\n"))
        },
        IndicatorType::Atr => {
            let res_df = df_lazy
                .with_column(col("close").shift(lit(1)).alias("prev_close"))
                .with_columns([
                    (col("high") - col("low")).alias("hl"),
                    (col("high") - col("prev_close")).abs().alias("hpc"),
                    (col("low")  - col("prev_close")).abs().alias("lpc"),
                ])
                .with_column(
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
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::KeltnerChannels => {
            let mult = config.params.get("multiplier").and_then(|v| v.as_f64()).unwrap_or(2.0);
            let alpha = 2.0 / (period as f64 + 1.0);
            let res_df = df_lazy
                .with_column(col("close").shift(lit(1)).alias("prev_close"))
                .with_column(
                    when((col("high") - col("low")).gt_eq((col("high") - col("prev_close")).abs())
                        .and((col("high") - col("low")).gt_eq((col("low") - col("prev_close")).abs())))
                        .then(col("high") - col("low"))
                        .when((col("high") - col("prev_close")).abs().gt_eq((col("low") - col("prev_close")).abs()))
                        .then((col("high") - col("prev_close")).abs())
                        .otherwise((col("low") - col("prev_close")).abs())
                        .alias("tr")
                )
                .with_columns([
                    col("close").ewm_mean(EwmOptions { alpha, adjust: false, min_periods: period, bias: false, ignore_nulls: true }).alias("value"),
                    col("tr").rolling_mean(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("atr"),
                ])
                .with_columns([
                    (col("value") + lit(mult) * col("atr")).alias("upper"),
                    (col("value") - lit(mult) * col("atr")).alias("lower"),
                ])
                .collect().map_err(|e: PolarsError| e.to_string())?;
                
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)?];
            if let Some(sid) = config.extra_target_ids.get("upper") {
                let df = res_df.clone().lazy().select([col("time"), col("upper").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            if let Some(sid) = config.extra_target_ids.get("lower") {
                let df = res_df.clone().lazy().select([col("time"), col("lower").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            Ok(commands.join("\n"))
        },
        IndicatorType::DonchianChannels => {
            let res_df = df_lazy
                .with_columns([
                    col("high").rolling_max(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("upper"),
                    col("low").rolling_min(RollingOptionsFixedWindow { window_size: period, min_periods: period, ..Default::default() }).alias("lower"),
                ])
                .with_column(((col("upper") + col("lower")) / lit(2.0)).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
                
            let mut commands = vec![df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)?];
            if let Some(sid) = config.extra_target_ids.get("upper") {
                let df = res_df.clone().lazy().select([col("time"), col("upper").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            if let Some(sid) = config.extra_target_ids.get("lower") {
                let df = res_df.clone().lazy().select([col("time"), col("lower").alias("value")]).collect().unwrap();
                commands.push(df_to_set_data_cmd(&df, sid, &config.chart_id, Some(&config.target_series_id), ind_type_str)?);
            }
            Ok(commands.join("\n"))
        },
        _ => Err(format!("Unsupported volatility indicator: {:?}", config.indicator_type)),
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
        IndicatorType::BollingerBands => {
            let std_mult = config.params.get("std_dev").and_then(|v| v.as_f64()).unwrap_or(2.0);
            if data.len() < period { return Err("Insufficient data".to_string()); }
            let closes: Vec<f64> = data.iter().rev().take(period).map(|p| p.close).collect();
            let mean = closes.iter().sum::<f64>() / period as f64;
            let std = (closes.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / period as f64).sqrt();
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, mean, &config.chart_id, Some(&config.target_series_id), ind_type_str)];
            if let Some(sid) = config.extra_target_ids.get("upper") { cmds.push(single_update_cmd(sid, point.time, mean + std_mult * std, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            if let Some(sid) = config.extra_target_ids.get("lower") { cmds.push(single_update_cmd(sid, point.time, mean - std_mult * std, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
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
            Ok((single_update_cmd(&config.target_series_id, point.time, new_atr, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Atr(new_atr))))
        },
        IndicatorType::KeltnerChannels => {
            let mult = config.params.get("multiplier").and_then(|v| v.as_f64()).unwrap_or(2.0);
            if data.len() < period + 1 { return Err("Insufficient Keltner data".to_string()); }
            let alpha = 2.0 / (period as f64 + 1.0);
            let ema = match state {
                Some(IndicatorState::Ema(prev)) => point.close * alpha + prev * (1.0 - alpha),
                _ => { let mut e = data[0].close; for p in data { e = p.close * alpha + e * (1.0 - alpha); } e }
            };
            let trs: Vec<f64> = data.windows(2).rev().take(period).map(|w| {
                let pc = w[0].close;
                (w[1].high - w[1].low).max((w[1].high - pc).abs()).max((w[1].low - pc).abs())
            }).collect();
            let atr = trs.iter().sum::<f64>() / period as f64;
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, ema, &config.chart_id, Some(&config.target_series_id), ind_type_str)];
            if let Some(sid) = config.extra_target_ids.get("upper") { cmds.push(single_update_cmd(sid, point.time, ema + mult * atr, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            if let Some(sid) = config.extra_target_ids.get("lower") { cmds.push(single_update_cmd(sid, point.time, ema - mult * atr, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            Ok((cmds.join("\n"), Some(IndicatorState::Ema(ema))))
        },
        IndicatorType::DonchianChannels => {
            if data.len() < period { return Err("Insufficient Donchian data".to_string()); }
            let window: Vec<&Point> = data.iter().rev().take(period).collect();
            let hh = window.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
            let mid = (hh + ll) / 2.0;
            let mut cmds = vec![single_update_cmd(&config.target_series_id, point.time, mid, &config.chart_id, Some(&config.target_series_id), ind_type_str)];
            if let Some(sid) = config.extra_target_ids.get("upper") { cmds.push(single_update_cmd(sid, point.time, hh, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            if let Some(sid) = config.extra_target_ids.get("lower") { cmds.push(single_update_cmd(sid, point.time, ll, &config.chart_id, Some(&config.target_series_id), ind_type_str)); }
            Ok((cmds.join("\n"), None))
        },
        _ => Err(format!("Unsupported volatility indicator: {:?}", config.indicator_type)),
    }
}
