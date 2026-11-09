//! Example demo showing glastor-aria-core and glastor-css-binder build integration.

use glastor_aria_core::fsm::StateMachine;
use glastor_aria_core::primitives::accordion::{AccordionEvent, AccordionFsm, AccordionMode};
use glastor_aria_core::primitives::dialog::{DialogEvent, DialogFsm};

fn main() {
    println!("=== GLASTOR UI Engine — Turnkey Example ===");

    // 1. Accessibility FSM
    let mut dialog = DialogFsm::new("checkout-modal");
    println!("Initial Dialog State: {:?}", dialog.state());

    let open_actions = dialog.transition(DialogEvent::OpenTriggerClicked);
    println!("State after trigger click: {:?}", dialog.state());
    println!("Emitted DOM Actions: {:?}", open_actions);

    let container_attrs = dialog.container_aria_attributes();
    println!("Generated ARIA Attributes for Container:");
    for (k, v) in container_attrs.to_key_values() {
        println!("  {k}=\"{v}\"");
    }

    // 2. Accordion Primitive
    let items = vec!["shipping".to_string(), "payment".to_string()];
    let mut accordion = AccordionFsm::new(items, AccordionMode::Single);
    let _ = accordion.transition(AccordionEvent::ToggleItem("payment".to_string()));
    println!("\nAccordion 'payment' expanded: {}", accordion.is_expanded("payment"));

    // 3. Simulated component markup containing utility classes
    let _simulated_markup = r#"
        <div class="flex items-center justify-between p-4 bg-primary text-primary-foreground rounded-lg hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 dark:bg-black w-[350px]">
            <span>Accessible Modal Ready</span>
        </div>
    "#;

    println!("\nZero-Node CSS bundle compiled into dist/style.min.css during build.rs!");
}
