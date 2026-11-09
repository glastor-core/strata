//! # glastor-aria-core
//!
//! Headless WAI-ARIA 1.2 accessibility primitives and deterministic state machines
//! for Rust web frameworks (Leptos, Dioxus, Yew).
//!
//! Part of the **GLASTOR® Core** infrastructure suite.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapter;
pub mod attributes;
pub mod fsm;
pub mod primitives;

pub use attributes::{AriaAttributes, AriaHasPopup, AriaOrientation, AriaRole};
pub use fsm::{StateMachine, TransitionResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{ActionRecorder, ActionSink};
    use crate::primitives::accordion::*;
    use crate::primitives::dialog::*;
    use crate::primitives::dismissable::*;
    use crate::primitives::focus_trap::*;
    use crate::primitives::popover::*;
    use crate::primitives::roving_tabindex::*;

    #[test]
    fn test_accordion_single_mode_lifecycle() {
        let items = vec!["sec-1".to_string(), "sec-2".to_string()];
        let mut accordion = AccordionFsm::new(items, AccordionMode::Single);

        assert!(!accordion.is_expanded("sec-1"));
        let actions = accordion.transition(AccordionEvent::ToggleItem("sec-1".to_string()));
        assert!(accordion.is_expanded("sec-1"));
        assert!(actions.contains(&AccordionAction::ItemExpanded("sec-1".to_string())));

        // Expanding sec-2 in Single mode should collapse sec-1
        let actions = accordion.transition(AccordionEvent::ToggleItem("sec-2".to_string()));
        assert!(!accordion.is_expanded("sec-1"));
        assert!(accordion.is_expanded("sec-2"));
        assert!(actions.contains(&AccordionAction::ItemCollapsed("sec-1".to_string())));
        assert!(actions.contains(&AccordionAction::ItemExpanded("sec-2".to_string())));
    }

    #[test]
    fn test_popover_tooltip_trigger() {
        let mut tooltip = PopoverFsm::new("tooltip-1").as_tooltip(0);
        assert!(!tooltip.is_open());

        let actions = tooltip.transition(PopoverEvent::PointerEnter);
        assert!(tooltip.is_open());
        assert_eq!(actions, vec![PopoverAction::ShowOverlay]);

        let actions = tooltip.transition(PopoverEvent::PointerLeave);
        assert!(!tooltip.is_open());
        assert!(actions.contains(&PopoverAction::HideOverlay));
    }

    #[test]
    fn test_dialog_lifecycle_and_actions() {
        let mut dialog = DialogFsm::new("test-modal");
        assert_eq!(dialog.state(), &DialogState::Closed);

        // Open Dialog
        let actions = dialog.transition(DialogEvent::OpenTriggerClicked);
        assert_eq!(dialog.state(), &DialogState::Open);
        assert!(actions.contains(&DialogAction::TrapFocus));
        assert!(actions.contains(&DialogAction::FocusFirstElement));

        // ARIA attributes check
        let trigger_attrs = dialog.trigger_aria_attributes();
        assert_eq!(trigger_attrs.expanded, Some(true));
        assert_eq!(trigger_attrs.has_popup, Some(AriaHasPopup::Dialog));

        let container_attrs = dialog.container_aria_attributes();
        assert_eq!(container_attrs.role, Some(AriaRole::Dialog));
        assert_eq!(container_attrs.modal, Some(true));

        // Press Escape -> Closes and restores focus
        let actions = dialog.transition(DialogEvent::EscapePressed);
        assert_eq!(dialog.state(), &DialogState::Closed);
        assert!(actions.contains(&DialogAction::RestorePriorFocus));
        assert!(actions.contains(&DialogAction::PreventDefault));
    }

    #[test]
    fn test_focus_trap_sentinel_wrapping() {
        let mut trap = FocusTrapFsm::new("modal-container");
        let recorder = ActionRecorder::new();

        let actions = trap.transition(FocusTrapEvent::Activate {
            prior_focused_id: Some("open-btn".to_string()),
            initial_focus_id: None,
        });
        recorder.emit_all(actions);
        assert!(trap.is_active());

        // Hit end sentinel -> wraps to first
        let actions = trap.transition(FocusTrapEvent::BoundaryHit(SentinelBoundary::End));
        assert_eq!(actions, vec![FocusTrapAction::FocusFirstNode]);

        // Hit start sentinel -> wraps to last
        let actions = trap.transition(FocusTrapEvent::BoundaryHit(SentinelBoundary::Start));
        assert_eq!(actions, vec![FocusTrapAction::FocusLastNode]);

        // Deactivate -> Restores prior focus
        let actions = trap.transition(FocusTrapEvent::Deactivate);
        assert_eq!(
            actions,
            vec![
                FocusTrapAction::UnlockBoundary,
                FocusTrapAction::RestorePriorFocus("open-btn".to_string())
            ]
        );
        assert!(!trap.is_active());
    }

    #[test]
    fn test_dismissable_layer_stack_isolation() {
        let mut stack = DismissableStack::new();
        stack.push(10);
        stack.push(20); // Top layer

        assert!(!stack.is_top(10));
        assert!(stack.is_top(20));

        let mut layer_nested = DismissableFsm::new(20);
        let _ = layer_nested.transition(DismissableEvent::Mount);

        // Escape on top layer -> dismisses top layer
        let actions = layer_nested.transition(DismissableEvent::EscapeKeyDown);
        assert!(actions.contains(&DismissableAction::DismissSelf));

        stack.remove(20);
        assert!(stack.is_top(10));
    }

    #[test]
    fn test_roving_tabindex_navigation_with_disabled() {
        let items = vec![
            RovingItem::new("item-1"),
            RovingItem::new("item-2").with_disabled(true), // Disabled!
            RovingItem::new("item-3"),
        ];

        let mut roving = RovingTabindexFsm::new(items, AriaOrientation::Horizontal);
        assert_eq!(roving.active_index(), Some(0));

        // Navigate Next -> Should skip item-2 and jump directly to item-3
        let actions = roving.transition(RovingTabindexEvent::Navigate(RovingDirection::Next));
        assert_eq!(roving.active_index(), Some(2));
        assert!(actions.contains(&RovingTabindexAction::SetTabindex {
            index: 0,
            value: -1
        }));
        assert!(actions.contains(&RovingTabindexAction::SetTabindex { index: 2, value: 0 }));
        assert!(actions.contains(&RovingTabindexAction::FocusIndex {
            index: 2,
            id: "item-3".to_string()
        }));

        // Navigate Next from last element -> Loops back to item-1
        let _ = roving.transition(RovingTabindexEvent::Navigate(RovingDirection::Next));
        assert_eq!(roving.active_index(), Some(0));
    }

    #[test]
    fn test_aria_attributes_formatting() {
        let attrs = AriaAttributes::new()
            .with_role(AriaRole::Button)
            .with_expanded(true)
            .with_has_popup(AriaHasPopup::Menu)
            .with_controls("dropdown-menu-1")
            .with_tabindex(0);

        let kvs = attrs.to_key_values();
        assert!(kvs.contains(&("role", "button".to_string())));
        assert!(kvs.contains(&("aria-expanded", "true".to_string())));
        assert!(kvs.contains(&("aria-haspopup", "menu".to_string())));
        assert!(kvs.contains(&("aria-controls", "dropdown-menu-1".to_string())));
        assert!(kvs.contains(&("tabindex", "0".to_string())));
    }
}
