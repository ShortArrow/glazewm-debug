// glazewm-debug main entry point
//
// Application bootstrap following composition root pattern.
// Handles CLI argument parsing and dependency injection.

use clap::Parser;
use glazewm_debug::{App, GlazewmClient};
use std::path::PathBuf;
use std::process;
use tracing::{info, error};

#[derive(Parser, Debug)]
#[command(
    name = "glazewm-debug",
    about = "A CLI+JSON TUI debugger for glazewm window manager state visualization",
    version
)]
struct Args {
    /// Refresh interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    refresh_rate: u64,

    /// Minimal output mode
    #[arg(short, long)]
    quiet: bool,

    /// Path to glazewm executable
    #[arg(long, default_value = "glazewm")]
    glazewm_path: PathBuf,

    /// Command timeout in milliseconds
    #[arg(long, default_value = "5000")]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    init_logging(args.quiet);

    // Log startup
    info!("Starting glazewm-debug v{}", env!("CARGO_PKG_VERSION"));
    info!("Using glazewm at: {:?}", args.glazewm_path);
    info!("Refresh rate: {}ms", args.refresh_rate);

    // Create glazewm client
    let client = GlazewmClient::new(args.glazewm_path)
        .with_timeout(std::time::Duration::from_millis(args.timeout));

    // Verify glazewm is available
    if !client.is_available().await {
        error!("glazewm not found or not responding");
        error!("Please ensure glazewm is installed and available in PATH");
        error!("Test with: glazewm --version");
        process::exit(1);
    }

    // Create and run application
    let mut app = App::new(client, args.refresh_rate);

    if let Err(e) = app.run().await {
        error!("Application error: {}", e);
        process::exit(1);
    }

    info!("glazewm-debug shutdown complete");
}

fn init_logging(quiet: bool) {
    let level = if quiet {
        tracing::Level::ERROR
    } else {
        std::env::var("RUST_LOG")
            .map(|_| tracing::Level::DEBUG)
            .unwrap_or(tracing::Level::INFO)
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
}