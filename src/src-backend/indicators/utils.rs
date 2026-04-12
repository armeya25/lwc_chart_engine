use serde_json::json;
use polars::prelude::DataFrame;
use crate::time_utils;

pub fn single_update_cmd(sid: &str, time: i64, value: f64, chart_id: &str, indicator: Option<&str>, ind_type: Option<&str>) -> String {
    let mut map = serde_json::Map::new();
    map.insert("action".to_string(), json!("update_series_data"));
    map.insert("chartId".to_string(), json!(chart_id));
    map.insert("seriesId".to_string(), json!(sid));
    map.insert("data".to_string(), json!({ "time": time, "value": value }));
    if let Some(i) = indicator { map.insert("indicator".to_string(), json!(i)); }
    if let Some(t) = ind_type  { map.insert("indicatorType".to_string(), json!(t)); }
    serde_json::to_string(&json!(map)).unwrap()
}

pub fn df_to_set_data_cmd(df: &DataFrame, sid: &str, chart_id: &str, indicator: Option<&str>, ind_type: Option<&str>) -> Result<String, String> {
    let data = time_utils::df_to_json_list(df)?;
    let mut map = serde_json::Map::new();
    map.insert("action".to_string(), json!("set_series_data"));
    map.insert("chartId".to_string(), json!(chart_id));
    map.insert("seriesId".to_string(), json!(sid));
    map.insert("data".to_string(), json!(data));
    if let Some(i) = indicator { map.insert("indicator".to_string(), json!(i)); }
    if let Some(t) = ind_type  { map.insert("indicatorType".to_string(), json!(t)); }
    Ok(serde_json::to_string(&json!(map)).unwrap())
}

pub fn df_to_set_data_cmd_colored(df: &DataFrame, sid: &str, chart_id: &str, indicator: Option<&str>, ind_type: Option<&str>) -> Result<String, String> {
    let data = time_utils::df_to_json_list_colored(df)?;
    let mut map = serde_json::Map::new();
    map.insert("action".to_string(), json!("set_series_data"));
    map.insert("chartId".to_string(), json!(chart_id));
    map.insert("seriesId".to_string(), json!(sid));
    map.insert("data".to_string(), json!(data));
    if let Some(i) = indicator { map.insert("indicator".to_string(), json!(i)); }
    if let Some(t) = ind_type  { map.insert("indicatorType".to_string(), json!(t)); }
    Ok(serde_json::to_string(&json!(map)).unwrap())
}
