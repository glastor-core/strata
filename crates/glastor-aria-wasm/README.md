# glastor-aria-wasm

[![Crates.io](https://img.shields.io/crates/v/glastor-aria-wasm.svg)](https://crates.io/crates/glastor-aria-wasm)
[![Docs.rs](https://docs.rs/glastor-aria-wasm/badge.svg)](https://docs.rs/glastor-aria-wasm)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

Zero-dependency WebAssembly runtime bridge for **GLASTOR® Strata** headless accessibility finite state machines (FSMs) and harmonic spring motion physics.

Part of [**GLASTOR® Strata**](https://github.com/glastor-core/strata).

---

## Highlights

- **`wasm32-unknown-unknown` Native**: Zero JavaScript glue required. Directly callable from WebAssembly runtimes or vanilla JS.
- **Deterministic Accessibility FSMs**: Executes Dialog modal lifecycles, focus traps, and single/multiple Accordion state machines in sub-microsecond time.
- **Harmonic Spring Motion Physics**: High-precision analytical spring physics oscillator (`spring_evaluate`) formulated strictly with `f32` math primitives for fluid UI animations.
- **Zero Allocations in Critical Loops**: Fixed memory layout using bitmasks for composite widget state inspection.

---

## Exported C-ABI Functions

### Dialog Modal FSM
- `dialog_reset() -> u32`: Initializes or resets the Dialog FSM to `Closed` (0).
- `dialog_transition(event_id: u32) -> u32`: Dispatches events:
  - `1`: `OpenTriggerClicked`
  - `2`: `CloseTriggerClicked`
  - `3`: `EscapePressed`
  - `4`: `OutsidePointerDown`
  - Returns: `0` (Closed), `1` (Open).
- `dialog_is_open() -> u32`: Returns current modal state.

### Accordion FSM
- `accordion_init(count: u32, multiple: u32) -> u32`: Configures up to 8 accordion items in Single (`0`) or Multiple (`1`) mode.
- `accordion_toggle(item_index: u32) -> u32`: Toggles item (1-based index) and returns an expanded bitmask.

### Harmonic Spring Oscillator
- `spring_evaluate(k: f32, c: f32, m: f32, t: f32) -> f32`: Computes exact spring displacement at time `t` given stiffness ($k$), damping ($c$), and mass ($m$).

---

## JavaScript / WebAssembly Usage

```javascript
const wasmModule = await WebAssembly.instantiateStreaming(
  fetch("glastor_aria_wasm.wasm")
);
const { dialog_transition, dialog_is_open, spring_evaluate } = wasmModule.instance.exports;

// Open dialog
dialog_transition(1);
console.log(dialog_is_open()); // 1

// Evaluate spring physics at t = 0.2s (stiffness 180, damping 12, mass 1)
const position = spring_evaluate(180, 12, 1, 0.2);
```

---

## License

Licensed under either of:
- [Apache License, Version 2.0](../../LICENSE-APACHE)
- [MIT License](../../LICENSE-MIT)
