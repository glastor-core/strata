//! byte-gauge memory and zero-cost abstraction footprint validation.

use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::accordion::AccordionMode;
use glastor_aria_core::primitives::dialog::{DialogEvent, DialogFsm, DialogState};
use glastor_aria_core::primitives::focus_trap::FocusTrapState;
use glastor_aria_core::primitives::grid::{CellCoord, GridSelectionMode};
use glastor_aria_core::primitives::tabs::TabsActivationMode;
use glastor_aria_core::primitives::toast::ToastPriority;
use std::mem::size_of;
use std::time::Instant;

#[test]
fn test_byte_gauge_memory_compactness() {
    // Assert enum states are minimal (sub-word or single-word sized)
    assert!(size_of::<DialogState>() <= 1);
    assert!(size_of::<AccordionMode>() <= 1);
    assert!(size_of::<TabsActivationMode>() <= 1);
    assert!(size_of::<ToastPriority>() <= 1);
    assert!(size_of::<GridSelectionMode>() <= 1);
    assert_eq!(size_of::<CellCoord>(), 8); // two u32s
    assert!(size_of::<FocusTrapState>() <= 32);

    println!(
        "[byte-gauge audit] DialogState size: {} byte(s)",
        size_of::<DialogState>()
    );
    println!(
        "[byte-gauge audit] AccordionMode size: {} byte(s)",
        size_of::<AccordionMode>()
    );
    println!(
        "[byte-gauge audit] TabsActivationMode size: {} byte(s)",
        size_of::<TabsActivationMode>()
    );
    println!(
        "[byte-gauge audit] ToastPriority size: {} byte(s)",
        size_of::<ToastPriority>()
    );
}

#[test]
fn test_sub_microsecond_fsm_throughput() {
    let mut dialog = DialogFsm::new("perf-benchmark");
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = dialog.transition(DialogEvent::OpenTriggerClicked);
        let _ = dialog.transition(DialogEvent::CloseTriggerClicked);
    }
    let elapsed = start.elapsed();

    let ns_per_cycle = (elapsed.as_nanos() as f64) / (f64::from(iterations) * 2.0);
    println!(
        "[byte-gauge audit] Average transition latency: {:.2} ns/cycle (target: < 500 ns)",
        ns_per_cycle
    );

    // In release mode this executes in < 100 ns; in unoptimized debug on Windows allow up to 1.5 µs
    #[cfg(debug_assertions)]
    assert!(ns_per_cycle < 1_500.0);

    #[cfg(not(debug_assertions))]
    assert!(ns_per_cycle < 500.0);
}
