// glazewm-debug main entry point
//
// Application bootstrap following composition root pattern.
// Handles CLI argument parsing and dependency injection.

use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use glazewm_debug::{AppState, TuiApp, UpdateConfig, UpdateLoop};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::select;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    name = "glazewm-debug",
    about = "A CLI+JSON TUI debugger for glazewm window manager state visualization",
    version
)]
struct Args {
    /// Color test mode - test terminal color support with crossterm
    #[arg(long)]
    color_test: bool,

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
    #[arg(long, default_value = "10000")]
    timeout: u64,

    /// Run in demo mode with sample data (no glazewm required)
    #[arg(long)]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Handle color test mode
    if args.color_test {
        test_colors().await?;
        return Ok(());
    }

    // Initialize logging
    init_logging(args.quiet);

    info!(
        "Starting glazewm-debug v{} (CLI+JSON architecture)",
        env!("CARGO_PKG_VERSION")
    );

    // Create application state
    let state = AppState::new();

    // Create update loop configuration
    let update_config = UpdateConfig {
        refresh_interval: Duration::from_millis(args.refresh_rate),
        command_timeout: Duration::from_millis(args.timeout),
        glazewm_path: args.glazewm_path,
    };

    // Create update loop (demo mode or real mode)
    let update_loop = if args.demo {
        info!("Running in demo mode with sample data");
        UpdateLoop::new_demo(update_config, state.clone())
    } else {
        UpdateLoop::new(update_config, state.clone())
    };

    // Create TUI application
    let mut tui_app = match TuiApp::new() {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to initialize TUI: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    info!("Application started successfully");

    // Perform initial data load before starting TUI
    info!("Loading initial glazewm state...");
    if let Err(e) = update_loop.update_now().await {
        error!("Failed to load initial state: {}", e);
        // Continue anyway - TUI will show "No Data" message
    } else {
        info!("Initial state loaded successfully");
    }

    // Run both the update loop and TUI concurrently
    let result = select! {
        update_result = update_loop.run() => {
            match update_result {
                Ok(()) => {
                    info!("Update loop finished normally");
                    Ok(())
                }
                Err(e) => {
                    error!("Update loop error: {}", e);
                    Err(Box::new(e) as Box<dyn std::error::Error>)
                }
            }
        }
        tui_result = tui_app.run(state.clone()) => {
            match tui_result {
                Ok(()) => {
                    info!("TUI finished normally");
                    Ok(())
                }
                Err(e) => {
                    error!("TUI error: {}", e);
                    Err(Box::new(e) as Box<dyn std::error::Error>)
                }
            }
        }
    };

    info!("Application shutting down");
    result
}

async fn test_colors() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();

    println!("=== Crossterm Color Test ===");
    println!();

    // Test basic colors available in crossterm
    let colors = [
        (Color::Red, "Red"),
        (Color::Green, "Green"),
        (Color::Yellow, "Yellow"),
        (Color::Blue, "Blue"),
        (Color::Magenta, "Magenta"),
        (Color::Cyan, "Cyan"),
        (Color::Grey, "Grey"),
        (Color::DarkGrey, "DarkGrey"),
        (Color::White, "White"),
        (Color::Black, "Black"),
    ];

    println!("Basic 16 colors:");
    for (color, name) in &colors {
        execute!(
            stdout,
            SetForegroundColor(*color),
            Print(format!("{:12} ", name)),
            ResetColor
        )?;
    }
    println!();
    println!();

    // Test RGB colors (what we tried to use)
    println!("RGB colors:");
    let rgb_colors = [
        (Color::Rgb { r: 255, g: 0, b: 0 }, "RGB Red"),
        (Color::Rgb { r: 0, g: 255, b: 0 }, "RGB Green"),
        (
            Color::Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
            "RGB Yellow",
        ),
        (Color::Rgb { r: 0, g: 0, b: 255 }, "RGB Blue"),
        (
            Color::Rgb {
                r: 255,
                g: 0,
                b: 255,
            },
            "RGB Magenta",
        ),
        (
            Color::Rgb {
                r: 0,
                g: 255,
                b: 255,
            },
            "RGB Cyan",
        ),
    ];

    for (color, name) in &rgb_colors {
        execute!(
            stdout,
            SetForegroundColor(*color),
            Print(format!("{:12} ", name)),
            ResetColor
        )?;
    }
    println!();
    println!();

    // Test the specific colors we're trying to use in the app
    println!("App colors:");
    execute!(
        stdout,
        SetForegroundColor(Color::Red),
        Print("Active Monitor (Red) "),
        ResetColor
    )?;
    execute!(
        stdout,
        SetForegroundColor(Color::Blue),
        Print("Inactive Monitor (Blue) "),
        ResetColor
    )?;
    execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print("Active Workspace (Green) "),
        ResetColor
    )?;
    execute!(
        stdout,
        SetForegroundColor(Color::Grey),
        Print("Inactive Workspace (Grey) "),
        ResetColor
    )?;
    execute!(
        stdout,
        SetForegroundColor(Color::Magenta),
        Print("Focused Window (Magenta) "),
        ResetColor
    )?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("Normal Window (Cyan)"),
        ResetColor
    )?;
    println!();
    println!();

    println!("If you see colors above, the issue is in ratatui rendering, not crossterm.");
    println!("If you see only white/gray text, the issue is in crossterm/terminal support.");

    Ok(())
}

fn init_logging(quiet: bool) {
    use tracing_subscriber::FmtSubscriber;

    let level = if quiet {
        tracing::Level::ERROR
    } else {
        std::env::var("RUST_LOG")
            .map(|_| tracing::Level::DEBUG)
            .unwrap_or(tracing::Level::INFO)
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
