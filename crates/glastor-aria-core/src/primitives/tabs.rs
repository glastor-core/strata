//! Tabs headless accessibility primitive (WAI-ARIA 1.2).
//!
//! Complies with the W3C Tabs design pattern:
//! - Support for `Automatic` vs `Manual` tab activation.
//! - Directional keyboard arrow navigation (horizontal and vertical).
//! - Automatic skipping of disabled tabs.
//! - Strict ARIA attributes (`role="tablist"`, `role="tab"`, `role="tabpanel"`,
//!   `aria-selected`, `aria-controls`, `aria-labelledby`, `tabindex`).

use crate::attributes::{AriaAttributes, AriaOrientation, AriaRole};
use crate::fsm::{StateMachine, TransitionResult};

/// Identifier for a tab.
pub type TabId = String;

/// Selection and activation behavior for tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsActivationMode {
    /// Focusing a tab automatically activates and selects it.
    #[default]
    Automatic,
    /// Moving focus does not activate the tab until user presses Enter/Space or clicks.
    Manual,
}

/// Navigation direction for tab switching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabNavDirection {
    /// Move to next enabled tab.
    Next,
    /// Move to previous enabled tab.
    Previous,
    /// Jump to the first enabled tab.
    First,
    /// Jump to the last enabled tab.
    Last,
}

/// Descriptor of a single tab item in the tablist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    /// Unique tab identifier.
    pub id: TabId,
    /// Whether this tab is disabled.
    pub disabled: bool,
}

/// Events driving the tabs state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsEvent {
    /// Clicked on a tab.
    TabClicked(TabId),
    /// Arrow key navigation event.
    Navigate(TabNavDirection),
    /// Explicit activation request (e.g. Enter or Space in Manual mode).
    ActivateFocused,
    /// Keyboard focus entered tab.
    TabFocused(TabId),
}

/// Actions emitted on tab state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsAction {
    /// Active tab changed.
    TabSelected(TabId),
    /// Focused tab element changed (for roving focus).
    FocusTargetChanged(TabId),
}

/// Internal state of the Tabs FSM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsState {
    /// Currently selected and active tab.
    pub selected_tab: Option<TabId>,
    /// Currently focused tab (may differ in Manual mode).
    pub focused_tab: Option<TabId>,
}

/// Deterministic Tabs state machine.
#[derive(Debug, Clone)]
pub struct TabsFsm {
    items: Vec<TabItem>,
    state: TabsState,
    orientation: AriaOrientation,
    activation_mode: TabsActivationMode,
}

impl TabsFsm {
    /// Creates a new Tabs state machine.
    ///
    /// The first enabled tab is selected by default if available.
    pub fn new(items: Vec<TabItem>, orientation: AriaOrientation) -> Self {
        let first_enabled = items.iter().find(|i| !i.disabled).map(|i| i.id.clone());
        let initial_state = TabsState {
            selected_tab: first_enabled.clone(),
            focused_tab: first_enabled,
        };

        Self {
            items,
            state: initial_state,
            orientation,
            activation_mode: TabsActivationMode::Automatic,
        }
    }

    /// Sets the tab activation mode.
    #[must_use]
    pub fn with_activation_mode(mut self, mode: TabsActivationMode) -> Self {
        self.activation_mode = mode;
        self
    }

    /// Returns the currently active tab ID.
    pub fn selected_tab(&self) -> Option<&str> {
        self.state.selected_tab.as_deref()
    }

    /// Returns the currently focused tab ID.
    pub fn focused_tab(&self) -> Option<&str> {
        self.state.focused_tab.as_deref()
    }

    /// Returns the tablist orientation.
    pub fn orientation(&self) -> AriaOrientation {
        self.orientation
    }

    /// Generates ARIA attributes for the `tablist` container.
    pub fn tablist_aria_attributes(&self) -> AriaAttributes {
        AriaAttributes::new()
            .with_role(AriaRole::TabList)
            .with_orientation(self.orientation)
    }

