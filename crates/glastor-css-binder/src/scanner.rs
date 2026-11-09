//! Parallel source code utility class scanner.
//!
//! Scans Rust (`.rs`, `.rsx`), HTML, and template files for CSS class tokens using
//! optimized parallel regex evaluation via `rayon`.

use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that may occur during source scanning.
#[derive(Debug, Error)]
pub enum ScanError {
    /// File or directory I/O error.
    #[error("I/O error during scan: {0}")]
    Io(#[from] std::io::Error),
    /// Pattern matching error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

/// Utility class scanner configuration and executor.
#[derive(Debug, Clone)]
pub struct SourceScanner {
    extensions: Vec<String>,
}

impl Default for SourceScanner {
    fn default() -> Self {
        Self {
            extensions: vec![
                "rs".to_string(),
                "rsx".to_string(),
                "html".to_string(),
                "tera".to_string(),
            ],
        }
    }
}

impl SourceScanner {
    /// Creates a new scanner with default supported file extensions (`.rs`, `.rsx`, `.html`, `.tera`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of file extensions to inspect.
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Recursively collects all matching file paths from a base directory.
    pub fn collect_files(&self, base_dir: &Path) -> Result<Vec<PathBuf>, ScanError> {
        let mut files = Vec::new();
        if !base_dir.exists() {
            return Ok(files);
        }
        self.walk_dir_recursive(base_dir, &mut files)?;
        Ok(files)
    }

    fn walk_dir_recursive(&self, dir: &Path, acc: &mut Vec<PathBuf>) -> Result<(), ScanError> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Skip hidden directories and target builds
                    if !dir_name.starts_with('.')
                        && dir_name != "target"
                        && dir_name != "node_modules"
                    {
                        self.walk_dir_recursive(&path, acc)?;
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if self.extensions.iter().any(|target_ext| target_ext == ext) {
                        acc.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    /// Scans a single string for utility class tokens.
    pub fn scan_string(&self, content: &str) -> HashSet<String> {
        let pattern = Regex::new(r#"(?:class|className|classes!)\s*(?:=|:|\()\s*["']([^"']+)["']"#)
            .expect("Valid static regex");

        let mut classes = HashSet::new();
        for cap in pattern.captures_iter(content) {
            if let Some(matched) = cap.get(1) {
                for token in matched.as_str().split_whitespace() {
                    let clean = token.trim();
                    if !clean.is_empty() {
                        classes.insert(clean.to_string());
                    }
                }
            }
        }
        classes
    }

    /// Scans a directory of files in parallel, returning all unique utility class tokens found.
    pub fn scan_directory(&self, base_dir: &Path) -> Result<HashSet<String>, ScanError> {
        let files = self.collect_files(base_dir)?;

        let classes = files
            .par_iter()
            .map(|path| -> HashSet<String> {
                match fs::read_to_string(path) {
                    Ok(content) => self.scan_string(&content),
                    Err(_) => HashSet::new(),
                }
            })
            .reduce(HashSet::new, |mut acc, item| {
                acc.extend(item);
                acc
            });

        Ok(classes)
    }
}
