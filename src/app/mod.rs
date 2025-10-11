// Application layer module
// Manages state and coordinates between CLI and TUI layers

pub mod state;
pub mod update;

pub use state::AppState;
pub use update::UpdateLoop;