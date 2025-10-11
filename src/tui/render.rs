// Rendering logic for TUI
// Converts domain models into visual representation

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::domain::Monitor;

/// Renders the application state to the terminal
pub struct Renderer;

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self
    }

    /// Render the application state to the given frame
    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, monitors: &[Monitor]) {
        let size = frame.size();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Render header
        self.render_header(frame, chunks[0], monitors);

        // Render main content
        if monitors.is_empty() {
            self.render_no_data(frame, chunks[1]);
        } else {
            self.render_monitors(frame, chunks[1], monitors);
        }

        // Render footer
        self.render_footer(frame, chunks[2]);
    }

    /// Render the header with application title and stats
    fn render_header<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, monitors: &[Monitor]) {
        let monitor_count = monitors.len();
        let total_windows: usize = monitors.iter().map(|m| m.total_window_count()).sum();

        let header_text = format!(
            "glazewm-debug v{} | Monitors: {} | Windows: {}",
            env!("CARGO_PKG_VERSION"),
            monitor_count,
            total_windows
        );

        let header = Paragraph::new(header_text)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("glazewm State Viewer")
                    .style(Style::default().fg(Color::Cyan)),
            );

        frame.render_widget(header, area);
    }

    /// Render the footer with keyboard shortcuts
    fn render_footer<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let footer_text = "q/Esc: Quit | r: Refresh | Ctrl+C: Force Quit";

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(footer, area);
    }

    /// Render a message when no data is available
    fn render_no_data<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let no_data_text = vec![
            Spans::from("No monitors found."),
            Spans::from(""),
            Spans::from("Make sure glazewm is running and accessible."),
            Spans::from("Check the glazewm executable path in your configuration."),
        ];

        let no_data = Paragraph::new(no_data_text)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("No Data")
                    .style(Style::default().fg(Color::Yellow)),
            );

        frame.render_widget(no_data, area);
    }

    /// Render the list of monitors and their workspaces
    fn render_monitors<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, monitors: &[Monitor]) {
        let mut items = Vec::new();

        for monitor in monitors {
            // Monitor header
            let monitor_style = if monitor.is_focused() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let monitor_info = format!(
                "🖥️  Monitor {} ({}x{}) - {} windows",
                monitor.id(),
                monitor.geometry().size.width,
                monitor.geometry().size.height,
                monitor.total_window_count()
            );

            items.push(ListItem::new(Spans::from(Span::styled(
                monitor_info,
                monitor_style,
            ))));

            // Workspaces for this monitor
            for workspace in monitor.workspaces() {
                let workspace_style = if workspace.is_focused() {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let workspace_prefix = if workspace.is_focused() {
                    "  ▶ "
                } else {
                    "    "
                };
                let workspace_info = format!(
                    "{}📁 {} ({} windows) - {}",
                    workspace_prefix,
                    workspace.name(),
                    workspace.window_count(),
                    if workspace.is_visible() {
                        "visible"
                    } else {
                        "hidden"
                    }
                );

                items.push(ListItem::new(Spans::from(Span::styled(
                    workspace_info,
                    workspace_style,
                ))));

                // Windows in this workspace
                for window in workspace.windows() {
                    let window_style = if window.is_focused() {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    };

                    let window_prefix = if window.is_focused() {
                        "      ● "
                    } else {
                        "      ○ "
                    };
                    let window_info =
                        format!("{}{}", window_prefix, window.display_name_truncated(60));

                    items.push(ListItem::new(Spans::from(Span::styled(
                        window_info,
                        window_style,
                    ))));
                }

                // Add spacing between workspaces
                items.push(ListItem::new(Spans::from("")));
            }
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Monitors & Workspaces")
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        values::{MonitorId, Position, Rectangle, Size, WindowId, WorkspaceId},
        DisplayState, FocusState, TilingDirection, Window, WindowState, Workspace,
    };

    fn create_test_monitor() -> Monitor {
        let window = Window::new(
            WindowId::new("test-window".to_string()),
            "VS Code".to_string(),
            "Code".to_string(),
            Rectangle::new(Position::new(0, 0), Size::new(800, 600)),
            WindowState::Tiling,
            FocusState::Focused,
            DisplayState::Shown,
        );

        let workspace = Workspace::new(
            WorkspaceId::new("test-workspace".to_string()),
            "Development".to_string(),
            vec![window],
            TilingDirection::Horizontal,
            FocusState::Focused,
            DisplayState::Shown,
        );

        Monitor::new(
            MonitorId::new("test-monitor".to_string()),
            Rectangle::new(Position::new(0, 0), Size::new(1920, 1080)),
            vec![workspace],
            FocusState::Focused,
            96,
            1.0,
        )
    }

    #[test]
    fn should_create_renderer() {
        let _renderer = Renderer::new();
    }

    #[test]
    fn should_use_default() {
        let _renderer = Renderer::default();
    }

    // Note: Full rendering tests would require a mock terminal,
    // which is complex to set up. The rendering logic is tested
    // indirectly through integration tests.
}
