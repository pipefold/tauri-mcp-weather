use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use tauri::command;

// Struct to hold our MCP server process
pub struct McpServerState {
    pub process: Mutex<Option<Child>>,
}

// Start the MCP server as a child process
#[command]
pub fn start_mcp_server(state: tauri::State<'_, McpServerState>) -> Result<String, String> {
    // Check if server is already running
    let mut process_guard = match state.process.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("Failed to acquire lock on server state".to_string()),
    };

    if process_guard.is_some() {
        return Ok("MCP server is already running".to_string());
    }

    // Use absolute path to the script
    let node_script_path = std::env::current_dir()
        .unwrap()
        .join("src-tauri/src/mcp_weather_server.js")
        .to_string_lossy()
        .to_string();

    // Log path for debugging
    println!("Starting MCP server with script: {}", node_script_path);

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
            *process_guard = Some(child);

            // Drop the guard to release the lock
            drop(process_guard);

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
    let mut process_guard = match state.process.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("Failed to acquire lock on server state".to_string()),
    };

    if let Some(mut process) = process_guard.take() {
        match process.kill() {
            Ok(_) => Ok("MCP server stopped".to_string()),
            Err(e) => {
                // Put the process back since we couldn't kill it
                *process_guard = Some(process);
                Err(format!("Failed to stop MCP server: {}", e))
            }
        }
    } else {
        Ok("MCP server was not running".to_string())
    }
}

// Send message to MCP server's stdin
#[command]
pub fn send_to_mcp_server(
    message: String,
    state: tauri::State<'_, McpServerState>,
) -> Result<String, String> {
    println!("Attempting to send message to MCP server: {}", message);

    let mut process_guard = match state.process.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("Failed to acquire lock on server state".to_string()),
    };

    if let Some(ref mut process) = *process_guard {
        println!("Process found, attempting to write to stdin");
        if let Some(stdin) = process.stdin.as_mut() {
            match stdin.write_all(message.as_bytes()) {
                Ok(_) => {
                    println!("Message successfully sent to MCP server");
                    Ok("Message sent to MCP server".to_string())
                }
                Err(e) => {
                    println!("Failed to write to stdin: {}", e);
                    Err(format!("Failed to send message to MCP server: {}", e))
                }
            }
        } else {
            println!("Failed to get stdin handle");
            Err("Failed to get stdin of MCP server".to_string())
        }
    } else {
        println!("No process found in state");
        Err("MCP server not running".to_string())
    }
}
