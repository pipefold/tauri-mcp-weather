// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod mcp_server;

use mcp_server::{send_to_mcp_server, start_mcp_server, stop_mcp_server, McpServerState};
use std::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_mcp_server,
            stop_mcp_server,
            send_to_mcp_server
        ])
        .manage(McpServerState {
            process: Some(Mutex::new(None)),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
