use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDateTime;
use polars::prelude::*;

use crate::modules::streamer::candle_closed::CandleClosed;

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Cached period info for a single timeframe.
#[derive(Debug, Clone)]
pub struct PeriodCacheEntry {
    pub start_dt: NaiveDateTime,
    pub next_boundary: NaiveDateTime,
    pub start_idx: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Streamer {
    /// The full dataset loaded into memory.
    df: Option<DataFrame>,

    /// Cached "date" column as a Vec<NaiveDateTime> for fast access.
    pub dates: Vec<NaiveDateTime>,

    /// Current row index in the dataframe.
    pub current_idx: usize,

    pub opens: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub closes: Vec<f64>,
    pub volumes: Vec<f64>,

    /// Current datetime being processed.
    pub current_dt: Option<NaiveDateTime>,

    /// Total number of rows.
    pub total: usize,

    /// Check delta in seconds (input_tf duration - 1s).
    check_delta_secs: i64,

    /// Period boundary cache: timeframe -> PeriodCacheEntry.
    period_cache: HashMap<String, PeriodCacheEntry>,

    /// Candle closed logic (embedded, mirrors Python's inheritance).
    pub candle_closed: CandleClosed,
}

impl Streamer {
    pub fn new() -> Self {
        Streamer {
            df: None,
            dates: Vec::new(),
            current_idx: 0,
            opens: Vec::new(),
            highs: Vec::new(),
            lows: Vec::new(),
            closes: Vec::new(),
            volumes: Vec::new(),
            current_dt: None,
            total: 0,
            check_delta_secs: 59,
            period_cache: HashMap::new(),
            candle_closed: CandleClosed::new(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Load data from a parquet or CSV file.
    ///
    /// `slice` — if negative, takes the last N rows; if positive, skips first N rows; 0 = all data.
    /// `input_tf` — the base timeframe string (e.g. "1m", "5m") used to calculate check_delta.
    pub fn set_stream_data(
        &mut self,
        file_name: &str,
        slice: i64,
        input_tf: &str,
    ) -> Result<(), PolarsError> {
        let path = Path::new(file_name);

        let mut df = if file_name.ends_with(".parquet") {
            let file = std::fs::File::open(path)?;
            ParquetReader::new(file).finish()?
        } else {
            CsvReadOptions::default()
                .try_into_reader_with_file_path(Some(path.into()))?
                .finish()?
        };

        // Apply slice
        if slice != 0 {
            let height = df.height() as i64;
            if slice < 0 {
                // Take last N rows
                let offset = (height + slice).max(0) as i64;
                df = df.slice(offset, (-slice) as usize);
            } else {
                df = df.slice(slice, (height - slice).max(0) as usize);
            }
        }

        // Parse date column if it's a string
        let date_dtype = df.column("date")?.dtype().clone();
        if date_dtype == DataType::String {
            df = df
                .lazy()
                .with_column(col("date").str().to_datetime(
                    None, None, StrptimeOptions::default(), lit("raise"),
                ))
                .collect()?;
        }

        // Strip timezone if present
        if let DataType::Datetime(_, Some(_)) = df.column("date")?.dtype() {
            df = df
                .lazy()
                .with_column(col("date").dt().replace_time_zone(None, lit(false), NonExistent::Raise))
                .collect()?;
        }

        // Drop rows with null dates
        df = df.drop_nulls::<String>(Some(&["date".to_string()]))?;

        // Sort by date
        df = df.sort(["date"], SortMultipleOptions::default())?;

        // Cache dates as Vec<NaiveDateTime> for O(1) access
        let date_col = df.column("date")?.datetime()?.clone();
        self.dates = date_col
            .into_no_null_iter()
            .map(|ts_us| {
                let secs = ts_us / 1_000_000;
                let nsecs = ((ts_us % 1_000_000) * 1_000) as u32;
                chrono::DateTime::from_timestamp(secs, nsecs).unwrap().naive_utc()
            })
            .collect();

        // Cache OHLCV columns as Vec<f64>
        self.opens = df.column("open")?.f64()?.clone().into_no_null_iter().collect();
        self.highs = df.column("high")?.f64()?.clone().into_no_null_iter().collect();
        self.lows = df.column("low")?.f64()?.clone().into_no_null_iter().collect();
        self.closes = df.column("close")?.f64()?.clone().into_no_null_iter().collect();
        self.volumes = df.column("volume")?.f64()?.clone().into_no_null_iter().collect();

        self.total = df.height();
        self.df = Some(df);
        self.current_idx = 0;

        // Calculate check delta from input timeframe
        let duration_secs: i64 = if input_tf.ends_with("m") && !input_tf.ends_with("mo") {
            let minutes: i64 = input_tf[..input_tf.len() - 1].parse().unwrap_or(1);
            minutes * 60
        } else if input_tf == "1d" {
            86400
        } else {
            60
        };
        self.check_delta_secs = duration_secs - 1;
        self.candle_closed.check_delta_secs = self.check_delta_secs;
        self.candle_closed.input_tf = input_tf.to_string();

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Binary search on the cached dates vector (equivalent to polars search_sorted).
    fn search_sorted_left(&self, target: NaiveDateTime) -> usize {
        self.dates.partition_point(|d| *d < target)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Get cached period info for a timeframe, refreshing if the boundary has been crossed.
    fn get_period_info(&mut self, tf: &str) -> Result<PeriodCacheEntry, String> {
        let current_dt = self.current_dt.ok_or("current_dt is None")?;

        // Check cache validity
        if let Some(entry) = self.period_cache.get(tf) {
            if current_dt < entry.next_boundary {
                return Ok(entry.clone());
            }
        }

        // Cache miss / expired
        let start_dt = self.candle_closed.get_period_start(current_dt, tf)?;
        let next_boundary = self.candle_closed.calculate_next_boundary(tf, start_dt)?;
        let start_idx = self.search_sorted_left(start_dt);

        let entry = PeriodCacheEntry {
            start_dt,
            next_boundary,
            start_idx,
        };
        self.period_cache.insert(tf.to_string(), entry.clone());
        Ok(entry)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Get the aggregated OHLCV bar for the current period of the given timeframe.
    ///
    /// Returns (date, open, high, low, close, volume_opt).
    pub fn get_stream_data(
        &mut self,
        tf: &str,
    ) -> Result<Option<AggBar>, String> {
        let info = self.get_period_info(tf)?;
        let df = self.df.as_ref().ok_or("No data set. Call set_stream_data() first.")?;

        let start_idx = info.start_idx;
        let end_idx = self.current_idx + 1; // exclusive

        if start_idx >= end_idx {
            return Ok(None);
        }

        let subset = df.slice(start_idx as i64, end_idx - start_idx);

        if subset.height() == 0 {
            return Ok(None);
        }

        let open_col = subset.column("open").map_err(|e| e.to_string())?;
        let high_col = subset.column("high").map_err(|e| e.to_string())?;
        let low_col = subset.column("low").map_err(|e| e.to_string())?;
        let close_col = subset.column("close").map_err(|e| e.to_string())?;

        let open = open_col.f64().map_err(|e| e.to_string())?.get(0).unwrap_or(0.0);
        let high = high_col.f64().map_err(|e| e.to_string())?.max().unwrap_or(0.0);
        let low = low_col.f64().map_err(|e| e.to_string())?.min().unwrap_or(0.0);
        let close_len = close_col.len();
        let close = close_col
            .f64()
            .map_err(|e| e.to_string())?
            .get(close_len - 1)
            .unwrap_or(0.0);

        let volume = if let Ok(vol_col) = subset.column("volume") {
            vol_col.f64().ok().and_then(|v| v.sum())
        } else {
            None
        };

        Ok(Some(AggBar {
            date: info.start_dt,
            open,
            high,
            low,
            close,
            volume,
        }))
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Check if the candle for the given timeframe is closed.
    pub fn is_closed(&mut self, tf: &str) -> Result<bool, String> {
        let next_bar_dt = if self.current_idx + 1 < self.total {
            Some(self.dates[self.current_idx + 1])
        } else {
            None
        };

        self.candle_closed
            .is_closed(tf, self.current_idx, self.total, next_bar_dt)
    }

    /// Calculates a 'forming' candle for the target timeframe by aggregating bars 
    /// from the current period start to the current index.
    pub fn get_forming_candle(&mut self, tf: &str) -> Result<HashMap<String, f64>, String> {
        let idx = self.current_idx;
        if idx >= self.total { return Err("End of stream".into()); }

        let dt = self.dates[idx];
        let p_start = self.candle_closed.get_period_start(dt, tf)?;

        // Find the start index for the current period (O(N) but typically small)
        let mut s_idx = idx;
        while s_idx > 0 && self.dates[s_idx - 1] >= p_start {
            s_idx -= 1;
        }

        // Aggregate across [s_idx..=idx]
        let open = self.opens[s_idx];
        let mut high = self.highs[s_idx];
        let mut low = self.lows[s_idx];
        let mut volume = 0.0;

        for i in s_idx..=idx {
            if self.highs[i] > high { high = self.highs[i]; }
            if self.lows[i] < low { low = self.lows[i]; }
            volume += self.volumes[i];
        }

        let mut res = HashMap::new();
        res.insert("open".to_string(), open);
        res.insert("high".to_string(), high);
        res.insert("low".to_string(), low);
        res.insert("close".to_string(), self.closes[idx]);
        res.insert("volume".to_string(), volume);
        Ok(res)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Iterate over every row, updating internal state.
    /// Returns an iterator of row indices.
    pub fn stream_runner(&mut self, start_index: usize) -> StreamIter<'_> {
        self.period_cache.clear();
        StreamIter {
            streamer: self,
            i: 0,
            start_index,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Check if the stream has ended.
    pub fn is_stream_ended(&self) -> bool {
        self.current_idx >= self.total.saturating_sub(1)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Retrieves all historical aggregated bars up to the current index.
    pub fn get_chart_data(&self, tf: &str) -> Result<DataFrame, String> {
        let df = self.df.as_ref().ok_or("No data set. Call set_stream_data() first.")?;
        let subset = df.slice(0, self.current_idx + 1);

        if tf == "1m" {
            return Ok(subset);
        }

        let mut agg_exprs = vec![
            col("open").first(),
            col("high").max(),
            col("low").min(),
            col("close").last(),
        ];
        if subset.column("volume").is_ok() {
            agg_exprs.push(col("volume").sum());
        }

        if tf.ends_with('m') && !tf.ends_with("mo") {
            let minutes: i64 = tf[..tf.len() - 1].parse::<i64>().map_err(|e| e.to_string())?;
            let interval_secs = minutes * 60;

            let market_open_h = self.candle_closed.market_open_h;
            let market_open_m = self.candle_closed.market_open_m;

            let market_open_time_str = format!("{:02}:{:02}:00", market_open_h, market_open_m);

            let res = subset.lazy()
                .with_columns(vec![
                    // date_only = dt.date()
                    col("date").dt().date().cast(DataType::Datetime(TimeUnit::Microseconds, None)).alias("date_only"),
                ])
                .with_columns(vec![
                    // market_open_dt = date_only + market_hours
                    (col("date_only").cast(DataType::Int64) + lit(market_open_h as i64 * 3600 * 1_000_000 + market_open_m as i64 * 60 * 1_000_000))
                        .cast(DataType::Datetime(TimeUnit::Microseconds, None))
                        .alias("market_open_dt"),
                ])
                .with_columns(vec![
                    // base_dt logic using string casting for NaiveTime literal equivalent
                    when(col("date").dt().time().lt(lit(market_open_time_str.as_str()).cast(DataType::Time)))
                        .then(col("date_only"))
                        .otherwise(col("market_open_dt"))
                        .alias("base_dt"),
                ])
                .with_columns(vec![
                    // diff_seconds = (date - base_dt)
                    ((col("date").cast(DataType::Int64) - col("base_dt").cast(DataType::Int64)) / lit(1_000_000)).alias("diff_seconds"),
                ])
                .with_columns(vec![
                    // bucket_idx = floor(diff_seconds / interval)
                    (col("diff_seconds") / lit(interval_secs)).cast(DataType::Int64).alias("bucket_idx"),
                ])
                .with_columns(vec![
                    // period_start = base_dt + bucket_idx * interval
                    (col("base_dt").cast(DataType::Int64) + (col("bucket_idx") * lit(interval_secs * 1_000_000)))
                        .cast(DataType::Datetime(TimeUnit::Microseconds, None))
                        .alias("date"),
                ])
                .group_by_stable([col("date")])
                .agg(agg_exprs)
                .collect()
                .map_err(|e| e.to_string())?;

            Ok(res)
        } else {
            let every = match tf {
                "1d" => "1d",
                "1w" => "1w",
                _ => "1m",
            };

            let options = DynamicGroupOptions {
                index_column: "date".into(),
                every: polars::prelude::Duration::parse(every),
                period: polars::prelude::Duration::parse(every),
                offset: polars::prelude::Duration::parse("0ns"),
                closed_window: ClosedWindow::Left,
                label: Label::Left,
                include_boundaries: false,
                start_by: Default::default(),
            };

            let res = subset.lazy()
                .group_by_dynamic(col("date"), vec![], options)
                .agg(agg_exprs)
                .collect()
                .map_err(|e| e.to_string())?;

            Ok(res)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Aggregated OHLCV bar returned by `get_stream_data`.
#[derive(Debug, Clone)]
pub struct AggBar {
    pub date: NaiveDateTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Iterator returned by `stream_runner`.
pub struct StreamIter<'a> {
    streamer: &'a mut Streamer,
    i: usize,
    start_index: usize,
}

impl<'a> Iterator for StreamIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        let idx = self.start_index + i;
        if idx >= self.streamer.total {
            return None;
        }

        self.streamer.current_idx = idx;
        self.streamer.current_dt = Some(self.streamer.dates[idx]);
        self.streamer.candle_closed.current_dt = self.streamer.current_dt;

        self.i += 1;
        Some(idx)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streamer_creation() {
        let s = Streamer::new();
        assert_eq!(s.total, 0);
        assert!(s.current_dt.is_none());
        assert_eq!(s.check_delta_secs, 59);
    }
}
