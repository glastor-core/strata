//! llms.txt generator for edge delivery (zerho integration).
//!
//! Generates structured Markdown and llms.txt indexes allowing AI agents
//! to effortlessly discover and integrate components with zero ambiguity.

use crate::registry::schema::RegistryManifest;

/// Generator for AI agent documentation files (`llms.txt`).
#[derive(Debug, Clone, Default)]
pub struct LlmsDocGenerator;

impl LlmsDocGenerator {
    /// Generates the standard `llms.txt` content for an entire manifest.
    pub fn generate_llms_txt(manifest: &RegistryManifest) -> String {
        let mut doc = String::new();
        doc.push_str("# GLASTOR® UI Engine (`glastor-forge`) — Component Registry\n\n");
        doc.push_str(
            "> Next-generation headless accessibility & zero-Node CSS bundling for Rust WASM.\n\n",
        );
        doc.push_str("## Available Components\n\n");

        for (name, comp) in &manifest.components {
            doc.push_str(&format!("- **`{}`**: {}\n", name, comp.description));
            doc.push_str(&format!("  - Frameworks: {}\n", comp.frameworks.join(", ")));
            let deps: Vec<String> = comp
                .dependencies
                .iter()
                .map(|d| format!("{} v{}", d.name, d.version))
                .collect();
            doc.push_str(&format!("  - Dependencies: {}\n", deps.join(", ")));
            doc.push_str(&format!("  - CLI Command: `cargo ui add {}`\n\n", name));
        }

        doc.push_str("## Core Features\n\n");
        doc.push_str("1. **Zero-Node CSS**: Powered by `glastor-css-binder`, compiles directly in `build.rs`.\n");
        doc.push_str("2. **WCAG 2.2 / WAI-ARIA 1.2**: Pure headless state machines in `glastor-aria-core`.\n");
        doc.push_str("3. **Surgical Injection**: `cargo ui add <component>` injects Rust source code directly.\n");

        doc
    }
}
