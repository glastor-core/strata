# GLASTOR® Strata

[![CI](https://github.com/glastor-core/strata/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/glastor-core/strata/actions/workflows/ci.yml)
[![glastor-aria-core](https://img.shields.io/crates/v/glastor-aria-core.svg?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/glastor-aria-core)
[![aria downloads](https://img.shields.io/crates/d/glastor-aria-core.svg?style=flat-square&color=blue)](https://crates.io/crates/glastor-aria-core)
[![glastor-css-binder](https://img.shields.io/crates/v/glastor-css-binder.svg?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/glastor-css-binder)
[![glastor-ui-cli](https://img.shields.io/crates/v/glastor-ui-cli.svg?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/glastor-ui-cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=flat-square)](LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org)
[![Protected Branch](https://img.shields.io/badge/branch-main%20%5Bprotected%5D-success.svg?style=flat-square)](.github/BRANCH_PROTECTION.md)

> **Next-generation, zero-Node.js headless accessibility & style bundling engine for Rust web applications (Leptos, Dioxus, Yew).**

Official Repository: [https://github.com/glastor-core/strata](https://github.com/glastor-core/strata)  
Part of the **GLASTOR® Core** infrastructure suite.

---

## Architecture Overview

```
strata/
├── crates/
│   ├── glastor-aria-core     Headless WAI-ARIA 1.2 state machines (Dialog, Accordion, Popover, FocusTrap, Motion, Grid)
│   ├── glastor-css-binder    Zero-Node CSS scanning, compilation and bundling engine for build.rs and dev-watch
│   └── glastor-ui-cli        Decentralized component distributor and AST patcher for Cargo (cargo ui)
├── examples/
│   └── wasm-modal-demo       Turnkey executable example validating build.rs CSS compilation & Wasm FSM
├── .byte-gauge.toml          byte-gauge SIMD vectorization & sub-microsecond Wasm footprint audit
├── .specter.toml             specter AST API Contract & SemVer diffing configuration
├── zerho.json                zerho Edge documentation (<15ms TTFB) & Deno Deploy configuration
└── index.html                Interactive visual showcase and playground landing page
```

### 1. `glastor-aria-core`
A zero-allocation, headless accessibility library for Rust web frameworks.
- **Framework-Agnostic**: Pure deterministic Finite State Machines (FSM). Zero mandatory dependencies on Leptos or Dioxus runtimes.
- **WAI-ARIA 1.2 Compliance**: Strict type-safe representation of attributes, roles, and keyboard navigation standards.
- **WCAG 2.2 Ready**: Roving `tabindex`, boundary cycle focus trapping, dismissable layer stacks, accordion accordions, and sentinel element abstractions.
- **Mobile Hardening**: Emits `LockBodyScroll` and `UnlockBodyScroll` to prevent background elastic scroll bounce on mobile Safari/iOS.

### 2. `glastor-css-binder`
Say goodbye to `node_modules` and `package.json` in your Rust WASM CI/CD pipelines.
- **Zero Node.js Requirement**: Compiles directly within `build.rs` or in development watch mode.
- **Parallel Source Scanner**: Powered by `rayon` to extract utility classes from Rust/RSX macros in microseconds.
- **Variant Modifiers**: Supports `hover:`, `focus:`, `focus-visible:`, `active:`, `disabled:`, `dark:`, and arbitrary values (`w-[350px]`).
- **AppSec Sanitization**: Rejects dangerous rule-breaking characters and CSS data exfiltration vectors (`url(`, `expression(`, `javascript:`).
- **Design Tokens**: Generates modern theme variables (`--radius`, `--primary`, `--background`, `--foreground`).

### 3. `glastor-ui-cli` (`cargo ui`)
Decentralized component distributor (`cargo ui add <component>`).
- Reads from any `registry.json` manifest.
- Surgical AST code injection with automatic `mod.rs` maintenance and `Cargo.toml` dependency verification.
- Built-in Zip Slip / Path Traversal prevention.
- AI Agent discovery via `cargo ui generate-llms` (`llms.txt` and `llms-full.txt`).

---

## Quick Start

### Using `glastor-aria-core`

```rust
use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::dialog::{DialogAction, DialogEvent, DialogFsm, DialogState};

let mut dialog = DialogFsm::new("checkout-modal");
assert_eq!(dialog.state(), &DialogState::Closed);

// User clicks open trigger
let actions = dialog.transition(DialogEvent::OpenTriggerClicked);
assert_eq!(dialog.state(), &DialogState::Open);
assert!(actions.contains(&DialogAction::TrapFocus));
assert!(actions.contains(&DialogAction::LockBodyScroll));

// User presses Escape
let actions = dialog.transition(DialogEvent::EscapePressed);
assert_eq!(dialog.state(), &DialogState::Closed);
assert!(actions.contains(&DialogAction::RestorePriorFocus));
assert!(actions.contains(&DialogAction::UnlockBodyScroll));
```

### Using `glastor-css-binder` in `build.rs`

```rust
// build.rs
fn main() {
    glastor_css_binder::BuildBinder::new()
        .scan_dir("src")
        .output_css("dist/style.min.css")
        .compile()
        .expect("Failed to compile zero-Node CSS bundle");
}
```

### Using the CLI (`cargo-ui`)

```bash
# Initialize project configuration
cargo ui init --framework leptos

# List canonical components in registry
cargo ui list

# Inject button component into local source tree
cargo ui add button

# Audit API contract compatibility (specter standard)
cargo ui audit

# Generate AI documentation indexes (zerho edge delivery)
cargo ui generate-llms
```

---

## GLASTOR® Core Ecosystem Integration

| Tool | Integration Target | Role & Verification |
| :--- | :--- | :--- |
| **`byte-gauge`** | [`.byte-gauge.toml`](.byte-gauge.toml) | Audits Wasm binary footprint (<45KB), SIMD vectorization, and ensures FSM transitions execute in <500ns with &le;1 byte state memory. |
| **`specter`** | [`.specter.toml`](.specter.toml) | Generates `target/specter/contract-snapshot.json` via `cargo ui audit` and enforces SemVer backwards compatibility on public APIs. |
| **`zerho`** | [`zerho.json`](zerho.json) | Deploys interactive documentation and live playgrounds on Deno Deploy with TTFB <15ms using `llms.txt` and `llms-full.txt`. |

---

## Running the Turnkey Example

```bash
cargo run -p wasm-modal-demo
```

Compiles the zero-Node CSS bundle automatically during `build.rs` and runs the FSM accessibility lifecycle.

---

## Interactive Showcase

To preview the live interactive showcase landing page, open `index.html` in any web browser.

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
