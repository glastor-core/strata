//! WebAssembly bridge for GLASTOR Strata headless accessibility FSMs.
//!
//! Provides deterministic state machine execution and harmonic spring physics
//! directly in the browser via zero-dependency `wasm32-unknown-unknown`.

use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::accordion::{AccordionEvent, AccordionFsm, AccordionMode};
use glastor_aria_core::primitives::dialog::{DialogEvent, DialogFsm, DialogState};
use std::cell::RefCell;

thread_local! {
    static DIALOG: RefCell<DialogFsm> = RefCell::new(DialogFsm::new("strata-wasm-modal"));
    static ACCORDION: RefCell<AccordionFsm> = RefCell::new(AccordionFsm::new(
        vec!["item-1".to_string(), "item-2".to_string(), "item-3".to_string()],
        AccordionMode::Single,
    ));
}

/// Returns the Strata WebAssembly bridge version (0x000100 for v0.1.0).
#[unsafe(no_mangle)]
pub extern "C" fn strata_wasm_version() -> u32 {
    0x000100
}

/// Resets or initializes the dialog FSM. Returns 0 for Closed.
#[unsafe(no_mangle)]
pub extern "C" fn dialog_reset() -> u32 {
    DIALOG.with(|d| {
        *d.borrow_mut() = DialogFsm::new("strata-wasm-modal");
        0
    })
}

/// Dispatches an event to the Dialog FSM:
/// - `1` => OpenTriggerClicked
/// - `2` => CloseTriggerClicked
/// - `3` => EscapePressed
/// - `4` => OutsidePointerDown
///
/// Returns current dialog state:
/// - `0` => Closed
/// - `1` => Open
#[unsafe(no_mangle)]
pub extern "C" fn dialog_transition(event_id: u32) -> u32 {
    DIALOG.with(|d| {
        let mut fsm = d.borrow_mut();
        let event = match event_id {
            1 => DialogEvent::OpenTriggerClicked,
            2 => DialogEvent::CloseTriggerClicked,
            3 => DialogEvent::EscapePressed,
            4 => DialogEvent::OutsidePointerDown,
            _ => return if *fsm.state() == DialogState::Open { 1 } else { 0 },
        };
        let _actions = fsm.transition(event);
        if *fsm.state() == DialogState::Open { 1 } else { 0 }
    })
}

/// Returns whether the dialog is currently open.
#[unsafe(no_mangle)]
pub extern "C" fn dialog_is_open() -> u32 {
    DIALOG.with(|d| {
        if *d.borrow().state() == DialogState::Open { 1 } else { 0 }
    })
}

/// Configures accordion with item count (up to 8) and mode (0: Single, 1: Multiple).
#[unsafe(no_mangle)]
pub extern "C" fn accordion_init(count: u32, multiple: u32) -> u32 {
    ACCORDION.with(|a| {
        let items: Vec<String> = (0..count.min(8)).map(|i| format!("item-{}", i + 1)).collect();
        let mode = if multiple == 1 { AccordionMode::Multiple } else { AccordionMode::Single };
        *a.borrow_mut() = AccordionFsm::new(items, mode);
        0
    })
}

/// Toggles accordion item by 1-based index (1..8). Returns bitmask of expanded items.
#[unsafe(no_mangle)]
pub extern "C" fn accordion_toggle(item_index: u32) -> u32 {
    ACCORDION.with(|a| {
        let mut fsm = a.borrow_mut();
        let item_id = format!("item-{}", item_index);
        let _ = fsm.transition(AccordionEvent::ToggleItem(item_id));

        let mut mask: u32 = 0;
        for i in 1..=8 {
            let id = format!("item-{}", i);
            if fsm.is_expanded(&id) {
                mask |= 1 << (i - 1);
            }
        }
        mask
    })
}

/// Computes harmonic oscillator position at time `t` (0.0 .. 1.0) given stiffness (k), damping (c), mass (m).
/// Formulated strictly with Rust's f32 math primitives.
#[unsafe(no_mangle)]
pub extern "C" fn spring_evaluate(k: f32, c: f32, m: f32, t: f32) -> f32 {
    if m <= 0.0 || k <= 0.0 {
        return 1.0;
    }
    let omega_n = (k / m).sqrt();
    let zeta = c / (2.0 * (k * m).sqrt());

    if zeta < 1.0 {
        // Underdamped (oscillatory)
        let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
        let env = (-zeta * omega_n * t).exp();
        1.0 - env * ((omega_d * t).cos() + (zeta / (1.0 - zeta * zeta).sqrt()) * (omega_d * t).sin())
    } else {
        // Critically damped or overdamped
        1.0 - (-omega_n * t).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strata_wasm_dialog_lifecycle() {
        assert_eq!(dialog_reset(), 0);
        assert_eq!(dialog_is_open(), 0);
        assert_eq!(dialog_transition(1), 1); // Open
        assert_eq!(dialog_is_open(), 1);
        assert_eq!(dialog_transition(3), 0); // Escape closes
        assert_eq!(dialog_is_open(), 0);
    }

    #[test]
    fn test_strata_wasm_accordion_lifecycle() {
        assert_eq!(accordion_init(3, 0), 0);
        let mask = accordion_toggle(1);
        assert_eq!(mask, 0b001);
        // Single mode: toggling item 2 closes item 1
        let mask = accordion_toggle(2);
        assert_eq!(mask, 0b010);
    }

    #[test]
    fn test_strata_wasm_spring_math() {
        let pos_start = spring_evaluate(180.0, 12.0, 1.0, 0.0);
        assert!(pos_start.abs() < 1e-4);
        let pos_mid = spring_evaluate(180.0, 12.0, 1.0, 0.3);
        assert!(pos_mid > 0.5);
    }
}
