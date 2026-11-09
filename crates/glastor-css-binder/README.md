# glastor-css-binder

Zero-Node.js CSS scanning, generation, and bundling engine for Rust web applications.

## Highlights
- **Zero Node.js / NPM Requirement**: Compiles directly inside `build.rs` or local CLI dev loops.
- **Parallel Source Scanner**: Powered by `rayon` to extract utility classes across `.rs` / `.rsx` files in microseconds.
- **Wasm & CI/CD Optimized**: Outputs minified, tree-shaken CSS bundles with zero external runtimes.
- **Seamless Cargo Integration**: Direct `cargo:rerun-if-changed` integration for deterministic rebuilds.

