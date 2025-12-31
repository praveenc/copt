//! Icon definitions with Nerd Font and ASCII fallback support
//!
//! Detects terminal capabilities and provides appropriate icons.

use std::sync::OnceLock;

/// Icon set with all available icons
#[derive(Debug, Clone)]
pub struct IconSet {
    pub check: &'static str,
    pub cross: &'static str,
    pub warning: &'static str,
    pub info: &'static str,
    pub lightning: &'static str,
    pub folder_open: &'static str,
    pub folder_closed: &'static str,
    pub file: &'static str,
    pub chart: &'static str,
    pub gear: &'static str,
    pub sparkles: &'static str,
    pub inbox: &'static str,
    pub clock: &'static str,
    pub arrow_right: &'static str,
    pub bullet: &'static str,
}

impl IconSet {
    /// Nerd Font icons (requires Nerd Font installed)
    pub fn nerd_fonts() -> Self {
        Self {
            check: "\u{f00c}",           //
            cross: "\u{f00d}",           //
            warning: "\u{f071}",         //
            info: "\u{f129}",            //
            lightning: "\u{f0e7}",       //
            folder_open: "\u{f07c}",     //
            folder_closed: "\u{f07b}",   //
            file: "\u{f15b}",            //
            chart: "\u{f080}",           //
            gear: "\u{f013}",            //
            sparkles: "\u{2728}",        // ✨ (unicode sparkles, works everywhere)
            inbox: "\u{f01c}",           //
            clock: "\u{f017}",           //
            arrow_right: "\u{f061}",     //
            bullet: "\u{f111}",          //
        }
    }

    /// Unicode icons (works on most modern terminals)
    pub fn unicode() -> Self {
        Self {
            check: "\u{2713}",           // ✓
            cross: "\u{2717}",           // ✗
            warning: "\u{26a0}",         // ⚠
            info: "\u{2139}",            // ℹ
            lightning: "\u{26a1}",       // ⚡
            folder_open: "\u{25bc}",     // ▼
            folder_closed: "\u{25b6}",   // ▶
            file: "\u{2022}",            // •
            chart: "\u{2593}",           // ▓
            gear: "\u{2699}",            // ⚙
            sparkles: "\u{2728}",        // ✨
            inbox: "\u{1f4e5}",          // 📥
            clock: "\u{23f1}",           // ⏱
            arrow_right: "\u{2192}",     // →
            bullet: "\u{25cf}",          // ●
        }
    }

    /// ASCII fallback (works everywhere)
    pub fn ascii() -> Self {
        Self {
            check: "[ok]",
            cross: "[x]",
            warning: "[!]",
            info: "[i]",
            lightning: "[*]",
            folder_open: "[-]",
            folder_closed: "[+]",
            file: "[ ]",
            chart: "[#]",
            gear: "[@]",
            sparkles: "[~]",
            inbox: "[>]",
            clock: "[t]",
            arrow_right: "->",
            bullet: "*",
        }
    }
}

/// Detect whether the terminal likely supports Nerd Fonts
fn supports_nerd_fonts() -> bool {
    // Check for common Nerd Font terminal indicators
    // This is a heuristic and may not be 100% accurate

    // Check TERM_PROGRAM for known terminals that often have Nerd Fonts
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let nerd_font_terminals = [
            "iTerm.app",
            "WezTerm",
            "Alacritty",
            "kitty",
            "Hyper",
            "Tabby",
        ];
        if nerd_font_terminals
            .iter()
            .any(|t| term_program.contains(t))
        {
            return true;
        }
    }

    // Check for Nerd Font specific env var (some setups use this)
    if std::env::var("NERD_FONT").is_ok() {
        return true;
    }

    // Default to false to be safe
    false
}

/// Detect whether the terminal supports Unicode
fn supports_unicode() -> bool {
    // Check LANG/LC_ALL for UTF-8
    for var in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if val.to_lowercase().contains("utf") {
                return true;
            }
        }
    }

    // Check TERM for modern terminal types
    if let Ok(term) = std::env::var("TERM") {
        let unicode_terms = ["xterm", "screen", "tmux", "vt100", "linux", "rxvt"];
        if unicode_terms.iter().any(|t| term.contains(t)) {
            return true;
        }
    }

    // macOS terminals generally support Unicode
    #[cfg(target_os = "macos")]
    return true;

    #[cfg(not(target_os = "macos"))]
    false
}

/// Detect and return the appropriate icon set for the current terminal
pub fn detect_icons() -> IconSet {
    if supports_nerd_fonts() {
        IconSet::nerd_fonts()
    } else if supports_unicode() {
        IconSet::unicode()
    } else {
        IconSet::ascii()
    }
}

/// Global icon set instance
pub fn icons() -> &'static IconSet {
    static ICONS: OnceLock<IconSet> = OnceLock::new();
    ICONS.get_or_init(detect_icons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nerd_fonts_icons() {
        let icons = IconSet::nerd_fonts();
        assert!(!icons.check.is_empty());
        assert!(!icons.cross.is_empty());
    }

    #[test]
    fn test_unicode_icons() {
        let icons = IconSet::unicode();
        assert!(!icons.check.is_empty());
        assert_eq!(icons.check, "✓");
    }

    #[test]
    fn test_ascii_icons() {
        let icons = IconSet::ascii();
        assert_eq!(icons.check, "[ok]");
        assert_eq!(icons.cross, "[x]");
    }

    #[test]
    fn test_global_icons() {
        let i1 = icons();
        let i2 = icons();
        assert!(std::ptr::eq(i1, i2));
    }
}
