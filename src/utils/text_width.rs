// Unicode-aware text width calculation
// Based on Vim's approach for handling CJK and multi-byte characters

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Utility for calculating display width of text with proper Unicode handling
pub struct TextWidthCalculator;

impl TextWidthCalculator {
    /// Calculate the display width of a string in terminal columns
    /// Handles East Asian wide characters, combining characters, etc.
    pub fn display_width(text: &str) -> usize {
        UnicodeWidthStr::width(text)
    }

    /// Calculate the display width of a single character
    pub fn char_width(ch: char) -> usize {
        UnicodeWidthChar::width(ch).unwrap_or(0)
    }

    /// Truncate text to fit within a specific display width
    /// Ensures proper handling of multi-byte characters
    pub fn truncate_to_width(text: &str, max_width: usize) -> String {
        if Self::display_width(text) <= max_width {
            return text.to_string();
        }

        let mut result = String::new();
        let mut current_width = 0;

        for ch in text.chars() {
            let char_width = Self::char_width(ch);

            // Check if adding this character would exceed the limit
            if current_width + char_width > max_width.saturating_sub(3) {
                result.push_str("...");
                break;
            }

            result.push(ch);
            current_width += char_width;
        }

        result
    }

    /// Pad text to a specific width with spaces
    /// Handles multi-byte characters correctly
    pub fn pad_to_width(text: &str, target_width: usize) -> String {
        let current_width = Self::display_width(text);

        if current_width >= target_width {
            text.to_string()
        } else {
            let padding = " ".repeat(target_width - current_width);
            format!("{}{}", text, padding)
        }
    }

    /// Create a horizontal line of specific width using box-drawing characters
    /// Ensures proper alignment even with multi-byte text nearby
    pub fn horizontal_line(width: usize, ch: char) -> String {
        ch.to_string().repeat(width)
    }

    /// Align text within a box, handling multi-byte characters
    pub fn align_in_box(text: &str, box_width: usize, alignment: Alignment) -> String {
        let text_width = Self::display_width(text);

        if text_width >= box_width {
            return Self::truncate_to_width(text, box_width);
        }

        let padding = box_width - text_width;

        match alignment {
            Alignment::Left => {
                format!("{}{}", text, " ".repeat(padding))
            }
            Alignment::Center => {
                let left_padding = padding / 2;
                let right_padding = padding - left_padding;
                format!(
                    "{}{}{}",
                    " ".repeat(left_padding),
                    text,
                    " ".repeat(right_padding)
                )
            }
            Alignment::Right => {
                format!("{}{}", " ".repeat(padding), text)
            }
        }
    }
}

/// Text alignment options for box formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_ascii_width() {
        assert_eq!(TextWidthCalculator::display_width("hello"), 5);
        assert_eq!(TextWidthCalculator::display_width("VS Code"), 7);
    }

    #[test]
    fn should_calculate_cjk_width() {
        // Japanese characters are typically 2 cells wide
        assert_eq!(TextWidthCalculator::display_width("こんにちは"), 10);
        assert_eq!(TextWidthCalculator::display_width("設定"), 4);

        // Mixed ASCII and CJK - recalculate actual width
        let mixed = "VS Code - 設定";
        let actual_width = TextWidthCalculator::display_width(mixed);
        // VS Code = 7, " - " = 3, 設定 = 4, total = 14
        assert_eq!(actual_width, 14);
    }

    #[test]
    fn should_truncate_with_unicode_awareness() {
        let text = "VS Code - 設定画面";
        let truncated = TextWidthCalculator::truncate_to_width(text, 10);

        // Should not break in the middle of a multi-byte character
        assert!(TextWidthCalculator::display_width(&truncated) <= 10);
        assert!(truncated.contains("..."));
    }

    #[test]
    fn should_pad_text_correctly() {
        let padded = TextWidthCalculator::pad_to_width("設定", 10);
        assert_eq!(TextWidthCalculator::display_width(&padded), 10);
        assert!(padded.starts_with("設定"));
    }

    #[test]
    fn should_align_text_in_box() {
        let centered = TextWidthCalculator::align_in_box("設定", 10, Alignment::Center);
        assert_eq!(TextWidthCalculator::display_width(&centered), 10);

        let right = TextWidthCalculator::align_in_box("設定", 10, Alignment::Right);
        assert_eq!(TextWidthCalculator::display_width(&right), 10);
    }

    #[test]
    fn should_handle_char_width() {
        // ASCII characters
        assert_eq!(TextWidthCalculator::char_width('a'), 1);
        assert_eq!(TextWidthCalculator::char_width('1'), 1);

        // CJK characters
        assert_eq!(TextWidthCalculator::char_width('設'), 2);
        assert_eq!(TextWidthCalculator::char_width('あ'), 2);
        assert_eq!(TextWidthCalculator::char_width('中'), 2);
    }
}
