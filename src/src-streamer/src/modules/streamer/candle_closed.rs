use std::collections::HashMap;
use chrono::{NaiveDateTime, Datelike, Timelike, Duration};

/// Flattened map for O(1) lookup of timeframe types.
fn timeframe_type(tf: &str) -> Option<&'static str> {
    match tf {
        "1m" | "5m" | "15m" | "30m" | "60m" | "75m" | "125m" => Some("minute"),
        "1h" | "2h" | "4h" | "6h" | "8h" | "12h" => Some("hour"),
        "1d" => Some("day"),
        "1w" => Some("week"),
        "1mo" | "3mo" | "6mo" => Some("month"),
        "1y" => Some("year"),
        _ => {
            if tf.ends_with("m") && !tf.ends_with("mo") {
                Some("minute")
            } else if tf.ends_with("h") {
                Some("hour")
            } else if tf.ends_with("d") {
                Some("day")
            } else if tf.ends_with("w") {
                Some("week")
            } else if tf.ends_with("mo") {
                Some("month")
            } else if tf.ends_with("y") {
                Some("year")
            } else {
                None
            }
        }
    }
}

/// Parse the number of minutes represented by a timeframe string like "5m", "75m", "2h".
fn parse_tf_minutes(tf: &str) -> Result<i64, String> {
    if tf.ends_with("m") && !tf.ends_with("mo") {
        tf[..tf.len() - 1]
            .parse::<i64>()
            .map_err(|e| format!("Cannot parse minutes from '{}': {}", tf, e))
    } else if tf.ends_with("h") {
        tf[..tf.len() - 1]
            .parse::<i64>()
            .map(|h| h * 60)
            .map_err(|e| format!("Cannot parse hours from '{}': {}", tf, e))
    } else {
        Err(format!("Cannot parse minutes from: {}", tf))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct CandleClosed {
    pub market_open_time: String,
    pub market_close_time: String,

    pub market_open_h: u32,
    pub market_open_m: u32,
    pub market_close_h: u32,
    pub market_close_m: u32,

    /// Cache: timeframe -> minutes
    tf_minutes_cache: HashMap<String, i64>,

    /// Check delta in seconds (input_tf duration - 1s)
    pub check_delta_secs: i64,

    /// Current datetime being processed
    pub current_dt: Option<NaiveDateTime>,

    /// The base timeframe of the input data
    pub input_tf: String,
}

impl CandleClosed {
    pub fn new() -> Self {
        let mut cc = CandleClosed {
            market_open_time: "09:15".to_string(),
            market_close_time: "15:30".to_string(),
            market_open_h: 0,
            market_open_m: 0,
            market_close_h: 0,
            market_close_m: 0,
            tf_minutes_cache: HashMap::new(),
            check_delta_secs: 59, // default: 1m - 1s
            current_dt: None,
            input_tf: "".to_string(),
        };
        cc.set_market_hours("09:15", "15:30");
        cc
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    pub fn set_market_hours(&mut self, open_time: &str, close_time: &str) {
        self.market_open_time = open_time.to_string();
        self.market_close_time = close_time.to_string();

        let parts: Vec<&str> = open_time.split(':').collect();
        self.market_open_h = parts[0].parse().unwrap_or(9);
        self.market_open_m = parts[1].parse().unwrap_or(15);

        let parts_c: Vec<&str> = close_time.split(':').collect();
        self.market_close_h = parts_c[0].parse().unwrap_or(15);
        self.market_close_m = parts_c[1].parse().unwrap_or(30);
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Calculate the start of the period that `dt` falls into for the given timeframe.
    pub fn get_period_start(&mut self, dt: NaiveDateTime, tf: &str) -> Result<NaiveDateTime, String> {
        let tf_type = timeframe_type(tf).ok_or(format!("Unsupported timeframe: {}", tf))?;

        match tf_type {
            "minute" | "hour" => {
                let minutes = if let Some(&cached) = self.tf_minutes_cache.get(tf) {
                    cached
                } else {
                    let m = parse_tf_minutes(tf)?;
                    self.tf_minutes_cache.insert(tf.to_string(), m);
                    m
                };

                let market_open_dt = dt
                    .date()
                    .and_hms_opt(self.market_open_h, self.market_open_m, 0)
                    .unwrap();

                let base_dt = if dt < market_open_dt {
                    dt.date().and_hms_opt(0, 0, 0).unwrap()
                } else {
                    market_open_dt
                };

                let diff_secs = (dt - base_dt).num_seconds();
                let interval_secs = minutes * 60;
                let bucket_idx = diff_secs / interval_secs; // floor division (both positive)

                Ok(base_dt + Duration::seconds(bucket_idx * interval_secs))
            }
            "day" => Ok(dt.date().and_hms_opt(0, 0, 0).unwrap()),
            "week" => {
                let weekday = dt.weekday().num_days_from_monday() as i64;
                let monday = dt.date() - Duration::days(weekday);
                Ok(monday.and_hms_opt(0, 0, 0).unwrap())
            }
            "month" => {
                if tf == "1mo" {
                    Ok(dt
                        .date()
                        .with_day(1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap())
                } else if tf == "3mo" {
                    let month_idx = dt.month0(); // 0-based
                    let quarter_start = (month_idx / 3) * 3 + 1;
                    Ok(dt
                        .date()
                        .with_month(quarter_start)
                        .unwrap()
                        .with_day(1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap())
                } else {
                    // Generic Nmo
                    let months_str = tf.strip_suffix("mo").ok_or(format!("Bad tf: {}", tf))?;
                    let months: u32 = months_str.parse().map_err(|e| format!("{}", e))?;
                    let month_idx = dt.month0();
                    let start_month = (month_idx / months) * months + 1;
                    Ok(dt
                        .date()
                        .with_month(start_month)
                        .unwrap()
                        .with_day(1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap())
                }
            }
            "year" => Ok(dt
                .date()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()),
            _ => Err(format!("Unsupported timeframe: {}", tf)),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Calculate the start of the NEXT period boundary from a given period start.
    pub fn calculate_next_boundary(
        &self,
        tf: &str,
        start_dt: NaiveDateTime,
    ) -> Result<NaiveDateTime, String> {
        if tf.ends_with("m") && !tf.ends_with("mo") {
            let minutes: i64 = tf[..tf.len() - 1]
                .parse()
                .map_err(|e| format!("{}", e))?;
            Ok(start_dt + Duration::minutes(minutes))
        } else if tf.ends_with("h") {
            let hours: i64 = tf[..tf.len() - 1]
                .parse()
                .map_err(|e| format!("{}", e))?;
            Ok(start_dt + Duration::hours(hours))
        } else if tf.ends_with("w") {
            Ok(start_dt + Duration::weeks(1))
        } else if tf == "1d" {
            Ok(start_dt + Duration::days(1))
        } else if tf == "1mo" {
            let month = start_dt.month();
            let year = start_dt.year();
            let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
            Ok(start_dt
                .date()
                .with_year(ny)
                .unwrap()
                .with_month(nm)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap())
        } else if tf == "3mo" {
            let mut nm = start_dt.month() + 3;
            let mut ny = start_dt.year();
            if nm > 12 {
                nm -= 12;
                ny += 1;
            }
            Ok(start_dt
                .date()
                .with_year(ny)
                .unwrap()
                .with_month(nm)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap())
        } else if tf == "6mo" {
            let mut nm = start_dt.month() + 6;
            let mut ny = start_dt.year();
            if nm > 12 {
                nm -= 12;
                ny += 1;
            }
            Ok(start_dt
                .date()
                .with_year(ny)
                .unwrap()
                .with_month(nm)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap())
        } else if tf == "1y" {
            Ok(start_dt
                .date()
                .with_year(start_dt.year() + 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap())
        } else {
            Ok(start_dt + Duration::days(1))
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Check if the candle for the timeframe is closed at `current_dt`.
    pub fn is_candle_closed(&mut self, current_dt: NaiveDateTime, tf: &str) -> Result<bool, String> {
        let probe_dt = current_dt + Duration::seconds(1);

        let start_current = self.get_period_start(current_dt, tf)?;
        let start_next = self.get_period_start(probe_dt, tf)?;

        if start_current != start_next {
            // Skip first candle at market open if it's <= 5 minutes
            if start_current.hour() == self.market_open_h
                && start_current.minute() == self.market_open_m
            {
                let minutes_since_open =
                    (start_next - start_current).num_seconds() as f64 / 60.0;
                if minutes_since_open <= 5.0 {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // Special case for '1d' at Market Close
        if tf == "1d"
            && probe_dt.hour() == self.market_close_h
            && probe_dt.minute() == self.market_close_m
            && probe_dt.second() == 0
        {
            return Ok(true);
        }

        Ok(false)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Simplified closure check using shared state (_current_dt + check_delta).
    /// `total` and an optional next-bar datetime are needed for the lookahead logic.
    pub fn is_closed(
        &mut self,
        tf: &str,
        current_idx: usize,
        total: usize,
        next_bar_dt: Option<NaiveDateTime>,
    ) -> Result<bool, String> {
        let dt_open = match self.current_dt {
            Some(dt) => dt,
            None => return Ok(false),
        };

        let exact_close_dt = dt_open + Duration::seconds(self.check_delta_secs + 1);
        let period_start = self.get_period_start(dt_open, tf)?;
        let next_boundary = self.calculate_next_boundary(tf, period_start)?;

        // 1. Target Timeframe Boundary Crossing
        if exact_close_dt >= next_boundary {
            return Ok(true);
        }

        // 2. High-TF Lookahead (Holiday/Weekend skip)
        let tf_type = timeframe_type(tf);
        if matches!(tf_type, Some("week") | Some("month") | Some("year")) {
            let next_idx = current_idx + 1;
            if next_idx >= total {
                return Ok(true);
            }
            if let Some(next_dt) = next_bar_dt {
                if self.get_period_start(next_dt, tf)? != period_start {
                    return Ok(true);
                }
            }
        }

        // 3. Market Close logic for '1d' (essential for Intraday streams)
        if tf == "1d" {
            if exact_close_dt.hour() == self.market_close_h && exact_close_dt.minute() == self.market_close_m {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_period_start_5m() {
        let mut cc = CandleClosed::new();
        let dt = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 23, 0)
            .unwrap();
        let start = cc.get_period_start(dt, "5m").unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 20, 0)
            .unwrap();
        assert_eq!(start, expected);
    }

    #[test]
    fn test_period_start_15m() {
        let mut cc = CandleClosed::new();
        let dt = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 40, 0)
            .unwrap();
        let start = cc.get_period_start(dt, "15m").unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap();
        assert_eq!(start, expected);
    }

    #[test]
    fn test_period_start_1d() {
        let mut cc = CandleClosed::new();
        let dt = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let start = cc.get_period_start(dt, "1d").unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(start, expected);
    }

    #[test]
    fn test_next_boundary_5m() {
        let cc = CandleClosed::new();
        let start = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 15, 0)
            .unwrap();
        let next = cc.calculate_next_boundary("5m", start).unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 20, 0)
            .unwrap();
        assert_eq!(next, expected);
    }

    #[test]
    fn test_candle_closed_at_boundary() {
        let mut cc = CandleClosed::new();
        // 9:19:59 is the last second of the 9:15 5m candle
        let dt = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 19, 59)
            .unwrap();
        // First candle at market open <=5m should be skipped
        let closed = cc.is_candle_closed(dt, "5m").unwrap();
        assert_eq!(closed, false);
    }

    #[test]
    fn test_candle_closed_second_bar() {
        let mut cc = CandleClosed::new();
        // 9:24:59 is the last second of the 9:20 5m candle
        let dt = NaiveDate::from_ymd_opt(2025, 8, 25)
            .unwrap()
            .and_hms_opt(9, 24, 59)
            .unwrap();
        let closed = cc.is_candle_closed(dt, "5m").unwrap();
        assert_eq!(closed, true);
    }
}
