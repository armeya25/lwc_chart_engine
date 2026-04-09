#[cfg(feature = "python-bridge")]
use chrono::NaiveDateTime;
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use polars::prelude::*;
#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;
#[cfg(feature = "python-bridge")]
use pyo3::types::PyAny;
#[cfg(feature = "python-bridge")]
use pyo3_polars::PyDataFrame;
use std::sync::RwLock;

static CURRENT_TZ: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("UTC".to_string()));

pub fn set_backend_timezone(timezone_str: String) -> Result<(), String> {
    let _: Tz = timezone_str.parse().map_err(|e| format!("{}", e))?;
    let mut tz = CURRENT_TZ.write().unwrap();
    *tz = timezone_str;
    Ok(())
}

#[cfg(feature = "python-bridge")]
#[pyfunction]
pub fn py_set_backend_timezone(timezone_str: String) -> PyResult<()> {
    set_backend_timezone(timezone_str).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
}

fn get_current_tz() -> Tz {
    let tz_str = CURRENT_TZ.read().unwrap();
    tz_str.parse().unwrap_or(chrono_tz::UTC)
}

#[cfg(feature = "python-bridge")]
#[pyfunction]
pub fn py_ensure_timestamp(val: &Bound<'_, PyAny>) -> PyResult<Option<i64>> {
    if val.is_none() {
        return Ok(None);
    }

    let type_name = val.get_type().qualname()?.to_string();

    // Special handling for datetime objects to ensure UTC alignment
    if type_name == "datetime" {
        let has_tzinfo = val.getattr("tzinfo")?.is_none() == false;
        if has_tzinfo {
            // If already has tzinfo, .timestamp() is reliable
            if let Ok(ts) = val.call_method0("timestamp")?.extract::<f64>() {
                return Ok(Some(ts as i64));
            }
        } else {
            // Naive datetime: we MUST assume UTC to match the polars processing logic
            // Convert naive to UTC by adding UTC tzinfo
            let dt_module = val.py().import("datetime")?;
            let timezone_utc = dt_module.getattr("timezone")?.getattr("utc")?;

            let kwargs = pyo3::types::PyDict::new(val.py());
            kwargs.set_item("tzinfo", timezone_utc)?;

            let dt_utc = val.call_method("replace", (), Some(&kwargs))?;
            if let Ok(ts) = dt_utc.call_method0("timestamp")?.extract::<f64>() {
                return Ok(Some(ts as i64));
            }
        }
    }

    // Try calling .timestamp() directly as fallback (works for some other types)
    if let Ok(ts_res) = val.call_method0("timestamp") {
        if let Ok(ts) = ts_res.extract::<f64>() {
            return Ok(Some(ts as i64));
        }
    }

    if type_name == "date" {
        let dt_module = val.py().import("datetime")?;
        let datetime_class = dt_module.getattr("datetime")?;
        let time_class = dt_module.getattr("time")?;
        let dt = datetime_class.call_method1("combine", (val, time_class.call0()?))?;

        let timezone_utc = dt_module.getattr("timezone")?.getattr("utc")?;
        let kwargs = pyo3::types::PyDict::new(val.py());
        kwargs.set_item("tzinfo", timezone_utc)?;

        let dt_utc = dt.call_method("replace", (), Some(&kwargs))?;
        let ts = dt_utc.call_method0("timestamp")?.extract::<f64>()?;
        return Ok(Some(ts as i64));
    }

    if let Ok(s) = val.extract::<String>() {
        let tz = get_current_tz();
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
            let dt = tz
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| Utc.from_local_datetime(&naive).unwrap().with_timezone(&tz));
            return Ok(Some(dt.timestamp()));
        }
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            let naive = date.and_hms_opt(0, 0, 0).unwrap();
            let dt = tz
                .from_local_datetime(&naive)
                .single()
                .unwrap_or_else(|| Utc.from_local_datetime(&naive).unwrap().with_timezone(&tz));
            return Ok(Some(dt.timestamp()));
        }
    }

    if let Ok(ts) = val.extract::<i64>() {
        return Ok(Some(ts));
    }
    if let Ok(ts) = val.extract::<f64>() {
        return Ok(Some(ts as i64));
    }

    Ok(None)
}

#[cfg(feature = "python-bridge")]
#[pyfunction]
pub fn py_process_polars_data(pydf: Bound<'_, PyAny>) -> PyResult<PyDataFrame> {
    let pydf: PyDataFrame = pydf.extract()?;
    let df = pydf.0;
    let processed = process_polars_data(df)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{}", e)))?;
    Ok(PyDataFrame(processed))
}

