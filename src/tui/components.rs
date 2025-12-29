//! Reusable TUI components
//!
//! This module provides common UI components used throughout the optimizer.

use colored::Colorize;

/// A progress bar component
pub struct ProgressBar {
    pub current: usize,
    pub total: usize,
    pub width: usize,
    pub filled_char: char,
    pub empty_char: char,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            width: 30,
            filled_char: '█',
            empty_char: '░',
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    pub fn render(&self) -> String {
        let percent = if self.total > 0 {
            self.current as f64 / self.total as f64
        } else {
            0.0
        };

        let filled = (percent * self.width as f64) as usize;
        let empty = self.width - filled;

        format!(
            "[{}{}] {:>3}%",
            self.filled_char.to_string().repeat(filled).green(),
            self.empty_char.to_string().repeat(empty).bright_black(),
            (percent * 100.0) as usize
        )
    }
}

/// A badge component for displaying labels
pub struct Badge {
    pub text: String,
    pub style: BadgeStyle,
}

#[derive(Clone, Copy)]
pub enum BadgeStyle {
    Info,
    Success,
    Warning,
    Error,
    Muted,
}

impl Badge {
    pub fn new(text: &str, style: BadgeStyle) -> Self {
        Self {
            text: text.to_string(),
            style,
        }
    }

    pub fn info(text: &str) -> Self {
        Self::new(text, BadgeStyle::Info)
    }

    pub fn success(text: &str) -> Self {
        Self::new(text, BadgeStyle::Success)
    }

    pub fn warning(text: &str) -> Self {
        Self::new(text, BadgeStyle::Warning)
    }

    pub fn error(text: &str) -> Self {
        Self::new(text, BadgeStyle::Error)
    }

    pub fn render(&self) -> String {
        match self.style {
            BadgeStyle::Info => format!("[{}]", self.text).blue().to_string(),
            BadgeStyle::Success => format!("[{}]", self.text).green().to_string(),
            BadgeStyle::Warning => format!("[{}]", self.text).yellow().to_string(),
            BadgeStyle::Error => format!("[{}]", self.text).red().to_string(),
            BadgeStyle::Muted => format!("[{}]", self.text).bright_black().to_string(),
        }
    }
}

/// A table component for displaying structured data
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub column_widths: Vec<usize>,
}

impl Table {
    pub fn new(headers: Vec<&str>) -> Self {
        let headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let column_widths = headers.iter().map(|h| h.len()).collect();

        Self {
            headers,
            rows: Vec::new(),
            column_widths,
        }
    }

    pub fn add_row(&mut self, row: Vec<&str>) {
        let row: Vec<String> = row.iter().map(|s| s.to_string()).collect();

        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }

        self.rows.push(row);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Header row
        let header_line: Vec<String> = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                format!(
                    "{:width$}",
                    h,
                    width = self.column_widths.get(i).copied().unwrap_or(h.len())
                )
            })
            .collect();
        output.push_str(&header_line.join(" │ ").cyan().bold().to_string());
        output.push('\n');

        // Separator
        let separator: Vec<String> = self.column_widths.iter().map(|w| "─".repeat(*w)).collect();
        output.push_str(&separator.join("─┼─").bright_black().to_string());
        output.push('\n');

        // Data rows
        for row in &self.rows {
            let row_line: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let width = self.column_widths.get(i).copied().unwrap_or(cell.len());
                    format!("{:width$}", cell, width = width)
                })
                .collect();
            output.push_str(&row_line.join(" │ "));
            output.push('\n');
        }

        output
    }
}

/// A key-value display component
pub struct KeyValue {
    pub pairs: Vec<(String, String)>,
    pub key_width: usize,
}

impl KeyValue {
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
            key_width: 0,
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.key_width = self.key_width.max(key.len());
        self.pairs.push((key.to_string(), value.to_string()));
    }

    pub fn render(&self) -> String {
        self.pairs
            .iter()
            .map(|(k, v)| {
                format!(
                    "{:>width$}: {}",
                    k.bright_white(),
                    v.cyan(),
                    width = self.key_width
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for KeyValue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new(100);
        bar.set(50);
        let rendered = bar.render();
        assert!(rendered.contains("50%"));
    }

    #[test]
    fn test_badge() {
        let badge = Badge::info("INFO");
        let rendered = badge.render();
        assert!(rendered.contains("INFO"));
    }

    #[test]
    fn test_table() {
        let mut table = Table::new(vec!["Name", "Value"]);
        table.add_row(vec!["test", "123"]);
        let rendered = table.render();
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("test"));
    }

    #[test]
    fn test_key_value() {
        let mut kv = KeyValue::new();
        kv.add("Key", "Value");
        let rendered = kv.render();
        assert!(rendered.contains("Key"));
        assert!(rendered.contains("Value"));
    }
}