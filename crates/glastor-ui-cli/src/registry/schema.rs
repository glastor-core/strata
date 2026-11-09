//! Component registry JSON schema definition.
//!
//! Enforces strict contract specifications for distributed UI components.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Top-level manifest for a component registry (`registry.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryManifest {
    /// Registry schema version (e.g. "1.0.0").
    pub version: String,
    /// Name or organization owning the registry (e.g. "glastor-core").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Dictionary of available components keyed by name.
    pub components: BTreeMap<String, RegistryComponent>,
}

/// A component specification within the registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryComponent {
    /// Component identifier (e.g. "dialog", "button").
    pub name: String,
    /// Description of the component and accessibility features.
    pub description: String,
    /// Target frameworks supported ("leptos", "dioxus", "yew", "agnostic").
    pub frameworks: Vec<String>,
    /// Required Cargo dependencies (e.g. `glastor-aria-core = "0.1.0"`).
    #[serde(default)]
    pub dependencies: Vec<DependencyRequirement>,
    /// Files constituting this component.
    pub files: Vec<ComponentFile>,
}

/// Cargo dependency required by a component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyRequirement {
    /// Crate name on crates.io.
    pub name: String,
    /// SemVer version requirement.
    pub version: String,
    /// Optional crate features to enable.
    #[serde(default)]
    pub features: Vec<String>,
}

/// An individual source code file in a component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentFile {
    /// Target relative path (e.g. "dialog.rs" or "modal/header.rs").
    pub path: String,
    /// Raw Rust source code content.
    pub content: String,
}
