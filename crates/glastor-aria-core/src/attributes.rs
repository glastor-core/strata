//! Strict type-safe representations of WAI-ARIA 1.2 attributes and roles.
//!
//! Enforces WCAG compliance at compile-time instead of error-prone string manipulation.

use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// WAI-ARIA 1.2 Widget and Landmark Roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AriaRole {
    /// Dialog modal or non-modal.
    Dialog,
    /// Alert dialog requiring immediate user attention.
    AlertDialog,
    /// Interactive button.
    Button,
    /// Checkbox widget.
    Checkbox,
    /// Combobox composite widget.
    Combobox,
    /// Menu presentation container.
    Menu,
    /// Single menu item within a menu.
    MenuItem,
    /// Radio button within a radiogroup.
    Radio,
    /// Group of radio buttons.
    RadioGroup,
    /// Tab list container.
    TabList,
    /// Individual tab toggle.
    Tab,
    /// Content panel associated with a tab.
    TabPanel,
    /// Toolbar container for action buttons.
    Toolbar,
    /// Tooltip overlay element.
    Tooltip,
    /// Status live region notification.
    Status,
    /// Alert live region notification.
    Alert,
    /// Interactive grid or spreadsheet table container.
    Grid,
    /// Cell within a grid or spreadsheet table.
    Gridcell,
}

impl AriaRole {
    /// Returns the standard WAI-ARIA role attribute string value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::AlertDialog => "alertdialog",
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::Combobox => "combobox",
            Self::Menu => "menu",
            Self::MenuItem => "menuitem",
            Self::Radio => "radio",
            Self::RadioGroup => "radiogroup",
            Self::TabList => "tablist",
            Self::Tab => "tab",
            Self::TabPanel => "tabpanel",
            Self::Toolbar => "toolbar",
            Self::Tooltip => "tooltip",
            Self::Status => "status",
            Self::Alert => "alert",
            Self::Grid => "grid",
            Self::Gridcell => "gridcell",
        }
    }
}

impl fmt::Display for AriaRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents the `aria-haspopup` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AriaHasPopup {
    /// No popup.
    False,
    /// True or generic menu popup.
    Menu,
    /// Listbox popup.
    Listbox,
    /// Tree hierarchy popup.
    Tree,
    /// Grid popup.
    Grid,
    /// Dialog modal popup.
    Dialog,
}

impl AriaHasPopup {
    /// Returns the attribute value string according to WAI-ARIA 1.2.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::False => "false",
            Self::Menu => "menu",
            Self::Listbox => "listbox",
            Self::Tree => "tree",
            Self::Grid => "grid",
            Self::Dialog => "dialog",
        }
    }
}

/// Represents the `aria-orientation` property for composite widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AriaOrientation {
    /// Horizontal navigation orientation (e.g. standard tab list or toolbar).
    #[default]
    Horizontal,
    /// Vertical navigation orientation (e.g. dropdown menu or vertical tabs).
    Vertical,
}

impl AriaOrientation {
    /// Returns the attribute string value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Collection of resolved ARIA HTML attributes for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AriaAttributes {
    /// Optional role attribute.
    pub role: Option<AriaRole>,
    /// aria-expanded ("true" | "false")
    pub expanded: Option<bool>,
    /// aria-haspopup
    pub has_popup: Option<AriaHasPopup>,
    /// aria-controls id reference
    pub controls: Option<String>,
    /// aria-labelledby id reference
    pub labelledby: Option<String>,
    /// aria-describedby id reference
    pub describedby: Option<String>,
    /// aria-modal boolean flag
    pub modal: Option<bool>,
    /// aria-hidden boolean flag
    pub hidden: Option<bool>,
    /// aria-orientation
    pub orientation: Option<AriaOrientation>,
    /// aria-selected ("true" | "false")
    pub selected: Option<bool>,
    /// aria-disabled boolean flag
    pub disabled: Option<bool>,
    /// aria-atomic boolean flag
    pub atomic: Option<bool>,
    /// aria-live ("polite" | "assertive" | "off")
    pub live: Option<String>,
    /// tabindex attribute ("0" or "-1")
    pub tabindex: Option<i8>,
    /// aria-rowcount for data grids
    pub rowcount: Option<u32>,
    /// aria-colcount for data grids
    pub colcount: Option<u32>,
    /// aria-rowindex for data grid cells
    pub rowindex: Option<u32>,
    /// aria-colindex for data grid cells
    pub colindex: Option<u32>,
}

