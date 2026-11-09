# glastor-ui-engine (`glastor-forge`)

[![CI](https://github.com/glastor-core/glastor-ui-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/glastor-core/glastor-ui-engine/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/glastor-aria-core.svg)](https://crates.io/crates/glastor-aria-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

> **Next-generation, zero-Node.js headless accessibility & style bundling engine for Rust web applications (Leptos, Dioxus, Yew).**

Part of the **GLASTOR® Core** infrastructure suite.

---

## Architecture Overview

```
glastor-ui-engine/
├── glastor-aria-core     Pure headless WAI-ARIA 1.2 state machines (FocusTrap, DismissableLayer, RovingTabindex)
├── glastor-css-binder    Zero-Node CSS scanning, compilation and bundling engine for build.rs and dev-watch
└── glastor-ui-cli        Decentralized component distributor and AST patcher for Cargo
```

### 1. `glastor-aria-core`
A zero-allocation, headless accessibility library for Rust web frameworks.
- **Framework-Agnostic**: Pure deterministic Finite State Machines (FSM). Zero mandatory dependencies on Leptos or Dioxus runtimes.
- **WAI-ARIA 1.2 Compliance**: Strict type-safe representation of attributes, roles, and keyboard navigation standards.
- **Enterprise WCAG 2.2 Ready**: Roving `tabindex`, boundary cycle focus trapping, dismissable layer stacks, and sentinel element abstractions.

### 2. `glastor-css-binder`
Say goodbye to `node_modules` and `package.json` in your Rust WASM CI/CD pipelines.
- **Zero Node.js Requirement**: Compiles directly within `build.rs` or in development watch mode.
- **Parallel Source Scanner**: Powered by `rayon` to extract utility classes from Rust/RSX macros in microseconds.
- **Modern CSS Pipeline**: CSS variables injection, automatic layer grouping (`@layer base, components, utilities`), and production minification.

### 3. `glastor-ui-cli`
Decentralized component distributor (`cargo ui add <component>`).
- Reads from any `registry.json` manifest.
- Surgical AST code injection via `syn` and `quote`.

---

## Quick Start

### Using `glastor-aria-core`

```rust
use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::dialog::{DialogAction, DialogEvent, DialogFsm, DialogState};

let mut dialog = DialogFsm::new("user-settings-modal");
assert_eq!(dialog.state(), &DialogState::Closed);

// User clicks open trigger
let actions = dialog.transition(DialogEvent::OpenTriggerClicked);
assert_eq!(dialog.state(), &DialogState::Open);
assert!(actions.contains(&DialogAction::TrapFocus));

// User presses Escape
let actions = dialog.transition(DialogEvent::EscapePressed);
assert_eq!(dialog.state(), &DialogState::Closed);
assert!(actions.contains(&DialogAction::RestorePriorFocus));
```

### Using `glastor-css-binder` in `build.rs`

```rust
// build.rs
fn main() {
    glastor_css_binder::BuildBinder::new()
        .scan_dir("src")
        .output_css("dist/glastor.min.css")
        .compile()
        .expect("Failed to compile CSS bundle");
}
```

---

## Ecosytem Integration

- **`byte-gauge`**: SIMD & WebAssembly binary size auditing.
- **`specter`**: AST contract and API breaking change diffing.
- **`zerho`**: Edge-hosted interactive component docs (<15ms TTFB).

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

