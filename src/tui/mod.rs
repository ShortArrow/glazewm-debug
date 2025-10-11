// TUI layer module
// Handles terminal user interface using ratatui

pub mod app;
pub mod input;
pub mod render;

pub use app::TuiApp;
pub use input::InputHandler;
pub use render::Renderer;
