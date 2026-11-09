//! Dismissable Layer headless primitive.
//!
//! Provides WCAG compliant dismissal logic for overlays, dropdowns, and dialogs.
//! Manages nested dismissable layer stacks so that events (Escape key, outside click)
//! only dismiss the topmost active layer rather than closing the entire application context.

use crate::fsm::{StateMachine, TransitionResult};

/// Global or tree-level layer stack identifier.
pub type LayerId = u64;

/// Events directed at a dismissable layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DismissableEvent {
    /// Mounts and registers the layer at the top of the active stack.
    Mount,
    /// Unmounts and deregisters the layer.
    Unmount,
    /// User pressed the Escape key.
    EscapeKeyDown,
    /// User clicked or touched outside the layer boundary.
    PointerDownOutside {
        /// True if the click happened on a known child or friend layer.
        is_interactive_target: bool,
    },
    /// Explicit request to dismiss.
    DismissRequested,
}

/// Actions emitted by the dismissable layer state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DismissableAction {
    /// Close this specific layer.
    DismissSelf,
    /// Prevent default browser behavior on the event (e.g. stop event propagation).
    ConsumeEvent,
    /// Inform the global layer stack that this layer became top-level.
    RegisterAsTop,
    /// Inform the global layer stack that this layer was removed.
    UnregisterFromStack,
}

/// State of a dismissable layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DismissableState {
    /// Layer is unmounted or closed.
    #[default]
    Idle,
    /// Layer is mounted and active on screen.
    Mounted {
        /// Whether this layer currently sits at the top of the dismissable stack.
        is_top_layer: bool,
    },
    /// Dismissal has been triggered.
    Dismissed,
}

/// Dismissable Layer State Machine.
#[derive(Debug, Clone)]
pub struct DismissableFsm {
    layer_id: LayerId,
    state: DismissableState,
    close_on_escape: bool,
    close_on_pointer_outside: bool,
}

impl DismissableFsm {
    /// Creates a new dismissable layer state machine with standard defaults (closes on Escape and outside click).
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            layer_id,
            state: DismissableState::Idle,
            close_on_escape: true,
            close_on_pointer_outside: true,
        }
    }

    /// Configures whether Escape closes the layer.
    pub fn with_close_on_escape(mut self, enabled: bool) -> Self {
        self.close_on_escape = enabled;
        self
    }

    /// Configures whether clicking outside closes the layer.
    pub fn with_close_on_pointer_outside(mut self, enabled: bool) -> Self {
        self.close_on_pointer_outside = enabled;
        self
    }

    /// Returns the unique layer ID.
    pub fn layer_id(&self) -> LayerId {
        self.layer_id
    }

    /// Updates whether this layer is marked as the top of the active stack.
    pub fn set_top_layer(&mut self, is_top: bool) {
        if let DismissableState::Mounted {
            ref mut is_top_layer,
        } = self.state
        {
            *is_top_layer = is_top;
        }
    }
}

impl StateMachine for DismissableFsm {
    type State = DismissableState;
    type Event = DismissableEvent;
    type Action = DismissableAction;

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
            (DismissableState::Idle, DismissableEvent::Mount) => TransitionResult::with_actions(
                DismissableState::Mounted { is_top_layer: true },
                vec![DismissableAction::RegisterAsTop],
            ),

            (DismissableState::Mounted { .. }, DismissableEvent::Unmount) => {
                TransitionResult::with_actions(
                    DismissableState::Idle,
                    vec![DismissableAction::UnregisterFromStack],
                )
            }

            (DismissableState::Mounted { is_top_layer: true }, DismissableEvent::EscapeKeyDown)
                if self.close_on_escape =>
            {
                TransitionResult::with_actions(
                    DismissableState::Dismissed,
                    vec![
                        DismissableAction::ConsumeEvent,
                        DismissableAction::DismissSelf,
                        DismissableAction::UnregisterFromStack,
                    ],
                )
            }

            (
                DismissableState::Mounted { is_top_layer: true },
                DismissableEvent::PointerDownOutside {
                    is_interactive_target: false,
                },
            ) if self.close_on_pointer_outside => TransitionResult::with_actions(
                DismissableState::Dismissed,
                vec![
                    DismissableAction::ConsumeEvent,
                    DismissableAction::DismissSelf,
                    DismissableAction::UnregisterFromStack,
                ],
            ),

            (DismissableState::Mounted { .. }, DismissableEvent::DismissRequested) => {
                TransitionResult::with_actions(
                    DismissableState::Dismissed,
                    vec![
                        DismissableAction::DismissSelf,
                        DismissableAction::UnregisterFromStack,
                    ],
                )
            }

            _ => TransitionResult::state_only(self.state.clone()),
        }
    }
}

/// Stack of dismissable layers to track hierarchy.
#[derive(Debug, Clone, Default)]
pub struct DismissableStack {
    layers: Vec<LayerId>,
}

impl DismissableStack {
    /// Creates an empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a layer to the top of the stack.
    pub fn push(&mut self, layer: LayerId) {
        self.layers.retain(|&id| id != layer);
        self.layers.push(layer);
    }

    /// Removes a layer from the stack.
    pub fn remove(&mut self, layer: LayerId) {
        self.layers.retain(|&id| id != layer);
    }

    /// Checks if a given layer is the current topmost layer.
    pub fn is_top(&self, layer: LayerId) -> bool {
        self.layers.last().copied() == Some(layer)
    }

    /// Current count of active layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// True if the stack has no active layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}
