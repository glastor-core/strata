# Leptos Modal Demo (`glastor-aria-core` + `glastor-css-binder`)

Minimal client-side WASM example showing **GLASTOR® Strata** working end-to-end in **Leptos 0.7** without `node_modules` or `package.json`:

1. `build.rs` scans `src/` with `glastor-css-binder` and writes `dist/style.min.css`.
2. `ReactiveFsm` binds `DialogFsm` to a Leptos `RwSignal<DialogState>`.
3. `DomDialogSink` executes `FocusFirstElement`, `RestorePriorFocus`, `LockBodyScroll`, and `UnlockBodyScroll` in the browser DOM.

## Run Locally

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd examples/leptos-modal-demo
trunk serve --open
```
