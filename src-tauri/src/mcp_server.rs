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
pub fn start_mcp_server(state: tauri::State<'_, McpServerState>) -> Result<String, String> {
    // Check if server is already running
    if let Some(mutex) = &state.process {
        if let Ok(guard) = mutex.lock() {
            if guard.is_some() {
                return Ok("MCP server is already running".to_string());
            }
        }
    }

    let node_script_path = "./src-tauri/src/mcp_weather_server.js";

    // Start Node.js process with the MCP server script
    match Command::new("node")
        .arg(node_script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            // Log server started
            let pid = child.id();

            // Get handles to stdout/stderr before moving child into state
            let stderr = child.stderr.take().unwrap();
            let stdout = child.stdout.take().unwrap();

            // Store the process in the state
            if let Some(mutex) = &state.process {
                if let Ok(mut guard) = mutex.lock() {
                    *guard = Some(child);
                } else {
                    return Err("Failed to acquire lock to store MCP server process".to_string());
                }
            } else {
                return Err("MCP server state is not initialized".to_string());
            }

            // Start a thread to log stdout
            thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                    println!("MCP SERVER OUT: {}", line.trim());
                    line.clear();
                }
            });

            // Start a thread to log stderr
            thread::spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 {
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
        if let Ok(mut process_guard) = process_mutex.lock() {
            if let Some(mut process) = process_guard.take() {
                match process.kill() {
                    Ok(_) => {
                        // Process is now removed from state
                        Ok("MCP server stopped".to_string())
                    }
                    Err(e) => {
                        // Put the process back since we couldn't kill it
                        *process_guard = Some(process);
                        Err(format!("Failed to stop MCP server: {}", e))
                    }
                }
            } else {
                Ok("MCP server was not running".to_string())
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
