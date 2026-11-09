//! Cargo `build.rs` integration harness.
//!
//! Exposes a builder pipeline designed to run during `cargo build`, automatically
//! scanning project directories, generating atomic CSS, and emitting Cargo rerun directives.

use crate::minifier::CssMinifier;
use crate::rules::RulesEngine;
use crate::scanner::{ScanError, SourceScanner};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors arising during build script execution.
#[derive(Debug, Error)]
pub enum BuildBinderError {
    /// Scanner failure.
    #[error("Source scan error: {0}")]
    Scan(#[from] ScanError),
    /// File write or read failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Missing required configuration.
    #[error("Missing required build parameter: {0}")]
    MissingParameter(&'static str),
}

/// Orchestrator for compiling CSS within `build.rs` pipelines.
#[derive(Debug, Clone)]
pub struct BuildBinder {
    scan_dirs: Vec<PathBuf>,
    output_path: Option<PathBuf>,
    base_css: Option<String>,
    minify: bool,
    emit_cargo_rerun: bool,
}

impl Default for BuildBinder {
    fn default() -> Self {
        Self {
            scan_dirs: vec![PathBuf::from("src")],
            output_path: None,
            base_css: None,
            minify: true,
            emit_cargo_rerun: true,
        }
    }
}

impl BuildBinder {
    /// Creates a new `BuildBinder` with default settings (scans `src/`, enables minification).
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a directory to scan for utility class usages.
    pub fn scan_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.scan_dirs.push(dir.as_ref().to_path_buf());
        self
    }

    /// Sets the destination path for the compiled CSS bundle.
    pub fn output_css(mut self, path: impl AsRef<Path>) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Injects base CSS / theme variables at the beginning of the generated bundle.
    pub fn with_base_css(mut self, base: impl Into<String>) -> Self {
        self.base_css = Some(base.into());
        self
    }

    /// Controls whether the generated CSS is minified (default: true).
    pub fn with_minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    /// Controls whether `cargo:rerun-if-changed` instructions are printed to stdout (default: true).
    pub fn with_cargo_rerun(mut self, emit: bool) -> Self {
        self.emit_cargo_rerun = emit;
        self
    }

    /// Executes the scanning, bundling, and writing pipeline.
    pub fn compile(&self) -> Result<PathBuf, BuildBinderError> {
        let out_path = self
            .output_path
            .as_ref()
            .ok_or(BuildBinderError::MissingParameter("output_path"))?;

        let scanner = SourceScanner::new();
        let mut all_classes = Vec::new();

        for dir in &self.scan_dirs {
            if self.emit_cargo_rerun {
                println!("cargo:rerun-if-changed={}", dir.display());
            }

            if dir.exists() {
                let classes = scanner.scan_directory(dir)?;
                all_classes.extend(classes);
            }
        }

        all_classes.sort();
        all_classes.dedup();

        let rules_engine = RulesEngine::new();
        let mut stylesheet = String::new();

        if let Some(ref base) = self.base_css {
            stylesheet.push_str(base);
            stylesheet.push('\n');
        }

        let utility_css = rules_engine.compile_stylesheet(&all_classes);
        stylesheet.push_str(&utility_css);

        let final_css = if self.minify {
            CssMinifier::minify(&stylesheet)
        } else {
            stylesheet
        };

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(out_path, final_css)?;

        Ok(out_path.clone())
    }
}
