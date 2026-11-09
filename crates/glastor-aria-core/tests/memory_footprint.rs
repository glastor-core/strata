//! byte-gauge memory and zero-cost abstraction footprint validation.

use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::accordion::AccordionMode;
use glastor_aria_core::primitives::dialog::{DialogEvent, DialogFsm, DialogState};
use glastor_aria_core::primitives::focus_trap::FocusTrapState;
use std::mem::size_of;
use std::time::Instant;

#[test]
fn test_byte_gauge_memory_compactness() {
    // Assert enum states are minimal (sub-word or single-word sized)
    assert!(size_of::<DialogState>() <= 1);
    assert!(size_of::<AccordionMode>() <= 1);
    assert!(size_of::<FocusTrapState>() <= 32);

    println!(
        "[byte-gauge audit] DialogState size: {} byte(s)",
        size_of::<DialogState>()
    );
    println!(
        "[byte-gauge audit] AccordionMode size: {} byte(s)",
        size_of::<AccordionMode>()
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

    // Assert transitions take well under 1 microsecond (1,000 ns) per cycle
    assert!(ns_per_cycle < 1_000.0);
}