    /// Generates ARIA attributes for a specific `tab` button.
    pub fn tab_aria_attributes(&self, tab_id: &str) -> AriaAttributes {
        let is_selected = self.state.selected_tab.as_deref() == Some(tab_id);
        let is_disabled = self
            .items
            .iter()
            .find(|i| i.id == tab_id)
            .is_some_and(|i| i.disabled);

        let tabindex = if is_selected { 0 } else { -1 };

        let mut attrs = AriaAttributes::new()
            .with_role(AriaRole::Tab)
            .with_selected(is_selected)
            .with_controls(format!("panel-{tab_id}"))
            .with_tabindex(tabindex);

        if is_disabled {
            attrs = attrs.with_disabled(true);
        }

        attrs
    }

    /// Generates ARIA attributes for a `tabpanel`.
    pub fn tabpanel_aria_attributes(&self, tab_id: &str) -> AriaAttributes {
        let is_active = self.state.selected_tab.as_deref() == Some(tab_id);

        let mut attrs = AriaAttributes::new()
            .with_role(AriaRole::TabPanel)
            .with_labelledby(format!("tab-{tab_id}"))
            .with_tabindex(0);

        if !is_active {
            attrs = attrs.with_hidden(true);
        }

        attrs
    }

    fn find_next_enabled_index(&self, current: usize, direction: TabNavDirection) -> Option<usize> {
        let len = self.items.len();
        if len == 0 {
            return None;
        }

        match direction {
            TabNavDirection::First => self.items.iter().position(|i| !i.disabled),
            TabNavDirection::Last => self.items.iter().rposition(|i| !i.disabled),
            TabNavDirection::Next => {
                for offset in 1..len {
                    let idx = (current + offset) % len;
                    if !self.items[idx].disabled {
                        return Some(idx);
                    }
                }
                None
            }
            TabNavDirection::Previous => {
                for offset in 1..len {
                    let idx = (current + len - offset) % len;
                    if !self.items[idx].disabled {
                        return Some(idx);
                    }
                }
                None
            }
        }
    }
}

impl StateMachine for TabsFsm {
    type State = TabsState;
    type Event = TabsEvent;
    type Action = TabsAction;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn transition(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let mut actions = Vec::new();

        match event {
            TabsEvent::TabClicked(tab_id) => {
                let is_valid = self.items.iter().any(|i| i.id == tab_id && !i.disabled);

                if is_valid {
                    self.state.selected_tab = Some(tab_id.clone());
                    self.state.focused_tab = Some(tab_id.clone());
                    actions.push(TabsAction::TabSelected(tab_id.clone()));
                    actions.push(TabsAction::FocusTargetChanged(tab_id));
                }
            }

            TabsEvent::TabFocused(tab_id) => {
                let is_valid = self.items.iter().any(|i| i.id == tab_id && !i.disabled);

                if is_valid {
                    self.state.focused_tab = Some(tab_id.clone());
                    actions.push(TabsAction::FocusTargetChanged(tab_id.clone()));

                    if self.activation_mode == TabsActivationMode::Automatic {
                        self.state.selected_tab = Some(tab_id.clone());
                        actions.push(TabsAction::TabSelected(tab_id));
                    }
                }
            }

            TabsEvent::ActivateFocused => {
                if let Some(focused) = &self.state.focused_tab {
                    let is_valid = self.items.iter().any(|i| &i.id == focused && !i.disabled);

                    if is_valid && self.state.selected_tab.as_ref() != Some(focused) {
                        self.state.selected_tab = Some(focused.clone());
                        actions.push(TabsAction::TabSelected(focused.clone()));
                    }
                }
            }

            TabsEvent::Navigate(dir) => {
                let current_idx = self
                    .state
                    .focused_tab
                    .as_ref()
                    .and_then(|id| self.items.iter().position(|i| &i.id == id))
                    .unwrap_or(0);

                if let Some(next_idx) = self.find_next_enabled_index(current_idx, dir) {
                    let next_tab = self.items[next_idx].id.clone();
                    self.state.focused_tab = Some(next_tab.clone());
                    actions.push(TabsAction::FocusTargetChanged(next_tab.clone()));

                    if self.activation_mode == TabsActivationMode::Automatic {
                        self.state.selected_tab = Some(next_tab.clone());
                        actions.push(TabsAction::TabSelected(next_tab));
                    }
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
