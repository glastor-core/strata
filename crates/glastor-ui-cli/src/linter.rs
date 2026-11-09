//! Static WCAG 2.2 / WAI-ARIA accessibility compliance linter for Rust UI code.
//!
//! Scans `.rs`, `.rsx`, and `.html` source trees for common accessibility anti-patterns:
//! - Icon buttons lacking `aria-label` or inner accessible text.
//! - Modal dialogs missing `aria-labelledby` or `aria-describedby`.
//! - Elements with `tabindex > 0` (violates natural focus order).
//! - Popup triggers with `aria-haspopup` missing `aria-expanded`.

use std::fs;
use std::path::{Path, PathBuf};

/// Severity level of an accessibility violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Actionable warning.
    Warning,
    /// Severe WCAG failure.
    Error,
}

/// A single accessibility diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11yDiagnostic {
    /// File path where the violation was detected.
    pub file: PathBuf,
    /// Line number (1-based).
    pub line: usize,
    /// Rule identifier (e.g. `WCAG-4.1.2`).
    pub rule_id: &'static str,
    /// Severity level.
    pub level: DiagnosticLevel,
    /// Human-readable explanation and fix suggestion.
    pub message: String,
}

/// Linter engine for static accessibility auditing.
#[derive(Debug, Default)]
pub struct AccessibilityLinter;

impl AccessibilityLinter {
    /// Creates a new accessibility linter.
    pub fn new() -> Self {
        Self
    }

    /// Recursively scans a directory and lints all `.rs`, `.rsx`, and `.html` files.
    pub fn lint_directory(&self, dir: &Path) -> Result<Vec<A11yDiagnostic>, std::io::Error> {
        let mut diagnostics = Vec::new();
        self.walk_and_lint(dir, &mut diagnostics)?;
        Ok(diagnostics)
    }

    fn walk_and_lint(
        &self,
        current: &Path,
        diagnostics: &mut Vec<A11yDiagnostic>,
    ) -> Result<(), std::io::Error> {
        if !current.exists() {
            return Ok(());
        }

        if current.is_dir() {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();
                // Skip target, dist, node_modules, and git directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name == "target"
                        || name == "dist"
                        || name == "node_modules"
                        || name == "tests"
                    {
                        continue;
                    }
                }
                self.walk_and_lint(&path, diagnostics)?;
            }
        } else if let Some(ext) = current.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "rs" | "rsx" | "html") {
                let content = fs::read_to_string(current)?;
                self.lint_file_content(current, &content, diagnostics);
            }
        }

        Ok(())
    }

    /// Lints a single file's content in memory.
    pub fn lint_file_content(
        &self,
        file: &Path,
        content: &str,
        diagnostics: &mut Vec<A11yDiagnostic>,
    ) {
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip comments and doc comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            // Rule 1: Tabindex greater than 0 (WCAG 2.4.3 Focus Order)
            if trimmed.contains("tabindex=\"") || trimmed.contains("tabindex = \"") {
                for i in 1..=9 {
                    let needle = format!("tabindex=\"{i}\"");
                    let needle_space = format!("tabindex = \"{i}\"");
                    if trimmed.contains(&needle) || trimmed.contains(&needle_space) {
                        diagnostics.push(A11yDiagnostic {
                            file: file.to_path_buf(),
                            line: line_num,
                            rule_id: "WCAG-2.4.3",
                            level: DiagnosticLevel::Warning,
                            message: format!(
                                "Avoid positive tabindex ('{i}'). Use 0 for focusable elements or -1 for roving tabindex."
                            ),
                        });
                    }
                }
            }

            // Rule 2: Button lacking accessible label
            if trimmed.contains("<button") {
                let is_self_closing = trimmed.contains("/>");
                let has_aria_label =
                    trimmed.contains("aria-label") || trimmed.contains("aria-labelledby");
                let is_icon_only = trimmed.contains("<svg") || trimmed.contains("Icon");

                if is_self_closing && !has_aria_label {
                    diagnostics.push(A11yDiagnostic {
                        file: file.to_path_buf(),
                        line: line_num,
                        rule_id: "WCAG-4.1.2",
                        level: DiagnosticLevel::Error,
                        message: "Button element has no accessible name. Add 'aria-label' or inner text content.".to_string(),
                    });
                } else if is_icon_only && !has_aria_label {
                    diagnostics.push(A11yDiagnostic {
                        file: file.to_path_buf(),
                        line: line_num,
                        rule_id: "WCAG-1.1.1",
                        level: DiagnosticLevel::Warning,
                        message:
                            "Icon-only button appears to lack 'aria-label' or 'aria-labelledby'."
                                .to_string(),
                    });
                }
            }

            // Rule 3: Popup trigger without expanded state
            if trimmed.contains("aria-haspopup") && !trimmed.contains("aria-expanded") {
                diagnostics.push(A11yDiagnostic {
                    file: file.to_path_buf(),
                    line: line_num,
                    rule_id: "WAI-ARIA-1.2",
                    level: DiagnosticLevel::Warning,
                    message: "Element with 'aria-haspopup' should declare 'aria-expanded' ('true' | 'false').".to_string(),
                });
            }

            // Rule 4: Dialog modal without accessible title
            if (trimmed.contains("role=\"dialog\"") || trimmed.contains("role=\"alertdialog\""))
                && !trimmed.contains("aria-labelledby")
                && !trimmed.contains("aria-label")
            {
                diagnostics.push(A11yDiagnostic {
                    file: file.to_path_buf(),
                    line: line_num,
                    rule_id: "WCAG-2.4.6",
                    level: DiagnosticLevel::Error,
                    message: "Dialog or AlertDialog is missing 'aria-labelledby' or 'aria-label' pointing to its title heading.".to_string(),
                });
            }
        }
    }
}
