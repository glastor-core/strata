# glastor-aria-core

[![Crates.io](https://img.shields.io/crates/v/glastor-aria-core.svg)](https://crates.io/crates/glastor-aria-core)
[![Docs.rs](https://docs.rs/glastor-aria-core/badge.svg)](https://docs.rs/glastor-aria-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

Headless WAI-ARIA 1.2 accessibility primitives and deterministic state machines for Rust web frameworks (Leptos, Dioxus, Yew, Vanilla WASM).

Part of [**GLASTOR® Strata**](https://github.com/glastor-core/strata).

---

## Highlights

- **`#![forbid(unsafe_code)]`**: 100% safe Rust memory guarantees.
- **Deterministic FSMs**: Event-driven state machines with zero framework coupling.
- **`FocusTrap`**: Boundary sentinel cycling with prior focus restoration.
- **`DismissableLayer`**: Multi-layer stacked escape & outside pointer dismissal.
- **`RovingTabindex`**: Keyboard navigation for composite interactive widgets.
- **`GridFsm`**: 2D virtualized grid keyboard traversal & visible window slicing.
- **WCAG 2.2 AAA & WAI-ARIA 1.2 Ready**: Strictly verified by `byte-gauge` SIMD vectorization (<500ns transition, &le;1 byte state memory).

---

## Quick Start

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

---

## Supported Primitives

| Primitive | State Machine | Standards & Roles |
| :--- | :--- | :--- |
| **Dialog** | `DialogFsm` | `role="dialog"`, `aria-modal="true"`, focus wrap sentinels |
| **Accordion** | `AccordionFsm` | `aria-expanded`, single & multi modes, keyboard traversal |
| **Popover / Tooltip** | `PopoverFsm` | `aria-haspopup`, delay timers, dismissable layers |
| **Tabs** | `TabsFsm` | `role="tablist"`, roving tabindex, horizontal/vertical |
| **Toast** | `ToastFsm` | `role="status"`, priority queues, duration timers |
| **Virtual Grid** | `GridFsm` | 2D roving navigation, Home/End, window virtualization |

---

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT).
