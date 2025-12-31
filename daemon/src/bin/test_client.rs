//! IPC Test Client
//!
//! CLI tool for testing the daemon's IPC layer.
//! Usage:
//!   test_client status        - Get current mode
//!   test_client set-mode <mode> - Change mode (idle|dictation|intelligent|agent)
//!   test_client watch         - Stream all events (heartbeats, dictation, etc.)

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(format!(
        "{}/.local/share/second-brain/daemon.sock",
        home
    ))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "set-mode" => {
            if args.len() < 3 {
                eprintln!("Error: set-mode requires a mode argument");
                eprintln!("Usage: test_client set-mode <idle|dictation|intelligent|agent>");
                std::process::exit(1);
            }
            set_mode(&args[2]);
        }
        "watch" => watch_events(),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("IPC Test Client");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  test_client set-mode <mode>  - Change mode (idle|dictation|intelligent|agent)");
    eprintln!("  test_client watch            - Stream all events");
    eprintln!("  test_client help             - Show this help");
}

fn connect() -> UnixStream {
    let path = socket_path();
    println!("Connecting to {:?}...", path);

    match UnixStream::connect(&path) {
        Ok(stream) => {
            println!("Connected!");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            eprintln!("Is the daemon running?");
            std::process::exit(1);
        }
    }
}

fn send_command(stream: &mut UnixStream, command: &str) {
    println!("Sending: {}", command);
    writeln!(stream, "{}", command).expect("Failed to write to socket");
    stream.flush().expect("Failed to flush socket");
}

fn read_response(stream: &mut UnixStream) -> String {
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read response");
    line.trim().to_string()
}

fn set_mode(mode: &str) {
    // Validate mode
    let valid_modes = ["idle", "dictation", "intelligent", "agent"];
    if !valid_modes.contains(&mode) {
        eprintln!("Invalid mode: {}", mode);
        eprintln!("Valid modes: {:?}", valid_modes);
        std::process::exit(1);
    }

    let mut stream = connect();

    // Build the command
    let command = format!(
        r#"{{"id":"cli-1","kind":"command","type":"SET_MODE","payload":{{"mode":"{}"}}}}"#,
        mode
    );

    send_command(&mut stream, &command);

    // Read response
    let response = read_response(&mut stream);
    println!("Response: {}", response);

    // Pretty print if it's valid JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
            println!("\nFormatted:");
            println!("{}", pretty);
        }
    }
}

fn watch_events() {
    let stream = connect();
    let reader = BufReader::new(stream);

    println!("Watching for events (Ctrl+C to stop)...\n");

    for line in reader.lines() {
        match line {
            Ok(text) if !text.is_empty() => {
                // Try to parse and pretty-print
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    let kind = json.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                    let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                    
                    // Color code by kind
                    let prefix = match kind {
                        "event" => "\x1b[34m[EVENT]\x1b[0m",
                        "response" => "\x1b[32m[RESPONSE]\x1b[0m",
                        "error" => "\x1b[31m[ERROR]\x1b[0m",
                        _ => "[???]",
                    };

                    println!("{} {} {}", prefix, msg_type, text);
                } else {
                    println!("[RAW] {}", text);
                }
            }
            Ok(_) => {} // Empty line
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    println!("\nConnection closed.");
}
