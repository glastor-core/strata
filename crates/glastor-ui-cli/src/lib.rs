//! # glastor-ui-cli
//!
//! Decentralized UI distribution and code injection engine for the Cargo ecosystem.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod injector;
pub mod linter;
pub mod llms_doc;
pub mod registry;
pub mod watcher;

pub use linter::{A11yDiagnostic, AccessibilityLinter, DiagnosticLevel};
pub use watcher::CssWatcher;
