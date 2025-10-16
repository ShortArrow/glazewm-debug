// Main TUI application
// Coordinates terminal UI and event handling

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::app::AppState;
use crate::tui::{InputHandler, Renderer};

/// Display mode for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Detailed view with full information
    Detailed,
    /// Compact tree-style view
    Compact,
}

/// Main TUI application that manages the terminal interface
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    renderer: Renderer,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Result<Self, TuiError> {
        // Force color support detection
        std::env::remove_var("NO_COLOR");

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            renderer: Renderer::new(),
        })
    }

    /// Run the TUI application with parallel input handling and rendering
    pub async fn run(&mut self, state: AppState) -> Result<(), TuiError> {
        debug!("Starting TUI application with parallel processing");

        // Start input handling task
        let input_state = state.clone();
        let input_handle =
            tokio::spawn(async move { Self::handle_input_events(input_state).await });

        // Start rendering loop
        let render_result = self.render_loop(state.clone()).await;

        // Cancel input handling when rendering loop exits
        input_handle.abort();

        debug!("TUI application finished");
        render_result
    }

    /// Handle input events in a separate task
    async fn handle_input_events(state: AppState) -> Result<(), TuiError> {
        let input_handler = InputHandler::new();
        let mut last_toggle_time = std::time::Instant::now() - Duration::from_millis(500); // Initialize to allow immediate first toggle
        const DEBOUNCE_DURATION: Duration = Duration::from_millis(200); // 200ms debounce

        debug!("Starting input event handler with debounce");

        loop {
            // Check if application should stop
            if !state.is_running().await {
                debug!("Input handler stopping - application stopped");
                break;
            }

            // Poll for events with very short timeout for maximum responsiveness
            if event::poll(Duration::from_millis(20))? {
                if let Event::Key(key) = event::read()? {
                    let action = input_handler.handle_key(key);

                    match action {
                        InputAction::Quit => {
                            debug!("User requested quit");
                            state.stop().await;
                            break;
                        }
                        InputAction::Refresh => {
                            debug!("User requested refresh");
                            // The update loop will handle the refresh
                        }
                        InputAction::ToggleMode => {
                            let now = std::time::Instant::now();
                            if now.duration_since(last_toggle_time) >= DEBOUNCE_DURATION {
                                debug!("User toggled display mode (debounced)");
                                state.toggle_display_mode().await;
                                last_toggle_time = now;
                                debug!(
                                    "Display mode toggled to: {:?}",
                                    state.get_display_mode().await
                                );
                            } else {
                                debug!("Toggle ignored (debounce active)");
                            }
                        }
                        InputAction::None => {
                            // No action needed
                        }
                    }
                }
            } else {
                // Small sleep to prevent busy waiting
                sleep(Duration::from_millis(5)).await;
            }
        }

        debug!("Input event handler finished");
        Ok(())
    }

    /// Main rendering loop
    async fn render_loop(&mut self, state: AppState) -> Result<(), TuiError> {
        debug!("Starting render loop");

        loop {
            // Check if application should stop
            if !state.is_running().await {
                debug!("Render loop stopping - application stopped");
                break;
            }

            // Get current state
            let monitors = state.get_monitors().await;
            let display_mode = state.get_display_mode().await;

            // Render frame
            self.terminal.draw(|frame| {
                self.renderer.render(frame, &monitors, display_mode);
            })?;

            // 60fps rendering (16ms per frame)
            sleep(Duration::from_millis(16)).await;
        }

        debug!("Render loop finished");
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Actions that can be triggered by user input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    /// Exit the application
    Quit,
    /// Force refresh the data
    Refresh,
    /// Toggle display mode between detailed and compact
    ToggleMode,
    /// No action
    None,
}

/// TUI-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Terminal setup failed")]
    TerminalSetup,

    #[error("Rendering failed")]
    RenderingFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_define_input_actions() {
        assert_eq!(InputAction::Quit, InputAction::Quit);
        assert_ne!(InputAction::Quit, InputAction::Refresh);
        assert_ne!(InputAction::Refresh, InputAction::None);
    }

    #[tokio::test]
    async fn should_create_app_state() {
        let state = AppState::new();
        assert!(state.is_running().await);

        state.stop().await;
        assert!(!state.is_running().await);
    }
}
