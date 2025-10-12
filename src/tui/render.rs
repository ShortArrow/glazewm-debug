// Rendering logic for TUI
// Converts domain models into visual representation

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::domain::{Monitor, Workspace};
use crate::tui::DisplayMode;
use crate::utils::text_width::{Alignment, TextWidthCalculator};
use std::collections::HashMap;

/// Renders the application state to the terminal
pub struct Renderer;

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self
    }

    /// Get monitor style based on focus state (for testing)  
    pub fn get_monitor_style(is_focused: bool) -> Style {
        if is_focused {
            Style::default()
                .fg(Color::Red) // Use red for active monitor (highly visible)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Blue)
        }
    }

    /// Get workspace style based on focus state (for testing)
    pub fn get_workspace_style(is_focused: bool) -> Style {
        if is_focused {
            Style::default()
                .fg(Color::Green) // Use green for active workspace
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    }

    /// Get window style based on focus state (for testing)
    pub fn get_window_style(is_focused: bool) -> Style {
        if is_focused {
            Style::default()
                .fg(Color::Magenta) // Use magenta for focused window
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        }
    }

    /// Render the application state to the given frame
    pub fn render(
        &self,
        frame: &mut Frame,
        monitors: &[Monitor],
        mode: DisplayMode,
    ) {
        let size = frame.area();

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
        self.render_header(frame, chunks[0], monitors, mode);

        // Render main content
        if monitors.is_empty() {
            self.render_no_data(frame, chunks[1]);
        } else {
            match mode {
                DisplayMode::Detailed => self.render_monitors_detailed(frame, chunks[1], monitors),
                DisplayMode::Compact => self.render_monitors_compact(frame, chunks[1], monitors),
            }
        }

        // Render footer
        self.render_footer(frame, chunks[2]);
    }

    /// Render the header with application title and stats
    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitors: &[Monitor],
        mode: DisplayMode,
    ) {
        let monitor_count = monitors.len();
        let total_windows: usize = monitors.iter().map(|m| m.total_window_count()).sum();

        let mode_text = match mode {
            DisplayMode::Detailed => "Detailed",
            DisplayMode::Compact => "Compact",
        };

        let header_text = format!(
            "glazewm-debug v{} | Monitors: {} | Windows: {} | Mode: {}",
            env!("CARGO_PKG_VERSION"),
            monitor_count,
            total_windows,
            mode_text
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
                    .border_style(Style::default().fg(Color::Blue)), // Basic blue
            );

        frame.render_widget(header, area);
    }

    /// Render the footer with keyboard shortcuts
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = "q/Esc: Quit | r: Refresh | c: Toggle Mode | Ctrl+C: Force Quit";

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(footer, area);
    }

    /// Render a message when no data is available
    fn render_no_data(&self, frame: &mut Frame, area: Rect) {
        let no_data_text = vec![
            Line::from("No monitors found."),
            Line::from(""),
            Line::from("Make sure glazewm is running and accessible."),
            Line::from("Check the glazewm executable path in your configuration."),
        ];

        let no_data = Paragraph::new(no_data_text)
            .style(Style::default().fg(Color::LightMagenta)) // Use magenta instead of yellow
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("No Data")
                    .style(Style::default().fg(Color::LightMagenta)), // Use magenta instead of yellow
            );

        frame.render_widget(no_data, area);
    }

    /// Render the list of monitors and their workspaces (detailed mode) using proper ratatui layouts
    fn render_monitors_detailed(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitors: &[Monitor],
    ) {
        if monitors.is_empty() {
            return;
        }

        // Create vertical layout for all monitors
        let monitor_constraints: Vec<Constraint> = monitors
            .iter()
            .map(|monitor| {
                let workspace_count = monitor.workspaces().len().max(1);
                let estimated_height = workspace_count * 6 + 2; // Estimate per workspace + monitor border
                Constraint::Min(estimated_height as u16)
            })
            .collect();

        let monitor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(monitor_constraints)
            .margin(1) // Leave space for outer border
            .split(area);

        for (monitor_idx, monitor) in monitors.iter().enumerate() {
            if monitor_idx < monitor_chunks.len() {
                self.render_single_monitor_with_layout(frame, monitor_chunks[monitor_idx], monitor);
            }
        }

        // Render outer border for the entire area
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title("Monitors & Workspaces (Detailed)")
            .border_style(Style::default().fg(Color::Blue)); // Basic blue

        frame.render_widget(outer_block, area);
    }

    /// Render a single monitor using proper ratatui layout
    fn render_single_monitor_with_layout(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitor: &Monitor,
    ) {
        let monitor_style = Self::get_monitor_style(monitor.is_focused());
        
        // Debug: log the actual color being used
        tracing::debug!("Monitor {} style: {:?}", monitor.id(), monitor_style);

        let monitor_status = if monitor.is_focused() {
            " [Active]"
        } else {
            ""
        };
        let monitor_title = format!(
            "Monitor {} ({}x{}){}",
            monitor.id(),
            monitor.geometry().size.width,
            monitor.geometry().size.height,
            monitor_status
        );

        if monitor.workspaces().is_empty() {
            // Monitor with no workspaces
            let monitor_title_spans = Line::from(Span::styled(monitor_title, monitor_style));
            let empty_text = Paragraph::new("No workspaces")
                .style(Style::default().fg(Color::Gray)) // Basic gray
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(monitor_title_spans)
                        .border_style(monitor_style),
                );

            frame.render_widget(empty_text, area);
            return;
        }

        // Create layout for workspaces within this monitor
        let workspace_constraints: Vec<Constraint> = monitor
            .workspaces()
            .iter()
            .map(|workspace| {
                let window_count = workspace.windows().len().max(1);
                let estimated_height = window_count * 4 + 2; // Estimate per window + workspace border
                Constraint::Min(estimated_height as u16)
            })
            .collect();

        let workspace_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(workspace_constraints)
            .margin(1) // Leave space for monitor border
            .split(area);

        // Render monitor border first
        let monitor_title_spans = Line::from(Span::styled(monitor_title, monitor_style));
        let monitor_block = Block::default()
            .borders(Borders::ALL)
            .title(monitor_title_spans)
            .border_style(monitor_style);

        frame.render_widget(monitor_block, area);

        // Then render workspaces in the inner area
        for (ws_idx, workspace) in monitor.workspaces().iter().enumerate() {
            if ws_idx < workspace_chunks.len() {
                self.render_single_workspace_with_layout(
                    frame,
                    workspace_chunks[ws_idx],
                    workspace,
                );
            }
        }
    }

    /// Render a single workspace using proper ratatui layout
    fn render_single_workspace_with_layout(
        &self,
        frame: &mut Frame,
        area: Rect,
        workspace: &Workspace,
    ) {
        let workspace_style = Self::get_workspace_style(workspace.is_focused());
        
        // Debug: log the actual color being used  
        tracing::debug!("Workspace {} style: {:?}", workspace.name(), workspace_style);

        let workspace_status = if workspace.is_focused() {
            " [Active]"
        } else {
            ""
        };

        let workspace_title = format!("Workspace {}{}", workspace.name(), workspace_status);

        if workspace.windows().is_empty() {
            // Empty workspace
            let workspace_title_spans = Line::from(Span::styled(workspace_title, workspace_style));
            let empty_text = Paragraph::new("(Empty)")
                .style(Style::default().fg(Color::Gray)) // Basic gray
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(workspace_title_spans)
                        .border_style(workspace_style),
                );

            frame.render_widget(empty_text, area);
            return;
        }

        // Create vertical layout for windows within this workspace
        let window_constraints: Vec<Constraint> = workspace
            .windows()
            .iter()
            .map(|_| Constraint::Min(3)) // Each window needs at least 3 lines (title + content line + border)
            .collect();

        let window_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(window_constraints)
            .margin(1) // Leave space for workspace border
            .split(area);

        // Calculate window percentages
        let percentages = workspace.calculate_window_percentages();
        let percentage_map: HashMap<_, _> = percentages
            .into_iter()
            .map(|(id, percentage)| (id, percentage as f64))
            .collect();

        // Render workspace border first
        let workspace_title_spans = Line::from(Span::styled(workspace_title, workspace_style));
        let workspace_block = Block::default()
            .borders(Borders::ALL)
            .title(workspace_title_spans)
            .border_style(workspace_style);

        frame.render_widget(workspace_block, area);

        // Then render windows in the inner area
        for (win_idx, window) in workspace.windows().iter().enumerate() {
            if win_idx < window_chunks.len() {
                self.render_single_window_with_layout(
                    frame,
                    window_chunks[win_idx],
                    window,
                    &percentage_map,
                );
            }
        }
    }

    /// Render a single window using proper ratatui layout
    fn render_single_window_with_layout(
        &self,
        frame: &mut Frame,
        area: Rect,
        window: &crate::domain::Window,
        percentage_map: &HashMap<crate::domain::values::WindowId, f64>,
    ) {
        let window_style = Self::get_window_style(window.is_focused());
        
        // Debug: log the actual color being used
        tracing::debug!("Window {} style: {:?}", window.process_name(), window_style);

        let percentage = percentage_map.get(window.id()).unwrap_or(&0.0);
        let focus_indicator = if window.is_focused() { "*" } else { "" };

        let window_title = format!(
            "{}{} ({:.0}%)",
            window.process_name(),
            focus_indicator,
            percentage
        );

        // Create content for the window - truncate title and state to fit
        let available_width = area.width.saturating_sub(4) as usize; // minus borders and padding
        let truncated_title =
            TextWidthCalculator::truncate_to_width(window.title(), available_width);

        let state_text = format!(
            "{} {}x{}",
            window.state_indicator(),
            window.geometry().size.width,
            window.geometry().size.height
        );

        let window_content = vec![
            Line::from(truncated_title),
            Line::from(Span::styled(state_text, Style::default().fg(Color::Gray))), // Basic gray
        ];

        let window_title_spans = Line::from(Span::styled(window_title, window_style));
        let window_paragraph = Paragraph::new(window_content).style(window_style).block(
            Block::default()
                .borders(Borders::ALL)
                .title(window_title_spans)
                .border_style(window_style),
        );

        frame.render_widget(window_paragraph, area);
    }

    /// Render multiple monitors side by side
    fn render_monitors_side_by_side(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitors: &[Monitor],
    ) {
        // Create horizontal layout for monitors
        let monitor_count = monitors.len();
        let constraints: Vec<Constraint> = (0..monitor_count)
            .map(|_| Constraint::Percentage(100 / monitor_count as u16))
            .collect();

        let monitor_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (monitor_idx, monitor) in monitors.iter().enumerate() {
            let monitor_area = monitor_chunks[monitor_idx];
            self.render_single_monitor_detailed(frame, monitor_area, monitor);
        }
    }

    /// Render a single monitor in detailed mode
    fn render_single_monitor_detailed(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitor: &Monitor,
    ) {
        let mut items = Vec::new();

        let monitor_style = if monitor.is_focused() {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let monitor_status = if monitor.is_focused() { "[Active]" } else { "" };
        let monitor_title = format!(
            "Monitor {} ({}x{}) {}",
            monitor.id(),
            monitor.geometry().size.width,
            monitor.geometry().size.height,
            monitor_status
        );

        // Workspaces for this monitor
        for workspace in monitor.workspaces() {
            let workspace_style = if workspace.is_focused() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let workspace_status = if workspace.is_focused() {
                "[Active]"
            } else {
                ""
            };
            // Workspace as box per spec (for side-by-side view)
            let workspace_text = format!("Workspace {} {}", workspace.name(), workspace_status);
            let workspace_width = TextWidthCalculator::display_width(&workspace_text);
            let monitor_inner_width = (area.width as usize).saturating_sub(4); // Area width minus borders
            let header_padding = monitor_inner_width
                .saturating_sub(workspace_width)
                .saturating_sub(4); // Account for "┌─ ─┐"

            let workspace_top = format!("┌─ {} {}─┐", workspace_text, "─".repeat(header_padding));

            items.push(ListItem::new(Line::from(Span::styled(
                workspace_top,
                workspace_style,
            ))));

            // Windows in this workspace - box format for side-by-side
            if workspace.windows().is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "│ (Empty)",
                    Style::default().fg(Color::Gray),
                ))));
            } else {
                let percentages = workspace.calculate_window_percentages();
                let percentage_map: HashMap<_, _> = percentages.into_iter().collect();

                for window in workspace.windows() {
                    let percentage = percentage_map.get(window.id()).unwrap_or(&0.0);
                    let window_style = if window.is_focused() {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    };

                    let focus_indicator = if window.is_focused() { "*" } else { "" };

                    // Window box header
                    let header_text = format!(
                        "{}{} ({:.0}%)",
                        window.process_name(),
                        focus_indicator,
                        percentage
                    );
                    let window_info = format!(
                        "│ ┌─ {} ─┐",
                        TextWidthCalculator::truncate_to_width(&header_text, 15)
                    );

                    items.push(ListItem::new(Line::from(Span::styled(
                        window_info,
                        window_style,
                    ))));

                    // Window content
                    let truncated_title =
                        TextWidthCalculator::truncate_to_width(window.title(), 13);
                    let aligned_title =
                        TextWidthCalculator::align_in_box(&truncated_title, 13, Alignment::Left);
                    let window_content = format!("│ │ {} │", aligned_title);

                    items.push(ListItem::new(Line::from(Span::styled(
                        window_content,
                        window_style,
                    ))));

                    // Window details
                    let state_text = format!(
                        "{} {}x{}",
                        window.state_indicator(),
                        window.geometry().size.width,
                        window.geometry().size.height
                    );
                    let aligned_state =
                        TextWidthCalculator::align_in_box(&state_text, 13, Alignment::Left);
                    let window_details = format!("│ │ {} │", aligned_state);

                    items.push(ListItem::new(Line::from(Span::styled(
                        window_details,
                        Style::default().fg(Color::Gray),
                    ))));

                    // Window bottom border
                    items.push(ListItem::new(Line::from(Span::styled(
                        "│ └─────────────┘",
                        window_style,
                    ))));
                }
            }

            // Workspace bottom border
            let workspace_bottom = format!("└{}┘", "─".repeat(monitor_inner_width));
            items.push(ListItem::new(Line::from(Span::styled(
                workspace_bottom,
                workspace_style,
            ))));

            items.push(ListItem::new(Line::from("")));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(monitor_title)
                    .style(monitor_style),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }

    /// Render monitors in compact tree-style mode
    fn render_monitors_compact(
        &self,
        frame: &mut Frame,
        area: Rect,
        monitors: &[Monitor],
    ) {
        let mut items = Vec::new();

        for (monitor_idx, monitor) in monitors.iter().enumerate() {
            let is_last_monitor = monitor_idx == monitors.len() - 1;

            // Monitor header with tree prefix
            let monitor_style = if monitor.is_focused() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let monitor_info = format!(
                "Monitor {} ({}x{}) [{}] ({} windows)",
                monitor.id(),
                monitor.geometry().size.width,
                monitor.geometry().size.height,
                if monitor.is_focused() {
                    "Active"
                } else {
                    "Inactive"
                },
                monitor.total_window_count()
            );

            items.push(ListItem::new(Line::from(Span::styled(
                monitor_info,
                monitor_style,
            ))));

            // Workspaces for this monitor
            let workspaces = monitor.workspaces();
            for (ws_idx, workspace) in workspaces.iter().enumerate() {
                let is_last_workspace = ws_idx == workspaces.len() - 1;

                let workspace_style = if workspace.is_focused() {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let ws_prefix = if is_last_workspace && is_last_monitor {
                    "└─ "
                } else if is_last_workspace {
                    "└─ "
                } else {
                    "├─ "
                };

                let workspace_info = format!(
                    "{}WS {} [{}] ({} windows)",
                    ws_prefix,
                    workspace.name(),
                    if workspace.is_focused() {
                        "Active"
                    } else {
                        "Inactive"
                    },
                    workspace.window_count()
                );

                items.push(ListItem::new(Line::from(Span::styled(
                    workspace_info,
                    workspace_style,
                ))));

                // Windows in this workspace
                let windows = workspace.windows();
                for (win_idx, window) in windows.iter().enumerate() {
                    let is_last_window = win_idx == windows.len() - 1;

                    let window_style = if window.is_focused() {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    };

                    let win_prefix = if is_last_workspace && is_last_monitor {
                        if is_last_window {
                            "    └─ "
                        } else {
                            "    ├─ "
                        }
                    } else if is_last_workspace {
                        if is_last_window {
                            "    └─ "
                        } else {
                            "    ├─ "
                        }
                    } else if is_last_window {
                        "│   └─ "
                    } else {
                        "│   ├─ "
                    };

                    let window_info = format!(
                        "{}{} {} {}",
                        win_prefix,
                        window.state_indicator(),
                        window.display_name_truncated(40),
                        if window.is_focused() { "(Focused)" } else { "" }
                    );

                    items.push(ListItem::new(Line::from(Span::styled(
                        window_info,
                        window_style,
                    ))));
                }
            }

            // Add spacing between monitors
            if !is_last_monitor {
                items.push(ListItem::new(Line::from("")));
            }
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Monitors & Workspaces (Compact)")
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

    #[test]
    fn should_generate_workspace_box_layout() {
        let monitor = create_test_monitor();
        let renderer = Renderer::new();

        // Test workspace box structure
        // Should generate:
        // │ ┌─ Workspace Name [Active] ──────────┐ │
        // │ │ [window boxes]                     │ │
        // │ └────────────────────────────────────┘ │

        // This is a structural test - actual rendering would need mock terminal
        assert!(monitor.workspaces().len() > 0);
        assert!(monitor.workspaces()[0].windows().len() > 0);
    }

    #[test]
    fn should_calculate_equal_width_boxes() {
        let available_width = 54_usize; // Workspace inner width
        let window_count = 3;
        let spaces_between = window_count - 1;
        let total_box_width = available_width.saturating_sub(spaces_between);
        let box_width = total_box_width / window_count;

        // Each box should be equal width
        assert_eq!(box_width, 17); // (54 - 2) / 3 = 17

        // Total should not exceed available width
        let total_used = (box_width * window_count) + spaces_between;
        assert!(total_used <= available_width);
    }

    #[test]
    fn should_handle_unicode_in_workspace_headers() {
        use crate::utils::text_width::TextWidthCalculator;

        // Japanese workspace name
        let workspace_text = "Workspace 開発環境 [Active]";
        let width = TextWidthCalculator::display_width(&workspace_text);

        // Should calculate correct width for CJK characters
        assert!(width > workspace_text.len()); // CJK chars are wider than ASCII

        // Truncation should not break characters
        let truncated = TextWidthCalculator::truncate_to_width(&workspace_text, 20);
        assert!(truncated.len() <= workspace_text.len());
    }

    #[test]
    fn should_apply_correct_colors_for_focus_states() {
        // Monitor colors
        let focused_monitor_style = Renderer::get_monitor_style(true);
        assert_eq!(focused_monitor_style.fg, Some(Color::Red)); // Red
        assert!(focused_monitor_style.add_modifier.contains(Modifier::BOLD));

        let unfocused_monitor_style = Renderer::get_monitor_style(false);
        assert_eq!(unfocused_monitor_style.fg, Some(Color::Blue)); // Blue
        assert!(!unfocused_monitor_style.add_modifier.contains(Modifier::BOLD));

        // Workspace colors
        let focused_workspace_style = Renderer::get_workspace_style(true);
        assert_eq!(focused_workspace_style.fg, Some(Color::Green)); // Green
        assert!(focused_workspace_style.add_modifier.contains(Modifier::BOLD));

        let unfocused_workspace_style = Renderer::get_workspace_style(false);
        assert_eq!(unfocused_workspace_style.fg, Some(Color::Gray)); // Gray
        assert!(!unfocused_workspace_style.add_modifier.contains(Modifier::BOLD));

        // Window colors
        let focused_window_style = Renderer::get_window_style(true);
        assert_eq!(focused_window_style.fg, Some(Color::Magenta)); // Magenta
        assert!(focused_window_style.add_modifier.contains(Modifier::BOLD));

        let unfocused_window_style = Renderer::get_window_style(false);
        assert_eq!(unfocused_window_style.fg, Some(Color::Cyan)); // Cyan
        assert!(!unfocused_window_style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn should_maintain_color_consistency() {
        // Focused elements should use consistent colors
        let focused_workspace = Renderer::get_workspace_style(true);
        let focused_window = Renderer::get_window_style(true);
        
        // Focused workspace should use Green, focused window should use Magenta (different colors)
        assert_ne!(focused_workspace.fg, focused_window.fg);
        assert_eq!(focused_workspace.fg, Some(Color::Green));
        assert_eq!(focused_window.fg, Some(Color::Magenta));

        // Both should be bold
        assert!(focused_workspace.add_modifier.contains(Modifier::BOLD));
        assert!(focused_window.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn should_use_distinct_colors_for_different_states() {
        let monitor_focused = Renderer::get_monitor_style(true);
        let workspace_focused = Renderer::get_workspace_style(true);
        let window_focused = Renderer::get_window_style(true);
        
        let monitor_unfocused = Renderer::get_monitor_style(false);
        let workspace_unfocused = Renderer::get_workspace_style(false);
        let window_unfocused = Renderer::get_window_style(false);

        // All focused/unfocused colors should be different from each other
        assert_ne!(monitor_focused.fg, monitor_unfocused.fg);
        assert_ne!(workspace_focused.fg, workspace_unfocused.fg);
        assert_ne!(window_focused.fg, window_unfocused.fg);

        // Monitor should have unique colors
        assert_ne!(monitor_focused.fg, workspace_focused.fg);
        assert_ne!(monitor_unfocused.fg, workspace_unfocused.fg);
        assert_ne!(monitor_unfocused.fg, window_unfocused.fg);
    }

    // Note: Full rendering tests would require a mock terminal,
    // which is complex to set up. The rendering logic is tested
    // indirectly through integration tests.
}
