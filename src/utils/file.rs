//! File I/O utilities for the prompt optimizer
//!
//! Provides functions for reading and writing prompt files.

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::path::Path;

/// Read a prompt from a file
///
/// # Arguments
/// * `path` - Path to the prompt file
///
/// # Returns
/// The contents of the file as a string
///
/// # Errors
/// Returns an error if the file cannot be read
pub fn read_prompt_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read prompt file: {}", path.display()))
}

/// Write an optimized prompt to a file
///
/// # Arguments
/// * `path` - Path where the file should be written
/// * `content` - The prompt content to write
///
/// # Errors
/// Returns an error if the file cannot be written
pub fn write_prompt_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write prompt file: {}", path.display()))
}

/// Check if a file exists and is readable
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_file()
}

/// Get the file extension, if any
pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Determine if a file is likely a prompt file based on extension
pub fn is_prompt_file<P: AsRef<Path>>(path: P) -> bool {
    let valid_extensions = ["txt", "md", "prompt", "text", ""];

    match get_extension(&path) {
        Some(ext) => valid_extensions.contains(&ext.as_str()),
        None => {
            // No extension - check if it's a file
            path.as_ref().is_file()
        }
    }
}

/// Get file size in bytes
pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = std::fs::metadata(path.as_ref())
        .with_context(|| format!("Failed to get metadata for: {}", path.as_ref().display()))?;
    Ok(metadata.len())
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

/// Read multiple prompt files from a directory
pub fn read_prompts_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<(String, String)>> {
    let dir = dir.as_ref();
    let mut prompts = Vec::new();

    if !dir.is_dir() {
        anyhow::bail!("Path is not a directory: {}", dir.display());
    }

    for entry in
        std::fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_prompt_file(&path) {
            let content = read_prompt_file(&path)?;
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            prompts.push((name, content));
        }
    }

    Ok(prompts)
}

/// Async version of read_prompt_file using tokio
pub async fn read_prompt_file_async<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref().to_path_buf();
    tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read prompt file: {}", path.display()))
}

/// Async version of write_prompt_file using tokio
pub async fn write_prompt_file_async<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref().to_path_buf();

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    tokio::fs::write(&path, content)
        .await
        .with_context(|| format!("Failed to write prompt file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_read_prompt_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test prompt content").unwrap();

        let content = read_prompt_file(file.path()).unwrap();
        assert!(content.contains("Test prompt content"));
    }

    #[test]
    fn test_write_prompt_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_prompt.txt");

        write_prompt_file(&file_path, "Optimized prompt").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Optimized prompt");
    }

    #[test]
    fn test_file_exists() {
        let file = NamedTempFile::new().unwrap();
        assert!(file_exists(file.path()));
        assert!(!file_exists("/nonexistent/path/file.txt"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_extension("file.MD"), Some("md".to_string()));
        assert_eq!(get_extension("file"), None);
    }

    #[test]
    fn test_is_prompt_file() {
        assert!(is_prompt_file("prompt.txt"));
        assert!(is_prompt_file("README.md"));
        assert!(is_prompt_file("template.prompt"));
        assert!(!is_prompt_file("image.png"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2.0 KB");
        assert_eq!(format_file_size(1572864), "1.5 MB");
    }
}