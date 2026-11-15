//! Parallel source code utility class scanner.
//!
//! Scans Rust (`.rs`, `.rsx`), HTML, and template files for CSS class tokens using
//! optimized parallel regex evaluation via `rayon`.

use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use thiserror::Error;

/// Maximum directory recursion depth to guard against deep or cyclic trees.
const MAX_SCAN_DEPTH: usize = 32;

fn attr_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?:class|className|classes!)\s*(?:=|:|\()\s*["']([^"']+)["']"#)
            .expect("Valid static attribute regex")
    })
}

fn literal_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#""([^"\\]+)""#).expect("Valid static literal regex"))
}

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
        self.walk_dir_recursive(base_dir, &mut files, 0)?;
        Ok(files)
    }

    fn walk_dir_recursive(
        &self,
        dir: &Path,
        acc: &mut Vec<PathBuf>,
        depth: usize,
    ) -> Result<(), ScanError> {
        if depth > MAX_SCAN_DEPTH || !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            // Ignore symlinks to prevent directory traversal loops (CWE-674)
            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            if file_type.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip hidden directories and target builds
                if !dir_name.starts_with('.') && dir_name != "target" && dir_name != "node_modules"
                {
                    self.walk_dir_recursive(&path, acc, depth + 1)?;
                }
            } else if file_type.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
                && self.extensions.iter().any(|target_ext| target_ext == ext)
            {
                acc.push(path);
            }
        }
        Ok(())
    }

    /// Scans a single string for utility class tokens.
    ///
    /// Extracts classes from explicit `class` / `className` / `classes!()` attributes as well as
    /// Rust string literals (e.g. `pub const OVERLAY: &str = "..."` in component libraries) whose
    /// tokens resolve to known utility rules in [`crate::rules::RulesEngine`].
    pub fn scan_string(&self, content: &str) -> HashSet<String> {
        let mut classes = HashSet::new();
        for cap in attr_regex().captures_iter(content) {
            if let Some(matched) = cap.get(1) {
                for token in matched.as_str().split_whitespace() {
                    let clean = token.trim();
                    if !clean.is_empty() {
                        classes.insert(clean.to_string());
                    }
                }
            }
        }

        for cap in literal_regex().captures_iter(content) {
            if let Some(matched) = cap.get(1) {
                for token in matched.as_str().split_whitespace() {
                    let clean = token.trim();
                    if !clean.is_empty()
                        && crate::rules::RulesEngine::resolve_entry(clean).is_some()
                    {
                        classes.insert(clean.to_string());
                    }
                }
            }
        }

        classes
    }

    /// Scans all eligible files in a directory in parallel using `rayon`, returning a deduplicated
    /// sorted list of utility class tokens.
    pub fn scan_directory(&self, base_dir: &Path) -> Result<Vec<String>, ScanError> {
        let files = self.collect_files(base_dir)?;

        let combined: HashSet<String> = files
            .par_iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .flat_map_iter(|content| self.scan_string(&content))
            .collect();

        let mut sorted: Vec<String> = combined.into_iter().collect();
        sorted.sort();
        Ok(sorted)
    }
}
