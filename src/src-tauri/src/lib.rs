#![allow(dead_code)]
#[path = "../../src-backend/chart.rs"] mod chart;
#[path = "../../src-backend/drawings.rs"] mod drawings;
#[path = "../../src-backend/trader.rs"] mod trader;
#[path = "../../src-backend/types.rs"] mod types;
#[path = "../../src-backend/time_utils.rs"] mod time_utils;
#[path = "../../src-backend/indicators.rs"] mod indicators;

#[cfg(feature = "python-bridge")]
use pyo3::prelude::*;
#[cfg(feature = "python-bridge")]
use pyo3::wrap_pyfunction;

use tauri::{Emitter};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::io::{self, BufRead, Write};
use serde_json::Value;

struct StdoutLogger;

impl log::Log for StdoutLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let payload = serde_json::json!({
                "action": "log",
                "level": record.level().to_string(),
                "message": format!("{}", record.args()),
                "target": record.target()
            });
            if let Ok(json_str) = serde_json::to_string(&payload) {
                let mut stdout = std::io::stdout();
                let _ = stdout.write_all(json_str.as_bytes());
                let _ = stdout.write_all(b"\n");
                let _ = stdout.flush();
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: StdoutLogger = StdoutLogger;

static FRONTEND_READY: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[tauri::command]
fn frontend_ready() {
    let mut ready = FRONTEND_READY.lock().unwrap();
    *ready = true;
}

#[tauri::command]
fn emit_to_backend(action: String, data: Value) {
    let payload = serde_json::json!({
        "action": action,
        "data": data
    });
    let mut stdout = io::stdout();
    let _ = stdout.write_all(serde_json::to_string(&payload).unwrap().as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

#[tauri::command]
fn set_timezone(tz: String) -> Result<(), String> {
    time_utils::set_backend_timezone(tz)
}

#[tauri::command]
fn get_chart_state() -> Value {
    // Example usage of chart/types
    serde_json::to_value(types::ChartCommand::new("get_state", "chart-0")).unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg_attr(feature = "python-bridge", pyfunction)]
pub fn run() {
    let _ = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug));
    
    // Set a custom panic hook to report crashes to stderr
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = panic_info.location().map(|l| format!(" at {}:{}", l.file(), l.line())).unwrap_or_default();
        eprintln!("💥 [Chart Engine Backend Panic]: {}{} ", message, location);
    }));

    tauri::Builder::default()
        .setup(|app| {
            log::debug!("Tauri setup started");
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                // Robust Handshake: Wait for frontend with a fallback timeout
                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_secs(2);
                
                loop {
                    {
                        if *FRONTEND_READY.lock().unwrap() { break; }
                    }
                    if start.elapsed() > timeout { break; }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }

                // Final safety delay
                std::thread::sleep(std::time::Duration::from_millis(100));

                let mut stdout = io::stdout();
                let _ = stdout.write_all(b"__READY__\n");
                let _ = stdout.flush();
                
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    if let Ok(cmd_json) = line {
                        if !cmd_json.trim().is_empty() {
                            let _ = handle.emit("command", cmd_json);
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_ready, 
            emit_to_backend,
            set_timezone,
            get_chart_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(feature = "python-bridge")]
#[pymodule]
fn chart_engine_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<chart::Chart>()?;
    m.add_class::<chart::Series>()?;
    m.add_class::<drawings::DrawingTool>()?;
    m.add_class::<drawings::PriceLine>()?;
    m.add_class::<trader::PaperTrader>()?;
    m.add_class::<trader::Position>()?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(time_utils::py_set_backend_timezone, m)?)?;
    m.add_function(wrap_pyfunction!(time_utils::py_ensure_timestamp, m)?)?;
    m.add_function(wrap_pyfunction!(time_utils::py_process_polars_data, m)?)?;
    m.add_function(wrap_pyfunction!(indicators::py_get_indicator_schemas, m)?)?;
    Ok(())
}
