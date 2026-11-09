//! Toast and notification live region headless primitive (WCAG 2.2 & WAI-ARIA 1.2).
//!
//! Complies with the W3C Live Regions design pattern:
//! - Support for `Polite` (`aria-live="polite"`, `role="status"`) and
//!   `Assertive` (`aria-live="assertive"`, `role="alert"`).
//! - Automatic countdown timers with pause-on-hover and pause-on-focus.
//! - Priority-aware queueing (Urgent, High, Normal).
//! - Swipe-to-dismiss and programmatical dismissal triggers.

use crate::attributes::{AriaAttributes, AriaRole};
use crate::fsm::{StateMachine, TransitionResult};
use std::collections::VecDeque;

/// Unique toast identifier.
pub type ToastId = String;

/// Priority level for live region announcements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ToastPriority {
    /// Non-intrusive status update (`aria-live="polite"`).
    #[default]
    Normal,
    /// High-priority notification.
    High,
    /// Urgent alert requiring immediate screen reader interruption (`aria-live="assertive"`).
    Urgent,
}

/// Descriptor of a toast notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastDescriptor {
    /// Unique toast identifier.
    pub id: ToastId,
    /// Text message or announcement content.
    pub message: String,
    /// Urgency level.
    pub priority: ToastPriority,
    /// Duration in milliseconds before automatic dismissal (0 = persistent).
    pub duration_ms: u32,
}

/// Events driving the Toast state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastEvent {
    /// Schedule a new toast for display.
    Enqueue(ToastDescriptor),
    /// Pointer hovered over active toast (pause countdown).
    PointerEnter,
    /// Pointer left active toast (resume countdown).
    PointerLeave,
    /// Active toast gained keyboard focus (pause countdown).
    FocusIn,
    /// Active toast lost keyboard focus (resume countdown).
    FocusOut,
    /// Timer elapsed by delta milliseconds.
    TimerTick(u32),
    /// Explicit request to dismiss a toast.
    Dismiss(ToastId),
    /// Gesture-based swipe dismissal.
    SwipeDismiss(ToastId),
}

/// Actions emitted on toast transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAction {
    /// Announce the toast to assistive technology.
    AnnounceLiveRegion {
        /// ID of announced toast.
        id: ToastId,
        /// Message text.
        message: String,
        /// Whether announcement is assertive or polite.
        assertive: bool,
    },
    /// Schedule a timer for countdown in the host environment.
    ScheduleTimer(u32),
    /// Cancel current countdown timer.
    CancelTimer,
    /// Toast has been dismissed and should be removed from view.
    ToastDismissed(ToastId),
}

/// State of currently displaying toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastState {
    /// No toast is currently displayed.
    Idle,
    /// Active toast countdown is ticking.
    Showing {
        /// Active toast info.
        current: ToastDescriptor,
        /// Remaining time in ms.
        remaining_ms: u32,
    },
    /// Countdown is paused (e.g. hovered or focused).
    Paused {
        /// Active toast info.
        current: ToastDescriptor,
        /// Remaining time in ms.
        remaining_ms: u32,
    },
}

/// Headless Toast notification state machine with live region coordination.
#[derive(Debug, Clone)]
pub struct ToastFsm {
    state: ToastState,
    queue: VecDeque<ToastDescriptor>,
    max_queue_size: usize,
}

impl Default for ToastFsm {
    fn default() -> Self {
        Self::new(5)
    }
}

impl ToastFsm {
    /// Creates a new Toast state machine with a maximum queue capacity.
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            state: ToastState::Idle,
            queue: VecDeque::new(),
            max_queue_size,
        }
    }

    /// Returns the currently visible toast, if any.
    pub fn active_toast(&self) -> Option<&ToastDescriptor> {
        match &self.state {
            ToastState::Showing { current, .. } | ToastState::Paused { current, .. } => {
                Some(current)
            }
            ToastState::Idle => None,
        }
    }

    /// Returns remaining queue length.
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Generates ARIA attributes for the live region viewport container.
    pub fn viewport_aria_attributes(&self, assertive: bool) -> AriaAttributes {
        let role = if assertive {
            AriaRole::Alert
        } else {
            AriaRole::Status
        };

        AriaAttributes::new()
            .with_role(role)
            .with_atomic(true)
            .with_live(if assertive { "assertive" } else { "polite" })
    }

    /// Generates ARIA attributes for a specific toast element.
    pub fn toast_aria_attributes(&self, toast: &ToastDescriptor) -> AriaAttributes {
        let role = if toast.priority == ToastPriority::Urgent {
            AriaRole::Alert
        } else {
            AriaRole::Status
        };

        AriaAttributes::new().with_role(role).with_tabindex(0)
    }

    fn pop_next_toast(&mut self) -> Option<ToastDescriptor> {
        self.queue.pop_front()
    }
}

