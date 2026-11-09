//! Surgical source code injector and Cargo workspace modifier.

use crate::registry::schema::RegistryComponent;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Orchestrates component file writes and Cargo.toml dependency verification.
#[derive(Debug, Clone, Default)]
pub struct ComponentInjector;

impl ComponentInjector {
    /// Injects a component's files into the user's project directory and updates mod.rs.
    pub fn inject_component(
        component: &RegistryComponent,
        target_dir: &Path,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        fs::create_dir_all(target_dir)?;

        let mut installed_files = Vec::new();
        let mut module_names = Vec::new();

        for file in &component.files {
            Self::validate_relative_path(&file.path)?;
            let file_path = target_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&file_path, &file.content)?;
            installed_files.push(file_path.display().to_string());

            if let Some(stem) = Path::new(&file.path).file_stem().and_then(|s| s.to_str()) {
                module_names.push(stem.to_string());
            }
        }

        // Maintain components mod.rs
        Self::update_mod_rs(target_dir, &module_names)?;

        // Check Cargo.toml dependencies
        Self::verify_cargo_dependencies(component, Path::new("Cargo.toml"))?;

        Ok(installed_files)
    }

    /// Validates that a file path is strictly relative and contains no parent ('..') or root components.
    pub fn validate_relative_path(path_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path_str);
        if path.is_absolute() {
            return Err(format!("Security error: Absolute path '{path_str}' is forbidden").into());
        }
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(format!(
                        "Security error: Path traversal detected ('..') in component path '{path_str}'"
                    )
                    .into());
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return Err(format!(
                        "Security error: Root or prefix component forbidden in path '{path_str}'"
                    )
                    .into());
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Automatically registers newly installed components inside mod.rs.
    fn update_mod_rs(dir: &Path, module_names: &[String]) -> std::io::Result<()> {
        let mod_rs_path = dir.join("mod.rs");
        let existing = if mod_rs_path.exists() {
            fs::read_to_string(&mod_rs_path)?
        } else {
            "//! GLASTOR UI Components Module.\n\n".to_string()
        };

        let mut additions = String::new();
        for module in module_names {
            let decl = format!("pub mod {module};");
            if !existing.contains(&decl) {
                additions.push_str(&format!("{decl}\n"));
            }
        }

        if !additions.is_empty() {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&mod_rs_path)?;
            file.write_all(additions.as_bytes())?;
        }

        Ok(())
    }

    /// Checks if required dependencies exist in the root Cargo.toml.
    fn verify_cargo_dependencies(
        component: &RegistryComponent,
        cargo_toml_path: &Path,
    ) -> std::io::Result<()> {
        if !cargo_toml_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(cargo_toml_path)?;

        for dep in &component.dependencies {
            if !content.contains(&dep.name) {
                println!(
                    "  [NOTICE] Component '{}' recommends dependency: {} = \"{}\"",
                    component.name, dep.name, dep.version
                );
            }
        }

        Ok(())
    }
}
