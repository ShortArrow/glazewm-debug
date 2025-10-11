// Domain value objects
// Immutable data structures representing concepts in the window management domain

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a position in 2D space (x, y coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self::new(0, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Represents dimensions (width and height)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Represents a rectangular area (position + size)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size,
}

impl Rectangle {
    pub fn new(position: Position, size: Size) -> Self {
        Self { position, size }
    }

    pub fn from_coords(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self::new(Position::new(x, y), Size::new(width, height))
    }

    pub fn area(&self) -> u64 {
        self.size.area()
    }

    pub fn contains_point(&self, point: Position) -> bool {
        point.x >= self.position.x
            && point.x < (self.position.x + self.size.width as i32)
            && point.y >= self.position.y
            && point.y < (self.position.y + self.size.height as i32)
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.position, self.size)
    }
}

/// Unique identifier for monitors
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorId(String);

impl MonitorId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for workspaces
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for windows
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(String);

impl WindowId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_should_display_correctly() {
        let pos = Position::new(100, 200);
        assert_eq!(format!("{}", pos), "(100, 200)");
    }

    #[test]
    fn size_should_calculate_area() {
        let size = Size::new(800, 600);
        assert_eq!(size.area(), 480_000);
    }

    #[test]
    fn rectangle_should_contain_points() {
        let rect = Rectangle::from_coords(10, 10, 100, 100);
        
        assert!(rect.contains_point(Position::new(50, 50))); // Inside
        assert!(!rect.contains_point(Position::new(5, 5)));   // Outside
        assert!(!rect.contains_point(Position::new(150, 150))); // Outside
    }

    #[test]
    fn identifiers_should_display_correctly() {
        let monitor_id = MonitorId::new("mon-1".to_string());
        let workspace_id = WorkspaceId::new("ws-1".to_string());
        let window_id = WindowId::new("win-1".to_string());

        assert_eq!(format!("{}", monitor_id), "mon-1");
        assert_eq!(format!("{}", workspace_id), "ws-1");
        assert_eq!(format!("{}", window_id), "win-1");
    }
}