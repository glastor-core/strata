//! Dialog / Modal headless composite primitive.
//!
//! Orchestrates the Focus Trap, Dismissable Layer, and WAI-ARIA 1.2 `dialog` / `alertdialog`
//! specifications into a cohesive, deterministic component machine.

use crate::attributes::{AriaAttributes, AriaRole};
use crate::fsm::{StateMachine, TransitionResult};
use crate::primitives::dismissable::{DismissableAction, DismissableEvent, DismissableFsm};
use crate::primitives::focus_trap::{
    FocusTrapAction, FocusTrapEvent, FocusTrapFsm, SentinelBoundary,
};

/// High-level events for a headless dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogEvent {
    /// User clicked the trigger button to open dialog.
    OpenTriggerClicked,
    /// User clicked an explicit close button within the dialog.
    CloseTriggerClicked,
    /// Escape key pressed.
    EscapePressed,
    /// Pointer down outside dialog content.
    OutsidePointerDown,
    /// Tab reached start or end sentinel boundary.
    SentinelHit(SentinelBoundary),
}

/// Side-effect actions emitted by the dialog state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogAction {
    /// Lock focus within dialog boundary.
    TrapFocus,
    /// Restore focus to element that triggered the dialog.
    RestorePriorFocus,
    /// Consume the event to prevent background scrolling or bubbling.
    PreventDefault,
    /// Focus boundary wrap to first element.
    FocusFirstElement,
    /// Focus boundary wrap to last element.
    FocusLastElement,
    /// Lock body scroll to prevent elastic background movement (mobile Safari/iOS).
    LockBodyScroll,
    /// Unlock body scroll when dialog is dismissed.
    UnlockBodyScroll,
}

/// State of the dialog.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DialogState {
    /// Dialog is closed and hidden from the accessibility tree.
    #[default]
    Closed,
    /// Dialog is open, visible, and trapping focus.
    Open,
}

/// Unified Headless Dialog State Machine.
#[derive(Debug, Clone)]
pub struct DialogFsm {
    dialog_id: String,
    trigger_id: String,
    state: DialogState,
    is_modal: bool,
    is_alert: bool,
    focus_trap: FocusTrapFsm,
    dismissable: DismissableFsm,
}

impl DialogFsm {
    /// Creates a new dialog FSM with a given identifier.
    pub fn new(dialog_id: impl Into<String>) -> Self {
        let id = dialog_id.into();
        let trigger_id = format!("{id}-trigger");
        Self {
            focus_trap: FocusTrapFsm::new(id.clone()),
            dismissable: DismissableFsm::new(1),
            dialog_id: id,
            trigger_id,
            state: DialogState::Closed,
            is_modal: true,
            is_alert: false,
        }
    }

    /// Sets whether the dialog is an `alertdialog` (requires explicit confirmation).
    pub fn with_alert(mut self, alert: bool) -> Self {
        self.is_alert = alert;
        if alert {
            // Alert dialogs typically cannot be dismissed via outside click
            self.dismissable = self.dismissable.with_close_on_pointer_outside(false);
        }
        self
    }

    /// Returns the unique dialog ID.
    pub fn dialog_id(&self) -> &str {
        &self.dialog_id
    }

    /// Generates the strict ARIA attributes for the dialog trigger button.
    pub fn trigger_aria_attributes(&self) -> AriaAttributes {
        AriaAttributes::new()
            .with_role(AriaRole::Button)
            .with_has_popup(crate::attributes::AriaHasPopup::Dialog)
            .with_expanded(self.state == DialogState::Open)
            .with_controls(self.dialog_id.clone())
    }

    /// Generates the strict ARIA attributes for the dialog container element.
    pub fn container_aria_attributes(&self) -> AriaAttributes {
        let role = if self.is_alert {
            AriaRole::AlertDialog
        } else {
            AriaRole::Dialog
        };

        AriaAttributes::new()
            .with_role(role)
            .with_modal(self.is_modal)
            .with_labelledby(format!("{}-title", self.dialog_id))
            .with_describedby(format!("{}-description", self.dialog_id))
    }
}

impl StateMachine for DialogFsm {
    type State = DialogState;
    type Event = DialogEvent;
    type Action = DialogAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        match (&self.state, &event) {
            (DialogState::Closed, DialogEvent::OpenTriggerClicked) => {
                let trap_actions = self.focus_trap.transition(FocusTrapEvent::Activate {
                    prior_focused_id: Some(self.trigger_id.clone()),
                    initial_focus_id: None,
                });
                let _ = self.dismissable.transition(DismissableEvent::Mount);
                self.state = DialogState::Open;

                let mut actions = vec![DialogAction::TrapFocus, DialogAction::LockBodyScroll];
                if trap_actions.contains(&FocusTrapAction::FocusFirstNode) {
                    actions.push(DialogAction::FocusFirstElement);
                }
                actions
            }

            (DialogState::Open, DialogEvent::CloseTriggerClicked) => {
                let _ = self.focus_trap.transition(FocusTrapEvent::Deactivate);
                let _ = self
                    .dismissable
                    .transition(DismissableEvent::DismissRequested);
                self.state = DialogState::Closed;
                vec![
                    DialogAction::RestorePriorFocus,
                    DialogAction::UnlockBodyScroll,
                ]
            }

            (DialogState::Open, DialogEvent::EscapePressed) => {
                let dismiss_actions = self.dismissable.transition(DismissableEvent::EscapeKeyDown);
                if dismiss_actions.contains(&DismissableAction::DismissSelf) {
                    let _ = self.focus_trap.transition(FocusTrapEvent::Deactivate);
                    self.state = DialogState::Closed;
                    vec![
                        DialogAction::PreventDefault,
                        DialogAction::RestorePriorFocus,
                        DialogAction::UnlockBodyScroll,
                    ]
                } else {
                    Vec::new()
                }
            }

            (DialogState::Open, DialogEvent::OutsidePointerDown) => {
                let dismiss_actions =
                    self.dismissable
                        .transition(DismissableEvent::PointerDownOutside {
                            is_interactive_target: false,
                        });
                if dismiss_actions.contains(&DismissableAction::DismissSelf) {
                    let _ = self.focus_trap.transition(FocusTrapEvent::Deactivate);
                    self.state = DialogState::Closed;
                    vec![
                        DialogAction::RestorePriorFocus,
                        DialogAction::UnlockBodyScroll,
                    ]
                } else {
                    Vec::new()
                }
            }

            (DialogState::Open, DialogEvent::SentinelHit(boundary)) => {
                let trap_actions = self
                    .focus_trap
                    .transition(FocusTrapEvent::BoundaryHit(*boundary));
                let mut actions = Vec::new();
                if trap_actions.contains(&FocusTrapAction::FocusFirstNode) {
                    actions.push(DialogAction::FocusFirstElement);
                } else if trap_actions.contains(&FocusTrapAction::FocusLastNode) {
                    actions.push(DialogAction::FocusLastElement);
                }
                actions
            }

            _ => Vec::new(),
        }
    }

    fn next_transition(&self, event: &Self::Event) -> TransitionResult<Self::State, Self::Action> {
        let mut clone = self.clone();
        let actions = clone.transition(event.clone());
        TransitionResult::with_actions(clone.state, actions)
    }
}
