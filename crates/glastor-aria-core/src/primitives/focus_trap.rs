//! Focus Trap headless primitive.
//!
//! Enforces WCAG 2.2 focus trapping within a specified boundary (modals, dialogs, drawers).
//! Prevents keyboard users from navigating outside the active layer by catching focus at boundary
//! sentinels and wrapping around, while strictly restoring initial focus on exit.

use crate::fsm::{StateMachine, TransitionResult};

/// Identifiers for focus trap boundary sentinels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SentinelBoundary {
    /// Sentinel node rendered directly before the trapped container.
    Start,
    /// Sentinel node rendered directly after the trapped container.
    End,
}

/// Events driving the focus trap state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTrapEvent {
    /// Activates the trap, optionally specifying which element had focus previously.
    Activate {
        /// Element ID or handle that held focus before the trap was activated.
        prior_focused_id: Option<String>,
        /// Optional ID of the element that should receive initial focus inside the trap.
        initial_focus_id: Option<String>,
    },
    /// Deactivates the trap, signaling that focus should be restored.
    Deactivate,
    /// Focus reached a boundary sentinel element.
    BoundaryHit(SentinelBoundary),
    /// A node within the trap explicitly requested focus.
    FocusNode(String),
}

/// Actions emitted to the UI or DOM runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTrapAction {
    /// Focus the first focusable element inside the trapped container.
    FocusFirstNode,
    /// Focus the last focusable element inside the trapped container.
    FocusLastNode,
    /// Focus an explicit node ID.
    FocusNodeById(String),
    /// Restore focus to the element that was focused prior to activation.
    RestorePriorFocus(String),
    /// Lock keyboard navigation within boundary.
    LockBoundary,
    /// Unlock boundary navigation.
    UnlockBoundary,
}

/// State of the focus trap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FocusTrapState {
    /// The trap is inactive, normal page navigation proceeds.
    #[default]
    Inactive,
    /// The trap is actively constraining focus.
    Active {
        /// The element ID that had focus before the trap was triggered.
        prior_focused_id: Option<String>,
    },
}

/// Deterministic Focus Trap State Machine.
#[derive(Debug, Clone, Default)]
pub struct FocusTrapFsm {
    state: FocusTrapState,
    container_id: String,
}

impl FocusTrapFsm {
    /// Creates a new focus trap state machine for a specific container ID.
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            state: FocusTrapState::Inactive,
            container_id: container_id.into(),
        }
    }

    /// Returns the target container ID.
    pub fn container_id(&self) -> &str {
        &self.container_id
    }

    /// Whether the focus trap is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self.state, FocusTrapState::Active { .. })
    }
}

impl StateMachine for FocusTrapFsm {
    type State = FocusTrapState;
    type Event = FocusTrapEvent;
    type Action = FocusTrapAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let result = self.next_transition(&event);
        self.state = result.next_state;
        result.actions
    }

    fn next_transition(&self, event: &Self::Event) -> TransitionResult<Self::State, Self::Action> {
        match (&self.state, event) {
            (
                FocusTrapState::Inactive,
                FocusTrapEvent::Activate {
                    prior_focused_id,
                    initial_focus_id,
                },
            ) => {
                let mut actions = vec![FocusTrapAction::LockBoundary];
                if let Some(initial_id) = initial_focus_id {
                    actions.push(FocusTrapAction::FocusNodeById(initial_id.clone()));
                } else {
                    actions.push(FocusTrapAction::FocusFirstNode);
                }

                TransitionResult::with_actions(
                    FocusTrapState::Active {
                        prior_focused_id: prior_focused_id.clone(),
                    },
                    actions,
                )
            }

            (FocusTrapState::Active { prior_focused_id }, FocusTrapEvent::Deactivate) => {
                let mut actions = vec![FocusTrapAction::UnlockBoundary];
                if let Some(prior_id) = prior_focused_id {
                    actions.push(FocusTrapAction::RestorePriorFocus(prior_id.clone()));
                }

                TransitionResult::with_actions(FocusTrapState::Inactive, actions)
            }

            (
                FocusTrapState::Active { .. },
                FocusTrapEvent::BoundaryHit(SentinelBoundary::Start),
            ) => {
                // Focus wrapped backwards to start sentinel -> loop to last node
                TransitionResult::with_actions(
                    self.state.clone(),
                    vec![FocusTrapAction::FocusLastNode],
                )
            }

            (FocusTrapState::Active { .. }, FocusTrapEvent::BoundaryHit(SentinelBoundary::End)) => {
                // Focus wrapped forward to end sentinel -> loop to first node
                TransitionResult::with_actions(
                    self.state.clone(),
                    vec![FocusTrapAction::FocusFirstNode],
                )
            }

            (FocusTrapState::Active { .. }, FocusTrapEvent::FocusNode(id)) => {
                TransitionResult::with_actions(
                    self.state.clone(),
                    vec![FocusTrapAction::FocusNodeById(id.clone())],
                )
            }

            // Ignored/no-op transitions
            _ => TransitionResult::state_only(self.state.clone()),
        }
    }
}
