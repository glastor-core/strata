//! Integration tests for cargo-ui CLI commands.

use glastor_ui_cli::config::GlastorUiConfig;
use glastor_ui_cli::injector::ComponentInjector;
use glastor_ui_cli::llms_doc::LlmsDocGenerator;
use glastor_ui_cli::registry::RegistryLoader;
use std::fs;
use tempfile::tempdir;

#[test]
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
