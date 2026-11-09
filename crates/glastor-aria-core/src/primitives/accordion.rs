//! Accordion / Collapsible headless primitive.
//!
//! Conforms to WAI-ARIA Accordion design pattern:
//! - Multi-item or single-item expansion mode.
//! - Directional arrow key navigation between accordion triggers.
//! - `Home` / `End` navigation to first/last triggers.
//! - Automatic generation of `aria-expanded`, `aria-controls`, and `aria-disabled` attributes.

use crate::attributes::{AriaAttributes, AriaRole};
use crate::fsm::{StateMachine, TransitionResult};
use std::collections::HashSet;

/// Accordion item identifier.
pub type AccordionItemId = String;

/// Selection mode for an accordion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccordionMode {
    /// Only one item can be expanded at any time.
    #[default]
    Single,
    /// Multiple items can be expanded simultaneously.
    Multiple,
}

/// Navigation direction across accordion headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionNavDirection {
    /// Move focus to next header.
    Next,
    /// Move focus to previous header.
    Previous,
    /// Move focus to first header.
    First,
    /// Move focus to last header.
    Last,
}

/// Events driving the accordion state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccordionEvent {
    /// Toggle expansion of an item by ID.
    ToggleItem(AccordionItemId),
    /// Navigate between headers via keyboard.
    Navigate(AccordionNavDirection),
    /// Set the list of registered item IDs in order.
    RegisterItems(Vec<AccordionItemId>),
}

/// Side-effect actions emitted by the accordion machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccordionAction {
    /// Move DOM focus to the header of the specified item.
    FocusHeader(AccordionItemId),
    /// Notify observers of an expansion change.
    ItemExpanded(AccordionItemId),
    /// Notify observers of a collapse change.
    ItemCollapsed(AccordionItemId),
}

/// State of the accordion primitive.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccordionState {
    /// Currently expanded items.
    pub expanded_items: HashSet<AccordionItemId>,
    /// Ordered list of item IDs.
    pub items: Vec<AccordionItemId>,
    /// Index of the currently focused header.
    pub focused_index: Option<usize>,
}

/// Headless Accordion State Machine.
#[derive(Debug, Clone)]
pub struct AccordionFsm {
    state: AccordionState,
    mode: AccordionMode,
    allow_collapsing_all: bool,
}

impl AccordionFsm {
    /// Creates a new accordion state machine with a given list of item IDs.
    pub fn new(items: Vec<AccordionItemId>, mode: AccordionMode) -> Self {
        Self {
            state: AccordionState {
                expanded_items: HashSet::new(),
                items,
                focused_index: None,
            },
            mode,
            allow_collapsing_all: true,
        }
    }

    /// Sets whether all items are allowed to be collapsed simultaneously.
    pub fn with_allow_collapsing_all(mut self, allow: bool) -> Self {
        self.allow_collapsing_all = allow;
        self
    }

    /// Returns whether a given item is currently expanded.
    pub fn is_expanded(&self, id: &str) -> bool {
        self.state.expanded_items.contains(id)
    }

    /// Generates strict ARIA attributes for an item trigger button.
    pub fn trigger_aria_attributes(&self, item_id: &str) -> AriaAttributes {
        AriaAttributes::new()
            .with_role(AriaRole::Button)
            .with_expanded(self.is_expanded(item_id))
            .with_controls(format!("{item_id}-content"))
    }

    /// Generates strict ARIA attributes for an item content panel.
    pub fn panel_aria_attributes(&self, item_id: &str) -> AriaAttributes {
        AriaAttributes::new()
            .with_role(AriaRole::TabPanel)
            .with_labelledby(format!("{item_id}-trigger"))
            .with_hidden(!self.is_expanded(item_id))
    }
}

impl StateMachine for AccordionFsm {
    type State = AccordionState;
    type Event = AccordionEvent;
    type Action = AccordionAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let result = self.next_transition(&event);
        self.state = result.next_state;
        result.actions
    }

    fn next_transition(&self, event: &Self::Event) -> TransitionResult<Self::State, Self::Action> {
        let mut next_state = self.state.clone();
        let mut actions = Vec::new();

        match event {
            AccordionEvent::ToggleItem(id) => {
                let is_open = next_state.expanded_items.contains(id);

                if is_open {
                    if self.allow_collapsing_all || next_state.expanded_items.len() > 1 {
                        next_state.expanded_items.remove(id);
                        actions.push(AccordionAction::ItemCollapsed(id.clone()));
                    }
                } else {
                    if self.mode == AccordionMode::Single {
                        for prev_id in next_state.expanded_items.drain() {
                            actions.push(AccordionAction::ItemCollapsed(prev_id));
                        }
                    }
                    next_state.expanded_items.insert(id.clone());
                    actions.push(AccordionAction::ItemExpanded(id.clone()));
                }

                TransitionResult::with_actions(next_state, actions)
            }

            AccordionEvent::Navigate(dir) => {
                let count = next_state.items.len();
                if count == 0 {
                    return TransitionResult::state_only(next_state);
                }

                let current = next_state.focused_index.unwrap_or(0);
                let target = match dir {
                    AccordionNavDirection::First => 0,
                    AccordionNavDirection::Last => count - 1,
                    AccordionNavDirection::Next => (current + 1) % count,
                    AccordionNavDirection::Previous => (current + count - 1) % count,
                };

                next_state.focused_index = Some(target);
                let target_id = next_state.items[target].clone();
                actions.push(AccordionAction::FocusHeader(target_id));

                TransitionResult::with_actions(next_state, actions)
            }

            AccordionEvent::RegisterItems(items) => {
                next_state.items.clone_from(items);
                next_state.expanded_items.retain(|id| items.contains(id));
                TransitionResult::state_only(next_state)
            }
        }
    }
}