impl AriaAttributes {
    /// Creates a blank set of attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `role` attribute.
    pub fn with_role(mut self, role: AriaRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the `aria-expanded` attribute.
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Sets the `aria-selected` attribute.
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Sets the `aria-disabled` attribute.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Sets the `aria-atomic` attribute.
    pub fn with_atomic(mut self, atomic: bool) -> Self {
        self.atomic = Some(atomic);
        self
    }

    /// Sets the `aria-live` attribute.
    pub fn with_live(mut self, live: impl Into<String>) -> Self {
        self.live = Some(live.into());
        self
    }

    /// Sets the `aria-orientation` attribute.
    pub fn with_orientation(mut self, orientation: AriaOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Sets the `aria-haspopup` attribute.
    pub fn with_has_popup(mut self, popup: AriaHasPopup) -> Self {
        self.has_popup = Some(popup);
        self
    }

    /// Sets the `aria-controls` attribute referencing an element ID.
    pub fn with_controls(mut self, id: impl Into<String>) -> Self {
        self.controls = Some(id.into());
        self
    }

    /// Sets the `aria-labelledby` attribute.
    pub fn with_labelledby(mut self, id: impl Into<String>) -> Self {
        self.labelledby = Some(id.into());
        self
    }

    /// Sets the `aria-describedby` attribute.
    pub fn with_describedby(mut self, id: impl Into<String>) -> Self {
        self.describedby = Some(id.into());
        self
    }

    /// Sets the `aria-hidden` attribute.
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Sets the `aria-modal` attribute.
    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets the `tabindex` attribute.
    pub fn with_tabindex(mut self, index: i8) -> Self {
        self.tabindex = Some(index);
        self
    }

    /// Sets the `aria-rowcount` attribute.
    pub fn with_rowcount(mut self, count: u32) -> Self {
        self.rowcount = Some(count);
        self
    }

    /// Sets the `aria-colcount` attribute.
    pub fn with_colcount(mut self, count: u32) -> Self {
        self.colcount = Some(count);
        self
    }

    /// Sets the `aria-rowindex` attribute.
    pub fn with_rowindex(mut self, index: u32) -> Self {
        self.rowindex = Some(index);
        self
    }

    /// Sets the `aria-colindex` attribute.
    pub fn with_colindex(mut self, index: u32) -> Self {
        self.colindex = Some(index);
        self
    }

    /// Converts the collection of attributes to standard key-value string pairs.
    pub fn to_key_values(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::with_capacity(8);

        if let Some(role) = self.role {
            pairs.push(("role", role.as_str().to_string()));
        }
        if let Some(exp) = self.expanded {
            pairs.push(("aria-expanded", exp.to_string()));
        }
        if let Some(popup) = self.has_popup {
            pairs.push(("aria-haspopup", popup.as_str().to_string()));
        }
        if let Some(ref controls) = self.controls {
            pairs.push(("aria-controls", controls.clone()));
        }
        if let Some(ref labelledby) = self.labelledby {
            pairs.push(("aria-labelledby", labelledby.clone()));
        }
        if let Some(ref describedby) = self.describedby {
            pairs.push(("aria-describedby", describedby.clone()));
        }
        if let Some(modal) = self.modal {
            pairs.push(("aria-modal", modal.to_string()));
        }
        if let Some(hidden) = self.hidden {
            pairs.push(("aria-hidden", hidden.to_string()));
        }
        if let Some(selected) = self.selected {
            pairs.push(("aria-selected", selected.to_string()));
        }
        if let Some(disabled) = self.disabled {
            pairs.push(("aria-disabled", disabled.to_string()));
        }
        if let Some(atomic) = self.atomic {
            pairs.push(("aria-atomic", atomic.to_string()));
        }
        if let Some(ref live) = self.live {
            pairs.push(("aria-live", live.clone()));
        }
        if let Some(orientation) = self.orientation {
            pairs.push(("aria-orientation", orientation.as_str().to_string()));
        }
        if let Some(idx) = self.tabindex {
            pairs.push(("tabindex", idx.to_string()));
        }
        if let Some(rc) = self.rowcount {
            pairs.push(("aria-rowcount", rc.to_string()));
        }
        if let Some(cc) = self.colcount {
            pairs.push(("aria-colcount", cc.to_string()));
        }
        if let Some(ri) = self.rowindex {
            pairs.push(("aria-rowindex", ri.to_string()));
        }
        if let Some(ci) = self.colindex {
            pairs.push(("aria-colindex", ci.to_string()));
        }

        pairs
    }
}
