use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use tauri::command;

// Struct to hold our MCP server process
pub struct McpServerState {
    pub process: Option<Mutex<std::process::Child>>,
}

// Start the MCP server as a child process
#[command]
pub fn start_mcp_server() -> Result<String, String> {
    let node_script_path = "./src-tauri/src/mcp_weather_server.js";

    // Start Node.js process with the MCP server script
    match Command::new("node")
        .arg(node_script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            // Log server started
            let pid = child.id();

            // Start a thread to log stdout/stderr
            let mut stderr = BufReader::new(child.stderr.unwrap());
            let mut stdout = BufReader::new(child.stdout.unwrap());

            thread::spawn(move || {
                let mut line = String::new();
                while stdout.read_line(&mut line).unwrap() > 0 {
                    println!("MCP SERVER OUT: {}", line.trim());
                    line.clear();
                }
            });

            thread::spawn(move || {
                let mut line = String::new();
                while stderr.read_line(&mut line).unwrap() > 0 {
                    eprintln!("MCP SERVER ERR: {}", line.trim());
                    line.clear();
                }
            });

            Ok(format!("MCP server started with PID: {}", pid))
        }
        Err(e) => Err(format!("Failed to start MCP server: {}", e)),
    }
}

// Stop the MCP server
#[command]
pub fn stop_mcp_server(state: tauri::State<'_, McpServerState>) -> Result<String, String> {
    if let Some(process_mutex) = &state.process {
        if let Ok(mut process) = process_mutex.lock() {
            match process.kill() {
                Ok(_) => Ok("MCP server stopped".to_string()),
                Err(e) => Err(format!("Failed to stop MCP server: {}", e)),
            }
        } else {
            Err("Failed to acquire lock on MCP server process".to_string())
        }
    } else {
        Err("MCP server not running".to_string())
    }
}

// Send message to MCP server's stdin
#[command]
pub fn send_to_mcp_server(
    message: String,
    state: tauri::State<'_, McpServerState>,
) -> Result<String, String> {
    if let Some(process_mutex) = &state.process {
        if let Ok(mut process) = process_mutex.lock() {
            if let Some(stdin) = process.stdin.as_mut() {
                match stdin.write_all(message.as_bytes()) {
                    Ok(_) => Ok("Message sent to MCP server".to_string()),
                    Err(e) => Err(format!("Failed to send message to MCP server: {}", e)),
                }
            } else {
                Err("Failed to get stdin of MCP server".to_string())
            }
        } else {
            Err("Failed to acquire lock on MCP server process".to_string())
        }
    } else {
        Err("MCP server not running".to_string())
    }
}
