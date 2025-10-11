// glazewm-debug library
//
// A CLI+JSON based TUI debugger for glazewm window manager state visualization.
// Built following Domain-Driven Design, Test-Driven Development, and UNIX philosophy.

pub mod app;
pub mod cli;
pub mod domain;
pub mod tui;
// pub mod config;

// Re-export commonly used types
pub use app::update::UpdateConfig;
pub use app::{AppState, UpdateLoop};
pub use cli::{GlazewmClient, GlazewmParser};
pub use domain::{DisplayState, FocusState, Monitor, Window, WindowState, Workspace};
pub use tui::TuiApp;

// Error types
pub use cli::CliError;
pub use domain::DomainError;
