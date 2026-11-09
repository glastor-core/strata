//! Integration tests for cargo-ui CLI commands.

use glastor_ui_cli::config::GlastorUiConfig;
use glastor_ui_cli::injector::ComponentInjector;
use glastor_ui_cli::llms_doc::LlmsDocGenerator;
use glastor_ui_cli::registry::RegistryLoader;
use std::fs;
use tempfile::tempdir;

#[test]
#[allow(clippy::assert_is_empty)]
fn test_cli_add_component_and_mod_rs() {
    let dir = tempdir().expect("tempdir creation");
    let components_dir = dir.path().join("src").join("components").join("ui");

    let manifest = RegistryLoader::load_manifest(None).expect("load manifest");
    let button_comp = manifest.components.get("button").expect("button component");

    let injected = ComponentInjector::inject_component(button_comp, &components_dir)
        .expect("injection succeeds");
    assert!(!injected.is_empty());

    // Check button.rs was created
    let button_rs = components_dir.join("button.rs");
    assert!(button_rs.exists());
    let content = fs::read_to_string(button_rs).expect("read button.rs");
    assert!(content.contains("ButtonVariant"));
    assert!(content.contains("button_props"));

    // Check mod.rs was created and updated
    let mod_rs = components_dir.join("mod.rs");
    assert!(mod_rs.exists());
    let mod_content = fs::read_to_string(mod_rs).expect("read mod.rs");
    assert!(mod_content.contains("pub mod button;"));
}

#[test]
fn test_cli_config_lifecycle() {
    let dir = tempdir().expect("tempdir creation");
    let config_path = dir.path().join("glastor-ui.toml");

    let cfg = GlastorUiConfig {
        framework: "dioxus".to_string(),
        ..Default::default()
    };
    cfg.save(&config_path).expect("save config");

    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).expect("read config");
    assert!(content.contains("framework = \"dioxus\""));
}

#[test]
fn test_cli_llms_doc_generation() {
    let manifest = RegistryLoader::load_manifest(None).expect("load manifest");
    let doc = LlmsDocGenerator::generate_llms_txt(&manifest);

    assert!(doc.contains("# GLASTOR® UI Engine"));
    assert!(doc.contains("`cargo ui add button`"));
    assert!(doc.contains("`cargo ui add dialog`"));
    assert!(doc.contains("`cargo ui add accordion`"));
    assert!(doc.contains("`cargo ui add grid`"));

    let full_doc = LlmsDocGenerator::generate_llms_full_txt(&manifest);
    assert!(full_doc.contains("Component Source Code Specifications"));
    assert!(full_doc.contains("ButtonVariant"));
    assert!(full_doc.contains("DialogStyles"));
}

#[test]
fn test_tokens_sync_pipeline() {
    use glastor_ui_cli::tokens_sync::TokensSynchronizer;

    let json = r##"{
        "color": {
            "brand": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let mut sync = TokensSynchronizer::new();
    let count = sync.parse_json(json).expect("valid parse");
    assert_eq!(count, 1);

    let css = sync.to_css();
    assert!(css.contains("--color-brand: #3b82f6;"));

    let rs = sync.to_rust_code();
    assert!(rs.contains("pub mod color"));
    assert!(rs.contains(r"pub const BRAND: &str = "));
}

#[test]
fn test_security_path_traversal_prevention() {
    // Attempt relative traversal escape
    let malicious = "../../../etc/passwd";
    let result = ComponentInjector::validate_relative_path(malicious);
    assert!(result.is_err());

    // Attempt absolute path escape
    let malicious_abs = if cfg!(windows) {
        "C:\\malicious.rs"
    } else {
        "/tmp/malicious.rs"
    };
    let result_abs = ComponentInjector::validate_relative_path(malicious_abs);
    assert!(result_abs.is_err());

    // Valid path passes
    let valid = "subfolder/button.rs";
    let result_valid = ComponentInjector::validate_relative_path(valid);
    assert!(result_valid.is_ok());
}

#[test]
#[allow(clippy::assert_is_empty)]
fn test_accessibility_linter_diagnostics() {
    use glastor_ui_cli::linter::{AccessibilityLinter, DiagnosticLevel};
    use std::path::Path;

    let linter = AccessibilityLinter::new();
    let mut diags = Vec::new();

    // Positive tabindex should trigger warning
    let bad_tabindex = r#"<div tabindex="5">focus</div>"#;
    linter.lint_file_content(Path::new("test.rsx"), bad_tabindex, &mut diags);
    assert!(
        diags
            .iter()
            .any(|d| d.rule_id == "WCAG-2.4.3" && d.level == DiagnosticLevel::Warning)
    );

    // Self-closing button without aria-label triggers error
    let bad_btn = r#"<button class="p-2" />"#;
    linter.lint_file_content(Path::new("test.rsx"), bad_btn, &mut diags);
    assert!(
        diags
            .iter()
            .any(|d| d.rule_id == "WCAG-4.1.2" && d.level == DiagnosticLevel::Error)
    );

    // Valid compliant button passes without error
    let mut clean_diags = Vec::new();
    let good_btn = r#"<button aria-label="Close modal">✕</button>"#;
    linter.lint_file_content(Path::new("test.rsx"), good_btn, &mut clean_diags);
    assert!(clean_diags.is_empty());
}

#[test]
fn test_css_watcher_cycle() {
    use glastor_ui_cli::watcher::CssWatcher;

    let dir = tempdir().expect("tempdir");
    let src_dir = dir.path().join("src");
    let out_file = dir.path().join("dist").join("style.min.css");
    fs::create_dir_all(&src_dir).expect("create src");

    let component_file = src_dir.join("comp.rs");
    fs::write(&component_file, r#"<div class="flex p-4"></div>"#).expect("write file");

    let mut watcher = CssWatcher::new(&src_dir, &out_file);
    let recompiled = watcher.check_and_recompile().expect("watcher check");
    assert!(recompiled);
    assert!(out_file.exists());

    // Subsequent check without edits should not recompile
    let recompiled_second = watcher.check_and_recompile().expect("watcher check 2");
    assert!(!recompiled_second);
}
