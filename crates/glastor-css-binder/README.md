# glastor-css-binder

[![Crates.io](https://img.shields.io/crates/v/glastor-css-binder.svg)](https://crates.io/crates/glastor-css-binder)
[![Docs.rs](https://docs.rs/glastor-css-binder/badge.svg)](https://docs.rs/glastor-css-binder)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

Zero-Node.js CSS scanning, generation, and bundling engine for Rust web applications.

Part of [**GLASTOR® Strata**](https://github.com/glastor-core/strata).

---

## Highlights

- **`#![forbid(unsafe_code)]`**: 100% memory safety.
- **Zero Node.js / NPM Requirement**: Compiles directly inside `build.rs` or in development watch loops.
- **Parallel Source Scanner**: Powered by `rayon` to extract utility classes across `.rs` / `.rsx` trees in microseconds.
- **Spring Motion Physics**: Generates real damped harmonic oscillator `@keyframes` (`SpringConfig { stiffness, damping, mass }`).
- **W3C Design Tokens**: Seamless synchronization with modern CSS custom property themes.
- **AppSec Hardening**: Built-in sanitization against CSS injection vectors (`url(`, `expression(`, `javascript:`).

---

## Usage in `build.rs`

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

---

## Spring Physics Animation

```rust
use glastor_css_binder::SpringConfig;

let bouncy = SpringConfig::bouncy();
let keyframes = bouncy.generate_keyframes("strata-spring-pop", 60);
// Outputs smooth, zero-Node CSS keyframes with real harmonic overshoot
```

---

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT).