impl StateMachine for ToastFsm {
    type State = ToastState;
    type Event = ToastEvent;
    type Action = ToastAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let mut actions = Vec::new();

        match event {
            ToastEvent::Enqueue(toast) => {
                // If urgent, prioritize to front of queue
                if toast.priority == ToastPriority::Urgent {
                    self.queue.push_front(toast);
                } else if self.queue.len() < self.max_queue_size {
                    self.queue.push_back(toast);
                }

                // If currently idle, immediately display next
                if matches!(self.state, ToastState::Idle) {
                    if let Some(next) = self.pop_next_toast() {
                        let duration = next.duration_ms;
                        let assertive = next.priority == ToastPriority::Urgent;

                        actions.push(ToastAction::AnnounceLiveRegion {
                            id: next.id.clone(),
                            message: next.message.clone(),
                            assertive,
                        });

                        if duration > 0 {
                            actions.push(ToastAction::ScheduleTimer(duration));
                        }

                        self.state = ToastState::Showing {
                            remaining_ms: duration,
                            current: next,
                        };
                    }
                }
            }

            ToastEvent::PointerEnter | ToastEvent::FocusIn => {
                if let ToastState::Showing {
                    current,
                    remaining_ms,
                } = &self.state
                {
                    actions.push(ToastAction::CancelTimer);
                    self.state = ToastState::Paused {
                        current: current.clone(),
                        remaining_ms: *remaining_ms,
                    };
                }
            }

            ToastEvent::PointerLeave | ToastEvent::FocusOut => {
                if let ToastState::Paused {
                    current,
                    remaining_ms,
                } = &self.state
                {
                    if *remaining_ms > 0 {
                        actions.push(ToastAction::ScheduleTimer(*remaining_ms));
                    }
                    self.state = ToastState::Showing {
                        current: current.clone(),
                        remaining_ms: *remaining_ms,
                    };
                }
            }

            ToastEvent::TimerTick(delta_ms) => {
                if let ToastState::Showing {
                    current,
                    remaining_ms,
                } = &mut self.state
                {
                    if *remaining_ms > delta_ms {
                        *remaining_ms -= delta_ms;
                    } else {
                        // Expired
                        let dismissed_id = current.id.clone();
                        actions.push(ToastAction::ToastDismissed(dismissed_id));
                        self.state = ToastState::Idle;

                        // Check next toast in queue
                        if let Some(next) = self.pop_next_toast() {
                            let duration = next.duration_ms;
                            let assertive = next.priority == ToastPriority::Urgent;

                            actions.push(ToastAction::AnnounceLiveRegion {
                                id: next.id.clone(),
                                message: next.message.clone(),
                                assertive,
                            });

                            if duration > 0 {
                                actions.push(ToastAction::ScheduleTimer(duration));
                            }

                            self.state = ToastState::Showing {
                                remaining_ms: duration,
                                current: next,
                            };
                        }
                    }
                }
            }

            ToastEvent::Dismiss(id) | ToastEvent::SwipeDismiss(id) => {
                let should_dismiss = match &self.state {
                    ToastState::Showing { current, .. } | ToastState::Paused { current, .. } => {
                        current.id == id
                    }
                    ToastState::Idle => false,
                };

                if should_dismiss {
                    actions.push(ToastAction::CancelTimer);
                    actions.push(ToastAction::ToastDismissed(id));
                    self.state = ToastState::Idle;

                    // Immediately show next queued item
                    if let Some(next) = self.pop_next_toast() {
                        let duration = next.duration_ms;
                        let assertive = next.priority == ToastPriority::Urgent;

                        actions.push(ToastAction::AnnounceLiveRegion {
                            id: next.id.clone(),
                            message: next.message.clone(),
                            assertive,
                        });

                        if duration > 0 {
                            actions.push(ToastAction::ScheduleTimer(duration));
                        }

                        self.state = ToastState::Showing {
                            remaining_ms: duration,
                            current: next,
                        };
                    }
                } else {
                    // Remove from queue if present
                    self.queue.retain(|t| t.id != id);
                }
            }
        }

        actions
    }

    fn next_transition(&self, event: &Self::Event) -> TransitionResult<Self::State, Self::Action> {
        let mut clone = self.clone();
        let actions = clone.transition(event.clone());
        TransitionResult::with_actions(clone.state, actions)
    }
}
