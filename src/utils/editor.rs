//! Editor command utilities
//!
//! Shared logic for launching external editors with appropriate flags.

/// Build editor command with appropriate flags for GUI editors.
///
/// When `wait` is true, adds --wait flags for editors that support it
/// (used when we need the editor to block until the user closes the file).
/// When `wait` is false, opens the file without blocking (for viewing).
pub fn build_editor_command(
    editor: &str,
    file_path: &std::path::Path,
    wait: bool,
) -> (String, Vec<String>) {
    let editor_lower = editor.to_lowercase();
    let file_arg = file_path.to_string_lossy().to_string();

    // Extract just the binary name for matching (handle full paths)
    let editor_name = std::path::Path::new(editor)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(editor)
        .to_lowercase();

    // VSCode: `code --wait` for editing, `code` for viewing
    if editor_name.contains("code") || editor_lower.contains("visual studio code") {
        let mut args = Vec::new();
        if wait {
            args.push("--wait".to_string());
        }
        args.push(file_arg);
        return (editor.to_string(), args);
    }

    // Zed: `zed --wait` for editing, `zed` for viewing
    if editor_name == "cli" && editor_lower.contains("zed") {
        let mut args = Vec::new();
        if wait {
            args.push("--wait".to_string());
        }
        args.push(file_arg);
        return (editor.to_string(), args);
    }
    if editor_name.contains("zed") {
        let mut args = Vec::new();
        if wait {
            args.push("--wait".to_string());
        }
        args.push(file_arg);
        return (editor.to_string(), args);
    }

    // Default: terminal editors (vim, nano, emacs, etc.) block by default
    (editor.to_string(), vec![file_arg])
}
