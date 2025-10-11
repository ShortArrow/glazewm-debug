// Input handling for TUI
// Processes keyboard events and converts them to actions

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::InputAction;

/// Handles keyboard input and converts to application actions
pub struct InputHandler;

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self
    }

    /// Handle a key event and return the corresponding action
    pub fn handle_key(&self, key: KeyEvent) -> InputAction {
        match key.code {
            // Quit commands
            KeyCode::Char('q') | KeyCode::Char('Q') => InputAction::Quit,
            KeyCode::Esc => InputAction::Quit,

            // Quit with Ctrl+C
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                InputAction::Quit
            }

            // Refresh commands
            KeyCode::Char('r') | KeyCode::Char('R') => InputAction::Refresh,

            // Future: Navigation keys could be added here
            // KeyCode::Up => InputAction::NavigateUp,
            // KeyCode::Down => InputAction::NavigateDown,
            // KeyCode::Left => InputAction::NavigateLeft,
            // KeyCode::Right => InputAction::NavigateRight,

            // Unknown key
            _ => InputAction::None,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_event_with_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn should_handle_quit_keys() {
        let handler = InputHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('q'))),
            InputAction::Quit
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('Q'))),
            InputAction::Quit
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Esc)),
            InputAction::Quit
        );
        assert_eq!(
            handler.handle_key(key_event_with_ctrl(KeyCode::Char('c'))),
            InputAction::Quit
        );
    }

    #[test]
    fn should_handle_refresh_keys() {
        let handler = InputHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('r'))),
            InputAction::Refresh
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('R'))),
            InputAction::Refresh
        );
    }

    #[test]
    fn should_handle_unknown_keys() {
        let handler = InputHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('x'))),
            InputAction::None
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Enter)),
            InputAction::None
        );
    }
}
