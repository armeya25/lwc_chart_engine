#![allow(dead_code)]
#[path = "../../../chart_backend/src/chart.rs"] mod chart;
#[path = "../../../chart_backend/src/drawings.rs"] mod drawings;
#[path = "../../../chart_backend/src/types.rs"] mod types;
#[path = "../../../chart_backend/src/time_utils.rs"] mod time_utils;

use tauri::{Emitter};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::io::{self, BufRead, Write};
use serde_json::Value;

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
    time_utils::py_set_backend_timezone(tz).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chart_state() -> Value {
    // Example usage of chart/types
    serde_json::to_value(types::ChartCommand::new("get_state", "chart-0")).unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
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
