use polars::prelude::*;
use crate::types::{IndicatorConfig, IndicatorType, Point, IndicatorState};
use crate::indicators::utils::{df_to_set_data_cmd, single_update_cmd};

pub fn calculate_batch(config: &IndicatorConfig, df_lazy: LazyFrame) -> Result<String, String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
        IndicatorType::Vwap => {
            let res_df = df_lazy
                .with_column(((col("high") + col("low") + col("close")) / lit(3.0)).alias("tp"))
                .with_columns([
                    (col("tp") * col("volume")).cum_sum(false).alias("cum_tpv"),
                    col("volume").cum_sum(false).alias("cum_vol"),
                ])
                .with_column((col("cum_tpv") / col("cum_vol")).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Obv => {
            let res_df = df_lazy
                .with_column((col("close") - col("close").shift(lit(1))).alias("diff"))
                .with_column(
                    when(col("diff").gt(0.0)).then(col("volume"))
                    .when(col("diff").lt(0.0)).then(-col("volume"))
                    .otherwise(0.0)
                    .alias("v_dir")
                )
                .with_column(col("v_dir").cum_sum(false).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        IndicatorType::Adl => {
            let res_df = df_lazy
                .with_column(
                    (((col("close") - col("low")) - (col("high") - col("close"))) / (col("high") - col("low")) * col("volume")).alias("mfv")
                )
                .with_column(col("mfv").cum_sum(false).alias("value"))
                .collect().map_err(|e: PolarsError| e.to_string())?;
            df_to_set_data_cmd(&res_df, &config.target_series_id, &config.chart_id, Some(&config.target_series_id), ind_type_str)
        },
        _ => Err(format!("Unsupported volume indicator: {:?}", config.indicator_type)),
    }
}

pub fn calculate_step(
    config: &IndicatorConfig,
    data: &[Point],
    point: &Point,
    state: Option<IndicatorState>,
) -> Result<(String, Option<IndicatorState>), String> {
    let ind_type_str = Some(config.indicator_type.as_str());
    match config.indicator_type {
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
            Ok((single_update_cmd(&config.target_series_id, point.time, value, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Vwap { cum_tpv: new_tpv, cum_vol: new_vol })))
        },
        IndicatorType::Obv => {
            let (prev_obv, prev_close) = match state {
                Some(IndicatorState::Obv(val)) => (val, data[data.len()-2].close),
                _ => {
                    let mut obv = 0.0;
                    for i in 1..data.len() {
                        let c = data[i].close; let pc = data[i-1].close; let v = data[i].volume.unwrap_or(0.0);
                        if c > pc { obv += v; } else if c < pc { obv -= v; }
                    }
                    (obv, data[data.len()-2].close)
                }
            };
            let vol = point.volume.unwrap_or(0.0);
            let new_obv = if point.close > prev_close { prev_obv + vol }
                         else if point.close < prev_close { prev_obv - vol }
                         else { prev_obv };
            Ok((single_update_cmd(&config.target_series_id, point.time, new_obv, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Obv(new_obv))))
        },
        IndicatorType::Adl => {
            let curr_adl = match state {
                Some(IndicatorState::Adl(val)) => val,
                _ => data.iter().map(|p| {
                    let mfm = if p.high == p.low { 0.0 } else { ((p.close - p.low) - (p.high - p.close)) / (p.high - p.low) };
                    mfm * p.volume.unwrap_or(0.0)
                }).sum::<f64>(),
            };
            let mfm = if point.high == point.low { 0.0 } else { ((point.close - point.low) - (point.high - point.close)) / (point.high - point.low) };
            let new_adl = curr_adl + mfm * point.volume.unwrap_or(0.0);
            Ok((single_update_cmd(&config.target_series_id, point.time, new_adl, &config.chart_id, Some(&config.target_series_id), ind_type_str), Some(IndicatorState::Adl(new_adl))))
        },
        _ => Err(format!("Unsupported volume indicator: {:?}", config.indicator_type)),
    }
}
