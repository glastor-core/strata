//! Roving Tabindex headless primitive.
//!
//! Implements the WAI-ARIA authoring practices design pattern for keyboard navigation
//! across composite widgets (toolbars, menu bars, tab lists, radio groups, grids).
//! Maintains `tabindex="0"` on the actively focused item while assigning `tabindex="-1"`
//! to all sibling elements. Skips disabled items during arrow key traversal.

use crate::attributes::AriaOrientation;
use crate::fsm::{StateMachine, TransitionResult};

/// Navigation direction for keyboard roving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RovingDirection {
    /// Next element in reading order.
    Next,
    /// Previous element in reading order.
    Previous,
    /// First element in container.
    First,
    /// Last element in container.
    Last,
}

/// Item metadata tracked within a roving container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RovingItem {
    /// Unique identifier or element reference.
    pub id: String,
    /// Whether this element is disabled and should be bypassed during arrow traversal.
    pub is_disabled: bool,
}

impl RovingItem {
    /// Creates a new enabled roving item.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            is_disabled: false,
        }
    }

    /// Sets the disabled status of the item.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }
}

/// Events driving the roving tabindex state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RovingTabindexEvent {
    /// Navigate in a specific direction via arrow/home/end key.
    Navigate(RovingDirection),
    /// Explicitly focus an item by its index.
    FocusByIndex(usize),
    /// Update the collection of tracked items.
    UpdateItems(Vec<RovingItem>),
}

/// Actions emitted by the roving tabindex primitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RovingTabindexAction {
    /// Direct focus to the item at index.
    FocusIndex {
        /// Position index.
        index: usize,
        /// Element ID.
        id: String,
    },
    /// Update tabindex attributes on DOM nodes.
    SetTabindex {
        /// Position index.
        index: usize,
        /// 0 for the active item, -1 for inactive items.
        value: i8,
    },
}

/// State of roving tabindex navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RovingTabindexState {
    /// The index of the item currently possessing `tabindex="0"`.
    pub active_index: Option<usize>,
    /// Tracked collection of items.
    pub items: Vec<RovingItem>,
}

/// Headless Roving Tabindex State Machine.
#[derive(Debug, Clone)]
pub struct RovingTabindexFsm {
    state: RovingTabindexState,
    orientation: AriaOrientation,
    loop_around: bool,
}

impl RovingTabindexFsm {
    /// Creates a new roving tabindex state machine.
    pub fn new(items: Vec<RovingItem>, orientation: AriaOrientation) -> Self {
        let active_index = items.iter().position(|item| !item.is_disabled);
        Self {
            state: RovingTabindexState {
                active_index,
                items,
            },
            orientation,
            loop_around: true,
        }
    }

    /// Configures whether arrow navigation loops around boundaries.
    pub fn with_loop(mut self, should_loop: bool) -> Self {
        self.loop_around = should_loop;
        self
    }

    /// Returns the orientation configured.
    pub fn orientation(&self) -> AriaOrientation {
        self.orientation
    }

    /// Returns the index of the currently active element.
    pub fn active_index(&self) -> Option<usize> {
        self.state.active_index
    }

    /// Finds the next enabled index in a specified direction.
    fn find_target_index(&self, direction: RovingDirection) -> Option<usize> {
        let total = self.state.items.len();
        if total == 0 {
            return None;
        }

        match direction {
            RovingDirection::First => self.state.items.iter().position(|i| !i.is_disabled),
            RovingDirection::Last => self.state.items.iter().rposition(|i| !i.is_disabled),
            RovingDirection::Next => {
                let current = self.state.active_index.unwrap_or(0);
                for step in 1..total {
                    let idx = (current + step) % total;
                    if !self.loop_around && idx < current {
                        break;
                    }
                    if !self.state.items[idx].is_disabled {
                        return Some(idx);
                    }
                }
                None
            }
            RovingDirection::Previous => {
                let current = self.state.active_index.unwrap_or(0);
                for step in 1..total {
                    let idx = (current + total - (step % total)) % total;
                    if !self.loop_around && idx > current {
                        break;
                    }
                    if !self.state.items[idx].is_disabled {
                        return Some(idx);
                    }
                }
                None
            }
        }
    }
}

impl StateMachine for RovingTabindexFsm {
    type State = RovingTabindexState;
    type Event = RovingTabindexEvent;
    type Action = RovingTabindexAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let result = self.next_transition(&event);
        self.state = result.next_state;
        result.actions
    }

    fn next_transition(&self, event: &Self::Event) -> TransitionResult<Self::State, Self::Action> {
        match event {
            RovingTabindexEvent::Navigate(direction) => {
                if let Some(target) = self.find_target_index(*direction) {
                    let old_active = self.state.active_index;
                    let target_id = self.state.items[target].id.clone();

                    let mut actions = Vec::with_capacity(3);
                    if let Some(old) = old_active {
                        actions.push(RovingTabindexAction::SetTabindex {
                            index: old,
                            value: -1,
                        });
                    }
                    actions.push(RovingTabindexAction::SetTabindex {
                        index: target,
                        value: 0,
                    });
                    actions.push(RovingTabindexAction::FocusIndex {
                        index: target,
                        id: target_id,
                    });

                    let mut next_state = self.state.clone();
                    next_state.active_index = Some(target);
                    TransitionResult::with_actions(next_state, actions)
                } else {
                    TransitionResult::state_only(self.state.clone())
                }
            }

            RovingTabindexEvent::FocusByIndex(target) => {
                if *target < self.state.items.len() && !self.state.items[*target].is_disabled {
                    let old_active = self.state.active_index;
                    let target_id = self.state.items[*target].id.clone();

                    let mut actions = Vec::with_capacity(3);
                    if let Some(old) = old_active {
                        actions.push(RovingTabindexAction::SetTabindex {
                            index: old,
                            value: -1,
                        });
                    }
                    actions.push(RovingTabindexAction::SetTabindex {
                        index: *target,
                        value: 0,
                    });
                    actions.push(RovingTabindexAction::FocusIndex {
                        index: *target,
                        id: target_id,
                    });

                    let mut next_state = self.state.clone();
                    next_state.active_index = Some(*target);
                    TransitionResult::with_actions(next_state, actions)
                } else {
                    TransitionResult::state_only(self.state.clone())
                }
            }

            RovingTabindexEvent::UpdateItems(new_items) => {
                let new_active = new_items.iter().position(|i| !i.is_disabled);
                let next_state = RovingTabindexState {
                    active_index: new_active,
                    items: new_items.clone(),
                };
                TransitionResult::state_only(next_state)
            }
        }
    }
}
