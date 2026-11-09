//! Local project configuration (`glastor-ui.toml`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Glastor UI project configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlastorUiConfig {
    /// Target web framework ("leptos", "dioxus", "yew", or "agnostic").
    pub framework: String,
    /// Destination directory for injected UI components.
    pub components_dir: PathBuf,
    /// Output path for the zero-Node compiled CSS bundle.
    pub css_output: PathBuf,
    /// CSS source directories to scan.
    pub scan_dirs: Vec<PathBuf>,
}

impl Default for GlastorUiConfig {
    fn default() -> Self {
        Self {
            framework: "leptos".to_string(),
            components_dir: PathBuf::from("src/components/ui"),
            css_output: PathBuf::from("dist/glastor.min.css"),
            scan_dirs: vec![PathBuf::from("src")],
        }
    }
}

impl GlastorUiConfig {
    /// Config file name.
    pub const FILE_NAME: &'static str = "glastor-ui.toml";

    /// Loads configuration from the current or parent directory.
    pub fn load_or_default() -> Self {
        let path = Path::new(Self::FILE_NAME);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                // Parse simple key-value or fallback
                return Self::parse_simple_toml(&content);
            }
        }
        Self::default()
    }

    /// Saves configuration to file.
    pub fn save(&self, target_path: &Path) -> std::io::Result<()> {
        let mut content = String::new();
        content.push_str("# GLASTOR UI Engine Configuration\n");
        content.push_str(&format!("framework = \"{}\"\n", self.framework));
        content.push_str(&format!(
            "components_dir = \"{}\"\n",
            self.components_dir.display()
        ));
        content.push_str(&format!("css_output = \"{}\"\n", self.css_output.display()));
        fs::write(target_path, content)
    }

    fn parse_simple_toml(content: &str) -> Self {
        let mut cfg = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"');
                match key {
                    "framework" => cfg.framework = val.to_string(),
                    "components_dir" => cfg.components_dir = PathBuf::from(val),
                    "css_output" => cfg.css_output = PathBuf::from(val),
                    _ => {}
                }
            }
        }
        cfg
    }
}
