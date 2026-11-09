//! Popover and Tooltip headless primitives.
//!
//! Handles transient overlays anchored to a trigger element:
//! - Click vs Hover activation trigger strategies.
//! - Escape key closing and outside pointer press detection.
//! - Strict WAI-ARIA 1.2 `aria-haspopup`, `aria-expanded`, `aria-describedby`.

use crate::attributes::{AriaAttributes, AriaHasPopup, AriaRole};
use crate::fsm::{StateMachine, TransitionResult};

/// Interaction trigger type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverTrigger {
    /// Opens when clicked/pressed (standard popover/dropdown).
    #[default]
    Click,
    /// Opens on hover or focus (tooltip).
    HoverOrFocus,
}

/// Events for Popover/Tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopoverEvent {
    /// Trigger clicked.
    TriggerClick,
    /// Pointer entered trigger area.
    PointerEnter,
    /// Pointer left trigger or popover area.
    PointerLeave,
    /// Trigger gained keyboard focus.
    TriggerFocus,
    /// Trigger lost keyboard focus.
    TriggerBlur,
    /// Escape key pressed.
    EscapePressed,
    /// Pointer down outside popover boundary.
    OutsidePointerDown,
    /// Explicit close request.
    CloseRequested,
}

/// Actions emitted by the popover state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopoverAction {
    /// Show the overlay.
    ShowOverlay,
    /// Hide the overlay.
    HideOverlay,
    /// Start open delay timer (for hover tooltips).
    ScheduleOpenTimer(u32),
    /// Cancel any pending timer.
    CancelTimer,
    /// Return focus to the trigger element.
    RestoreTriggerFocus,
}

/// State of the popover.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PopoverState {
    /// Hidden and inactive.
    #[default]
    Closed,
    /// Waiting for hover delay timer before opening.
    PendingOpen,
    /// Visible and active on screen.
    Open,
}

/// Headless Popover / Tooltip State Machine.
#[derive(Debug, Clone)]
pub struct PopoverFsm {
    state: PopoverState,
    popover_id: String,
    trigger_mode: PopoverTrigger,
    open_delay_ms: u32,
    is_tooltip: bool,
}

impl PopoverFsm {
    /// Creates a new popover FSM.
    pub fn new(popover_id: impl Into<String>) -> Self {
        Self {
            state: PopoverState::Closed,
            popover_id: popover_id.into(),
            trigger_mode: PopoverTrigger::Click,
            open_delay_ms: 200,
            is_tooltip: false,
        }
    }

    /// Configures as a tooltip (opens on hover/focus with delay).
    pub fn as_tooltip(mut self, delay_ms: u32) -> Self {
        self.trigger_mode = PopoverTrigger::HoverOrFocus;
        self.open_delay_ms = delay_ms;
        self.is_tooltip = true;
        self
    }

    /// Whether the popover is currently open.
    pub fn is_open(&self) -> bool {
        self.state == PopoverState::Open
    }

    /// Returns ARIA attributes for the trigger button.
    pub fn trigger_aria_attributes(&self) -> AriaAttributes {
        if self.is_tooltip {
            let mut attrs = AriaAttributes::new().with_role(AriaRole::Button);
            if self.is_open() {
                attrs = attrs.with_describedby(self.popover_id.clone());
            }
            attrs
        } else {
            AriaAttributes::new()
                .with_role(AriaRole::Button)
                .with_has_popup(AriaHasPopup::Dialog)
                .with_expanded(self.is_open())
                .with_controls(self.popover_id.clone())
        }
    }

    /// Returns ARIA attributes for the popover container.
    pub fn content_aria_attributes(&self) -> AriaAttributes {
        if self.is_tooltip {
            AriaAttributes::new()
                .with_role(AriaRole::Tooltip)
                .with_hidden(!self.is_open())
        } else {
            AriaAttributes::new()
                .with_role(AriaRole::Dialog)
                .with_hidden(!self.is_open())
        }
    }
}

impl StateMachine for PopoverFsm {
    type State = PopoverState;
    type Event = PopoverEvent;
    type Action = PopoverAction;

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
            (PopoverState::Closed, PopoverEvent::TriggerClick)
                if self.trigger_mode == PopoverTrigger::Click =>
            {
                TransitionResult::with_actions(PopoverState::Open, vec![PopoverAction::ShowOverlay])
            }

            (PopoverState::Open, PopoverEvent::TriggerClick)
                if self.trigger_mode == PopoverTrigger::Click =>
            {
                TransitionResult::with_actions(
                    PopoverState::Closed,
                    vec![
                        PopoverAction::HideOverlay,
                        PopoverAction::RestoreTriggerFocus,
                    ],
                )
            }

            (PopoverState::Closed, PopoverEvent::PointerEnter | PopoverEvent::TriggerFocus)
                if self.trigger_mode == PopoverTrigger::HoverOrFocus =>
            {
                if self.open_delay_ms == 0 {
                    TransitionResult::with_actions(
                        PopoverState::Open,
                        vec![PopoverAction::ShowOverlay],
                    )
                } else {
                    TransitionResult::with_actions(
                        PopoverState::PendingOpen,
                        vec![PopoverAction::ScheduleOpenTimer(self.open_delay_ms)],
                    )
                }
            }

            (PopoverState::PendingOpen, PopoverEvent::PointerLeave | PopoverEvent::TriggerBlur) => {
                TransitionResult::with_actions(
                    PopoverState::Closed,
                    vec![PopoverAction::CancelTimer],
                )
            }

            (
                PopoverState::Open,
                PopoverEvent::PointerLeave
                | PopoverEvent::TriggerBlur
                | PopoverEvent::EscapePressed
                | PopoverEvent::OutsidePointerDown
                | PopoverEvent::CloseRequested,
            ) => TransitionResult::with_actions(
                PopoverState::Closed,
                vec![
                    PopoverAction::HideOverlay,
                    PopoverAction::RestoreTriggerFocus,
                ],
            ),

            _ => TransitionResult::state_only(self.state.clone()),
        }
    }
}
