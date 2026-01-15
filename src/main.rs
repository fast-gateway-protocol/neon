//! FGP daemon for Neon (serverless Postgres) operations.
//!
//! Uses Neon HTTP API directly for low-latency database operations.
//!
//! # Usage
//! ```bash
//! fgp-neon start           # Start daemon in background
//! fgp-neon start -f        # Start in foreground
//! fgp-neon stop            # Stop daemon
//! fgp-neon status          # Check daemon status
//! ```

mod api;
mod models;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::{cleanup_socket, FgpServer};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::service::NeonService;

/// Neonctl credentials file structure.
#[derive(Deserialize)]
struct NeonctlCredentials {
    access_token: String,
}

/// Get Neon credentials from env var or neonctl config.
fn get_neon_credentials() -> Result<String> {
    // Try NEON_API_KEY first
    if let Ok(key) = std::env::var("NEON_API_KEY") {
        return Ok(key);
    }

    // Fall back to neonctl OAuth token
    let creds_path = shellexpand::tilde("~/.config/neonctl/credentials.json").to_string();
    let creds_json = std::fs::read_to_string(&creds_path).context(
        "No NEON_API_KEY set and neonctl credentials not found. Run `neonctl auth` first.",
    )?;

    let creds: NeonctlCredentials =
        serde_json::from_str(&creds_json).context("Failed to parse neonctl credentials")?;

    Ok(creds.access_token)
}

const DEFAULT_SOCKET: &str = "~/.fgp/services/neon/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-neon")]
#[command(about = "FGP daemon for Neon serverless Postgres operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/neon/daemon.sock)
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,

        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the running daemon
    Stop {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },

    /// Check daemon status
    Status {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, foreground } => cmd_start(socket, foreground),
        Commands::Stop { socket } => cmd_stop(socket),
        Commands::Status { socket } => cmd_status(socket),
    }
}

fn cmd_start(socket: String, foreground: bool) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    // Create parent directory
    if let Some(parent) = Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    // Get API key BEFORE fork (credentials access needs parent process)
    let api_key = get_neon_credentials()?;

    // Get org_id from environment (required)
    let org_id = std::env::var("NEON_ORG_ID").context(
        "NEON_ORG_ID environment variable not set. Run `neonctl orgs list` to find your org_id.",
    )?;

    let pid_file = format!("{}.pid", socket_path);

    println!("Starting fgp-neon daemon...");
    println!("Socket: {}", socket_path);
    println!("Org ID: {}", org_id);

    if foreground {
        // Foreground mode - initialize logging and run directly
        tracing_subscriber::fmt()
            .with_env_filter("fgp_neon=debug,fgp_daemon=debug")
            .init();

        let service = NeonService::new(api_key, org_id).context("Failed to create NeonService")?;
        let server =
            FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
        server.serve().context("Server error")?;
    } else {
        // Background mode - daemonize first, THEN create service
        // Tokio runtime must be created AFTER fork
        use daemonize::Daemonize;

        let daemonize = Daemonize::new()
            .pid_file(&pid_file)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {
                // Child process: initialize logging and run server
                tracing_subscriber::fmt()
                    .with_env_filter("fgp_neon=debug,fgp_daemon=debug")
                    .init();

                let service =
                    NeonService::new(api_key, org_id).context("Failed to create NeonService")?;
                let server =
                    FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
                server.serve().context("Server error")?;
            }
            Err(e) => {
                eprintln!("Failed to daemonize: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn cmd_stop(socket: String) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();
    let pid_file = format!("{}.pid", socket_path);

    if Path::new(&socket_path).exists() {
        if let Ok(client) = fgp_daemon::FgpClient::new(&socket_path) {
            if let Ok(response) = client.stop() {
                if response.ok {
                    println!("Daemon stopped.");
                    return Ok(());
                }
            }
        }
    }

    // Read PID
    let pid_str = std::fs::read_to_string(&pid_file)
        .context("Failed to read PID file - daemon may not be running")?;
    let pid: i32 = pid_str.trim().parse().context("Invalid PID in file")?;

    if !pid_matches_process(pid, "fgp-neon") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-neon daemon (PID: {})...", pid);

    // Send SIGTERM
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Wait a moment for cleanup
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Cleanup files
    let _ = cleanup_socket(&socket_path, Some(Path::new(&pid_file)));
    let _ = std::fs::remove_file(&pid_file);

    println!("Daemon stopped.");

    Ok(())
}

fn pid_matches_process(pid: i32, expected_name: &str) -> bool {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let command = String::from_utf8_lossy(&output.stdout);
            command.trim().contains(expected_name)
        }
        _ => false,
    }
}

fn cmd_status(socket: String) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    // Check if socket exists
    if !Path::new(&socket_path).exists() {
        println!("Status: NOT RUNNING");
        println!("Socket {} does not exist", socket_path);
        return Ok(());
    }

    // Try to connect and send health check
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            // Send health request
            let request = r#"{"id":"status","v":1,"method":"health","params":{}}"#;
            writeln!(stream, "{}", request)?;
            stream.flush()?;

            // Read response
            let mut reader = BufReader::new(stream);
            let mut response = String::new();
            reader.read_line(&mut response)?;

            println!("Status: RUNNING");
            println!("Socket: {}", socket_path);
            println!("Health: {}", response.trim());
        }
        Err(e) => {
            println!("Status: NOT RESPONDING");
            println!("Socket exists but connection failed: {}", e);
        }
    }

    Ok(())
}