pub fn process_polars_data(df: DataFrame) -> PolarsResult<DataFrame> {
    let tz_str = CURRENT_TZ.read().unwrap();
    let tz: Option<Tz> = tz_str.parse().ok();

    // 1. Lowercase all column names
    let mut df = df;
    let old_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new_names: Vec<String> = old_names.iter().map(|s| s.to_lowercase()).collect();
    for (old, new) in old_names.iter().zip(new_names.iter()) {
        if old != new {
            let _ = df.rename(old, new.into());
        }
    }

    // 2. Alias 'date' or 'datetime' to 'time' if necessary
    let has_time = df.get_column_names().iter().any(|s| s.as_str() == "time");
    if !has_time {
        if df.get_column_names().iter().any(|s| s.as_str() == "date") {
            let _ = df.rename("date", "time".into());
        } else if df
            .get_column_names()
            .iter()
            .any(|s| s.as_str() == "datetime")
        {
            let _ = df.rename("datetime", "time".into());
        }
    }

    // 3. Process time column
    if let Some(time_idx) = df
        .get_column_names()
        .iter()
        .position(|s| s.as_str() == "time")
    {
        let time_col = df.get_columns().get(time_idx).unwrap();
        let dtype = time_col.dtype().clone();

        let new_time_s = match &dtype {
            DataType::Datetime(tu, tz_opt) => {
                let ca = time_col.datetime()?;
                let tu_multiplier = match tu {
                    TimeUnit::Nanoseconds => 1_000_000_000,
                    TimeUnit::Microseconds => 1_000_000,
                    TimeUnit::Milliseconds => 1_000,
                };
                let vec: Vec<Option<i64>> = ca
                    .into_iter()
                    .map(|opt_ts| {
                        opt_ts.map(|ts| {
                            if tz_opt.is_some() {
                                ts / tu_multiplier
                            } else {
                                let secs = ts / tu_multiplier;
                                let nsecs =
                                    ((ts % tu_multiplier) * (1_000_000_000 / tu_multiplier)) as u32;
                                if let Some(naive) = chrono::DateTime::from_timestamp(secs, nsecs)
                                    .map(|dt| dt.naive_utc())
                                {
                                    if let Some(tz_val) = &tz {
                                        let dt = tz_val
                                            .from_local_datetime(&naive)
                                            .single()
                                            .unwrap_or_else(|| {
                                                Utc.from_local_datetime(&naive)
                                                    .unwrap()
                                                    .with_timezone::<Tz>(tz_val)
                                            });
                                        dt.timestamp()
                                    } else {
                                        naive.and_utc().timestamp()
                                    }
                                } else {
                                    0
                                }
                            }
                        })
                    })
                    .collect();
                PolarsResult::Ok(Series::new("time".into(), vec))
            }
            DataType::Date => {
                let ca = time_col.date()?;
                let vec: Vec<Option<i64>> = ca
                    .into_iter()
                    .map(|opt_days| {
                        opt_days.map(|days| {
                            let secs = days as i64 * 86400;
                            if let Some(naive) =
                                chrono::DateTime::from_timestamp(secs, 0).map(|dt| dt.naive_utc())
                            {
                                if let Some(tz_val) = &tz {
                                    let dt = tz_val
                                        .from_local_datetime(&naive)
                                        .single()
                                        .unwrap_or_else(|| {
                                            Utc.from_local_datetime(&naive)
                                                .unwrap()
                                                .with_timezone::<Tz>(tz_val)
                                        });
                                    dt.timestamp()
                                } else {
                                    naive.and_utc().timestamp()
                                }
                            } else {
                                0
                            }
                        })
                    })
                    .collect();
                PolarsResult::Ok(Series::new("time".into(), vec))
            }
            DataType::Int64 | DataType::Float64 => {
                let f64_s = time_col.cast(&DataType::Float64)?;
                let ca = f64_s.f64()?;
                let first_val = ca.get(0).unwrap_or(0.0);

                let scale = if first_val > 1e15 {
                    1_000_000_000.0
                } else if first_val > 1e12 {
                    1_000_000.0
                } else if first_val > 1e10 {
                    1_000.0
                } else {
                    1.0
                };

                let vec: Vec<Option<i64>> = ca
                    .into_iter()
                    .map(|opt_v| opt_v.map(|v| (v / scale) as i64))
                    .collect();
                PolarsResult::Ok(Series::new("time".into(), vec))
            }
            _ => PolarsResult::Ok(time_col.as_series().unwrap().clone()),
        }?;

        df.replace_column(time_idx, new_time_s)?;
    }

    // 4. Fill Nulls with 0
    let df = df.fill_null(FillNullStrategy::Zero)?;
    Ok(df)
}
