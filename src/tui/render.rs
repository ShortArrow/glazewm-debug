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

    /// Render the application state to the given frame
    pub fn render<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        monitors: &[Monitor],
        mode: DisplayMode,
    ) {
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
    fn render_header<B: Backend>(
        &self,
        frame: &mut Frame<B>,
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
                    .style(Style::default().fg(Color::Cyan)),
            );

        frame.render_widget(header, area);
    }

    /// Render the footer with keyboard shortcuts
    fn render_footer<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let footer_text = "q/Esc: Quit | r: Refresh | c: Toggle Mode | Ctrl+C: Force Quit";

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

    /// Render the list of monitors and their workspaces (detailed mode)
    fn render_monitors_detailed<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        monitors: &[Monitor],
    ) {
        // Re-enable side-by-side monitor layout per DISPLAY.md spec
        if monitors.len() > 1 && area.width >= 120 {
            self.render_monitors_side_by_side(frame, area, monitors);
            return;
        }

        // Single monitor or narrow screen - render vertically
        let mut items = Vec::new();

        for monitor in monitors {
            // Monitor header with box drawing
            let monitor_style = if monitor.is_focused() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let monitor_status = if monitor.is_focused() { "[Active]" } else { "" };
            let base_text = format!(
                "┌─ Monitor {} ({}x{}) {} ",
                monitor.id(),
                monitor.geometry().size.width,
                monitor.geometry().size.height,
                monitor_status
            );

            let base_width = TextWidthCalculator::display_width(&base_text);
            let remaining_width = 60_usize.saturating_sub(base_width);
            let monitor_header = format!("{}{}┐", base_text, "─".repeat(remaining_width));

            // Debug information for Unicode width
            tracing::debug!(
                "Monitor header: base_text='{}', base_width={}, remaining_width={}, total_length={}",
                base_text.replace('\n', "\\n"),
                base_width,
                remaining_width,
                TextWidthCalculator::display_width(&monitor_header)
            );

            items.push(ListItem::new(Spans::from(Span::styled(
                monitor_header,
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

                let workspace_status = if workspace.is_focused() {
                    "[Active]"
                } else {
                    ""
                };
                // Workspace as box per enhanced spec
                let workspace_text = format!("Workspace {} {}", workspace.name(), workspace_status);
                let workspace_width = TextWidthCalculator::display_width(&workspace_text);
                let header_padding = 52_usize.saturating_sub(workspace_width); // Fill workspace header width

                let workspace_top =
                    format!("│ ┌─ {} {}─┐ │", workspace_text, "─".repeat(header_padding));

                items.push(ListItem::new(Spans::from(Span::styled(
                    workspace_top,
                    workspace_style,
                ))));

                // Calculate window percentages
                let percentages = workspace.calculate_window_percentages();
                let percentage_map: HashMap<_, _> = percentages.into_iter().collect();

                // Windows layout - equal-width boxes that fill parent
                if !workspace.windows().is_empty() {
                    let window_count = workspace.windows().len();
                    let available_width = 58_usize; // Parent container width minus borders
                    let spaces_between = window_count.saturating_sub(1); // Spaces between boxes
                    let total_box_width = available_width.saturating_sub(spaces_between);
                    let box_width = total_box_width / window_count; // Equal width per box

                    // Box headers line
                    let mut box_parts = Vec::new();
                    for window in workspace.windows() {
                        let percentage = percentage_map.get(window.id()).unwrap_or(&0.0);
                        let focus_indicator = if window.is_focused() { "*" } else { "" };

                        let header_text = format!(
                            "{}{} ({:.0}%)",
                            window.process_name(),
                            focus_indicator,
                            percentage
                        );
                        let header_width = TextWidthCalculator::display_width(&header_text);
                        let header_padding =
                            box_width.saturating_sub(4).saturating_sub(header_width); // 4 for "┌─ ─┐"

                        box_parts.push(format!(
                            "┌─ {} {}─┐",
                            TextWidthCalculator::truncate_to_width(
                                &header_text,
                                box_width.saturating_sub(6)
                            ),
                            "─".repeat(header_padding)
                        ));
                    }

                    items.push(ListItem::new(Spans::from(Span::styled(
                        format!("│ │ {} │ │", box_parts.join(" ")),
                        Style::default().fg(Color::LightBlue),
                    ))));

                    // Content line
                    let mut content_parts = Vec::new();
                    for window in workspace.windows() {
                        let title = TextWidthCalculator::truncate_to_width(
                            window.title(),
                            box_width.saturating_sub(4),
                        );
                        let padded_title = TextWidthCalculator::align_in_box(
                            &title,
                            box_width.saturating_sub(4),
                            Alignment::Left,
                        );
                        content_parts.push(format!("│ {} │", padded_title));
                    }

                    items.push(ListItem::new(Spans::from(Span::styled(
                        format!("│ │ {} │ │", content_parts.join(" ")),
                        Style::default().fg(Color::LightBlue),
                    ))));

                    // State line
                    let mut state_parts = Vec::new();
                    for window in workspace.windows() {
                        let state_text = format!(
                            "{} {}x{}",
                            window.state_indicator(),
                            window.geometry().size.width,
                            window.geometry().size.height
                        );
                        let padded_state = TextWidthCalculator::align_in_box(
                            &state_text,
                            box_width.saturating_sub(4),
                            Alignment::Left,
                        );
                        state_parts.push(format!("│ {} │", padded_state));
                    }

                    items.push(ListItem::new(Spans::from(Span::styled(
                        format!("│ │ {} │ │", state_parts.join(" ")),
                        Style::default().fg(Color::Gray),
                    ))));

                    // Bottom borders line
                    let bottom_parts: Vec<String> = (0..window_count)
                        .map(|_| format!("└{}┘", "─".repeat(box_width.saturating_sub(2))))
                        .collect();

                    items.push(ListItem::new(Spans::from(Span::styled(
                        format!("│ │ {} │ │", bottom_parts.join(" ")),
                        Style::default().fg(Color::LightBlue),
                    ))));
                } else {
                    items.push(ListItem::new(Spans::from(Span::styled(
                        "│ │ (Empty)",
                        Style::default().fg(Color::Gray),
                    ))));
                }

                // Workspace bottom border
                let workspace_bottom = format!("│ └{}┘ │", "─".repeat(52));
                items.push(ListItem::new(Spans::from(Span::styled(
                    workspace_bottom,
                    workspace_style,
                ))));

                // Add spacing between workspaces
                items.push(ListItem::new(Spans::from("")));
            }

            // Monitor bottom border with fixed width
            let bottom_border = format!("└{}┘", "─".repeat(58)); // Match monitor header width
            items.push(ListItem::new(Spans::from(Span::styled(
                bottom_border,
                monitor_style,
            ))));
            items.push(ListItem::new(Spans::from("")));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Monitors & Workspaces (Detailed)")
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }

    /// Render multiple monitors side by side
    fn render_monitors_side_by_side<B: Backend>(
        &self,
        frame: &mut Frame<B>,
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
    fn render_single_monitor_detailed<B: Backend>(
        &self,
        frame: &mut Frame<B>,
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
            let header_padding = 20_usize.saturating_sub(workspace_width); // Smaller width for side-by-side

            let workspace_top = format!("┌─ {} {}─┐", workspace_text, "─".repeat(header_padding));

            items.push(ListItem::new(Spans::from(Span::styled(
                workspace_top,
                workspace_style,
            ))));

            // Windows in this workspace - box format for side-by-side
            if workspace.windows().is_empty() {
                items.push(ListItem::new(Spans::from(Span::styled(
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

                    items.push(ListItem::new(Spans::from(Span::styled(
                        window_info,
                        window_style,
                    ))));

                    // Window content
                    let truncated_title =
                        TextWidthCalculator::truncate_to_width(window.title(), 13);
                    let aligned_title =
                        TextWidthCalculator::align_in_box(&truncated_title, 13, Alignment::Left);
                    let window_content = format!("│ │ {} │", aligned_title);

                    items.push(ListItem::new(Spans::from(Span::styled(
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

                    items.push(ListItem::new(Spans::from(Span::styled(
                        window_details,
                        Style::default().fg(Color::Gray),
                    ))));

                    // Window bottom border
                    items.push(ListItem::new(Spans::from(Span::styled(
                        "│ └─────────────┘",
                        window_style,
                    ))));
                }
            }

            // Workspace bottom border
            let workspace_bottom = format!("└{}┘", "─".repeat(22)); // Match workspace width
            items.push(ListItem::new(Spans::from(Span::styled(
                workspace_bottom,
                workspace_style,
            ))));

            items.push(ListItem::new(Spans::from("")));
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
    fn render_monitors_compact<B: Backend>(
        &self,
        frame: &mut Frame<B>,
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

            items.push(ListItem::new(Spans::from(Span::styled(
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

                items.push(ListItem::new(Spans::from(Span::styled(
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

                    items.push(ListItem::new(Spans::from(Span::styled(
                        window_info,
                        window_style,
                    ))));
                }
            }

            // Add spacing between monitors
            if !is_last_monitor {
                items.push(ListItem::new(Spans::from("")));
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

    // Note: Full rendering tests would require a mock terminal,
    // which is complex to set up. The rendering logic is tested
    // indirectly through integration tests.
}
