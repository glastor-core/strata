# glastor-ui-cli (`cargo-ui`)

[![Crates.io](https://img.shields.io/crates/v/glastor-ui-cli.svg)](https://crates.io/crates/glastor-ui-cli)
[![Docs.rs](https://docs.rs/glastor-ui-cli/badge.svg)](https://docs.rs/glastor-ui-cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

Decentralized UI distribution, AST code injector, and accessibility linter for the Cargo ecosystem.

Part of [**GLASTOR® Strata**](https://github.com/glastor-core/strata).

---

## Features

- **Multi-Framework Injection**: `cargo ui add <component> --framework <leptos|dioxus|yew>`.
- **W3C Design Tokens Sync**: `cargo ui sync-tokens tokens.json -o src/tokens.css`.
- **Accessibility Linter**: `cargo ui lint-a11y src/` verifies missing `aria-label`, empty interactive elements, and bad roles.
- **Zero-Node CSS Watcher**: `cargo ui watch src/ -o dist/bundle.min.css`.
- **Specter Contract Audit**: `cargo ui audit` certifies SemVer API compatibility.
- **AI Agent Discovery**: `cargo ui generate-llms` outputs `llms.txt` and `llms-full.txt`.

---

## CLI Commands

```bash
# Initialize project configuration
cargo ui init --framework leptos

# List canonical components in registry
cargo ui list

# Inject button component into local source tree
cargo ui add button

# Synchronize W3C design tokens
cargo ui sync-tokens tokens.json -o src/theme.css

# Run WCAG & WAI-ARIA accessibility diagnostics
cargo ui lint-a11y src/

# Run background zero-Node CSS compilation watcher
cargo ui watch src/ -o dist/bundle.min.css
```

---

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT).
