// glazewm-debug library
// 
// A CLI+JSON based TUI debugger for glazewm window manager state visualization.
// Built following Domain-Driven Design, Test-Driven Development, and UNIX philosophy.

pub mod domain;
pub mod cli; 
pub mod app;
pub mod tui;
pub mod config;

// Re-export commonly used types
pub use domain::{Monitor, Workspace, Window, FocusState, WindowState, DisplayState};
pub use cli::GlazewmClient;
pub use app::App;

// Error types
pub use domain::DomainError;
pub use cli::CliError;