//! Registry management and resolution.

pub mod builtin;
pub mod schema;

pub use builtin::get_canonical_manifest;
pub use schema::{ComponentFile, DependencyRequirement, RegistryComponent, RegistryManifest};

use std::fs;
use std::path::Path;

/// Loader and validator for component registries.
#[derive(Debug, Clone, Default)]
pub struct RegistryLoader;

impl RegistryLoader {
    /// Loads the manifest from a local file, or falls back to canonical built-in components.
    pub fn load_manifest(
        custom_path: Option<&Path>,
    ) -> Result<RegistryManifest, Box<dyn std::error::Error>> {
        if let Some(path) = custom_path {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let manifest: RegistryManifest = serde_json::from_str(&content)?;
                return Ok(manifest);
            }
        }
        Ok(get_canonical_manifest())
    }
}
