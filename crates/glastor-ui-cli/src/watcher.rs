//! Real-time hot-reloading CSS file watcher daemon.
//!
//! Watches `.rs`, `.rsx`, and `.html` files in a source directory, detects modifications,
//! and recompiles the zero-Node CSS bundle in microsecond bursts without waiting
//! for Rust WebAssembly compilation.

use glastor_css_binder::BuildBinder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// Configuration and runner for the CSS watcher daemon.
#[derive(Debug)]
pub struct CssWatcher {
    scan_dir: PathBuf,
    output_css: PathBuf,
    poll_interval: Duration,
    file_snapshots: HashMap<PathBuf, SystemTime>,
}

impl CssWatcher {
    /// Creates a new CSS watcher daemon.
    pub fn new(scan_dir: impl Into<PathBuf>, output_css: impl Into<PathBuf>) -> Self {
        Self {
            scan_dir: scan_dir.into(),
            output_css: output_css.into(),
            poll_interval: Duration::from_millis(250),
            file_snapshots: HashMap::new(),
        }
    }

    /// Sets the polling interval.
    #[must_use]
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Scans the directory, checks for changes, and recompiles if needed.
    /// Returns `true` if a recompilation was triggered.
    pub fn check_and_recompile(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut current_snapshots = HashMap::new();
        Self::collect_snapshots(&self.scan_dir, &mut current_snapshots)?;

        let has_changed = current_snapshots != self.file_snapshots;

        if has_changed {
            let start = Instant::now();
            BuildBinder::new()
                .scan_dir(&self.scan_dir)
                .output_css(&self.output_css)
                .with_cargo_rerun(false)
                .compile()?;
            let elapsed = start.elapsed();

            println!(
                "[cargo-ui watch] Recompiled zero-Node CSS bundle in {:.2?} -> {}",
                elapsed,
                self.output_css.display()
            );

            self.file_snapshots = current_snapshots;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Runs the watcher daemon loop until interrupted.
    pub fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[cargo-ui watch] Monitoring '{}' for utility class changes...",
            self.scan_dir.display()
        );
        println!("  Output bundle: {}", self.output_css.display());
        println!("  Press Ctrl+C to terminate.");

        // Initial compilation pass
        let _ = self.check_and_recompile()?;

        loop {
            thread::sleep(self.poll_interval);
            let _ = self.check_and_recompile()?;
        }
    }

    fn collect_snapshots(
        current: &Path,
        snapshots: &mut HashMap<PathBuf, SystemTime>,
    ) -> Result<(), std::io::Error> {
        if !current.exists() {
            return Ok(());
        }

        if current.is_dir() {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name == "target"
                        || name == "dist"
                        || name == "node_modules"
                    {
                        continue;
                    }
                }
                Self::collect_snapshots(&path, snapshots)?;
            }
        } else if let Some(ext) = current.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "rs" | "rsx" | "html") {
                let metadata = fs::metadata(current)?;
                if let Ok(mtime) = metadata.modified() {
                    snapshots.insert(current.to_path_buf(), mtime);
                }
            }
        }

        Ok(())
    }
}
