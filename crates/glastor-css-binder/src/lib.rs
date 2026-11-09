//! # glastor-css-binder
//!
//! Native Zero-Node CSS scanning, compilation, and bundling engine for Rust web applications.
//!
//! Eliminates Node.js and NPM dependencies from Rust WASM CI/CD pipelines by extracting
//! utility classes from `.rs` and `.rsx` source trees and producing optimized CSS bundles.
//!
//! Part of the **GLASTOR® Core** infrastructure suite.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod build_binder;
pub mod minifier;
pub mod rules;
pub mod scanner;

pub use build_binder::{BuildBinder, BuildBinderError};
pub use minifier::CssMinifier;
pub use rules::{RulesEngine, ThemeTokens};
pub use scanner::{ScanError, SourceScanner};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rules_engine_variants_and_tokens() {
        let classes = vec![
            "hover:bg-slate-800".to_string(),
            "focus-visible:outline-none".to_string(),
            "dark:bg-black".to_string(),
            "bg-primary".to_string(),
        ];

        let engine = RulesEngine::new();
        let css = engine.compile_stylesheet(&classes);

        assert!(css.contains(r".hover\:bg-slate-800:hover { background-color: #1e293b; }"));
        assert!(css.contains(r".focus-visible\:outline-none:focus-visible { outline: 2px solid transparent; outline-offset: 2px; }"));
        assert!(css.contains(r".dark .dark\:bg-black { background-color: #000000; }"));
        assert!(css.contains(".bg-primary { background-color: var(--primary); }"));

        let theme = ThemeTokens::default();
        let theme_css = theme.to_css();
        assert!(theme_css.contains("--radius: 0.5rem;"));
        assert!(theme_css.contains("--primary: #18181b;"));
    }

    #[test]
    fn test_security_css_injection_sanitization() {
        // Legitimate arbitrary values pass
        let valid = "w-[300px]".to_string();
        let engine = RulesEngine::new();
        let css = engine.compile_stylesheet(&[valid]);
        assert!(css.contains(r".w-\[300px\] { width: 300px; }"));

        // Malicious arbitrary values containing rule breaks or urls are discarded
        let malicious_break = "w-[100px;background:red]".to_string();
        let malicious_url = "bg-[url('https://evil.com/leak')]".to_string();
        let malicious_expr = "w-[expression(alert(1))]".to_string();

        let malicious_css =
            engine.compile_stylesheet(&[malicious_break, malicious_url, malicious_expr]);
        assert!(!malicious_css.contains("background:red"));
        assert!(!malicious_css.contains("evil.com"));
        assert!(!malicious_css.contains("expression"));
    }

    #[test]
    fn test_rules_engine_standard_and_arbitrary() {
        let classes = vec![
            "flex".to_string(),
            "items-center".to_string(),
            "p-4".to_string(),
            "w-[350px]".to_string(),
            "bg-slate-900".to_string(),
        ];

        let engine = RulesEngine::new();
        let css = engine.compile_stylesheet(&classes);

        assert!(css.contains(".flex { display: flex; }"));
        assert!(css.contains(".items-center { align-items: center; }"));
        assert!(css.contains(".p-4 { padding: 1rem; }"));
        assert!(css.contains(r".w-\[350px\] { width: 350px; }"));
        assert!(css.contains(".bg-slate-900 { background-color: #0f172a; }"));
    }

    #[test]
    fn test_css_minifier() {
        let input = r"
            /* Comment to strip */
            .btn {
                display: flex;
                padding: 1rem 2rem;
            }
        ";

        let minified = CssMinifier::minify(input);
        assert_eq!(minified, ".btn{display:flex;padding:1rem 2rem;}");
    }

    #[test]
    fn test_scanner_and_build_binder_flow() {
        let dir = tempdir().expect("tempdir creation");
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");

        let component_code = r#"
            #[component]
            pub fn MyButton() -> impl IntoView {
                view! {
                    <button class="flex items-center p-4 bg-slate-900 text-white rounded-md">
                        "Click me"
                    </button>
                }
            }
        "#;
        fs::write(src_dir.join("button.rs"), component_code).expect("write component");

        let output_css = dir.path().join("dist").join("style.min.css");

        let binder = BuildBinder::new()
            .scan_dir(&src_dir)
            .output_css(&output_css)
            .with_cargo_rerun(false)
            .with_minify(true);

        let result = binder.compile();
        assert!(result.is_ok());

        let generated_css = fs::read_to_string(&output_css).expect("read generated css");
        assert!(generated_css.contains(".flex{display:flex;}"));
        assert!(generated_css.contains(".items-center{align-items:center;}"));
        assert!(generated_css.contains(".p-4{padding:1rem;}"));
        assert!(generated_css.contains(".rounded-md{border-radius:0.375rem;}"));
    }
}
