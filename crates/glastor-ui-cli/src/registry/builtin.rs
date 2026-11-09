//! Built-in canonical component registry.
//!
//! Provides production-ready, framework-agnostic accessible components
//! combining `glastor-aria-core` FSMs and `glastor-css-binder` utility styles.

use crate::registry::schema::{
    ComponentFile, DependencyRequirement, RegistryComponent, RegistryManifest,
};
use std::collections::BTreeMap;

/// Generates the canonical built-in component manifest.
pub fn get_canonical_manifest() -> RegistryManifest {
    let mut components = BTreeMap::new();

    // 1. Button component
    components.insert(
        "button".to_string(),
        RegistryComponent {
            name: "button".to_string(),
            description: "Accessible interactive button with focus ring, variants and loading states.".to_string(),
            frameworks: vec!["leptos".to_string(), "dioxus".to_string(), "agnostic".to_string()],
            dependencies: vec![
                DependencyRequirement {
                    name: "glastor-aria-core".to_string(),
                    version: "0.1.0".to_string(),
                    features: vec!["std".to_string()],
                },
            ],
            files: vec![
                ComponentFile {
                    path: "button.rs".to_string(),
                    content: r#"//! Accessible Button Component.
use glastor_aria_core::attributes::{AriaAttributes, AriaRole};

/// Visual button variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
}

impl ButtonVariant {
    pub fn class_names(&self) -> &'static str {
        match self {
            Self::Primary => "bg-primary text-primary-foreground hover:bg-slate-800",
            Self::Secondary => "bg-muted text-muted-foreground hover:bg-slate-100",
            Self::Outline => "border border-border text-foreground hover:bg-muted",
            Self::Ghost => "bg-transparent text-foreground hover:bg-muted",
        }
    }
}

/// Generates button classes and strict ARIA attributes.
pub fn button_props(variant: ButtonVariant, disabled: bool) -> (&'static str, AriaAttributes) {
    let base_classes = "inline-flex items-center justify-center rounded-md font-medium transition-all px-4 py-2 focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed";
    let attrs = AriaAttributes::new()
        .with_role(AriaRole::Button)
        .with_tabindex(if disabled { -1 } else { 0 });
    (base_classes, attrs)
}
"#.to_string(),
                },
            ],
        },
    );

    // 2. Dialog / Modal component
    components.insert(
        "dialog".to_string(),
        RegistryComponent {
            name: "dialog".to_string(),
            description: "WAI-ARIA 1.2 modal dialog with automatic FocusTrap and DismissableLayer.".to_string(),
            frameworks: vec!["leptos".to_string(), "dioxus".to_string(), "agnostic".to_string()],
            dependencies: vec![
                DependencyRequirement {
                    name: "glastor-aria-core".to_string(),
                    version: "0.1.0".to_string(),
                    features: vec!["std".to_string()],
                },
            ],
            files: vec![
                ComponentFile {
                    path: "dialog.rs".to_string(),
                    content: r#"//! Accessible Dialog Modal Component.
pub use glastor_aria_core::primitives::dialog::{
    DialogAction, DialogEvent, DialogFsm, DialogState,
};

/// Pre-configured styling classes for Dialog overlay and content.
pub struct DialogStyles;

impl DialogStyles {
    pub const OVERLAY: &'static str = "fixed inset-0 bg-black bg-opacity-50 transition-all";
    pub const CONTAINER: &'static str = "fixed inset-0 flex items-center justify-center p-4";
    pub const CONTENT: &'static str = "bg-background text-foreground rounded-lg border border-border p-6 shadow-xl w-full max-w-md";
    pub const TITLE: &'static str = "text-xl font-semibold text-foreground";
    pub const DESCRIPTION: &'static str = "text-sm text-muted-foreground mt-2";
}
"#.to_string(),
                },
            ],
        },
    );

    // 3. Accordion component
    components.insert(
        "accordion".to_string(),
        RegistryComponent {
            name: "accordion".to_string(),
            description: "WCAG 2.2 Collapsible accordion with arrow keyboard navigation.".to_string(),
            frameworks: vec!["leptos".to_string(), "dioxus".to_string(), "agnostic".to_string()],
            dependencies: vec![
                DependencyRequirement {
                    name: "glastor-aria-core".to_string(),
                    version: "0.1.0".to_string(),
                    features: vec!["std".to_string()],
                },
            ],
            files: vec![
                ComponentFile {
                    path: "accordion.rs".to_string(),
                    content: r#"//! Accessible Accordion Component.
pub use glastor_aria_core::primitives::accordion::{
    AccordionAction, AccordionEvent, AccordionFsm, AccordionItemId, AccordionMode,
};

pub struct AccordionStyles;

impl AccordionStyles {
    pub const CONTAINER: &'static str = "w-full border border-border rounded-md";
    pub const HEADER: &'static str = "flex items-center justify-between p-4 cursor-pointer hover:bg-muted font-medium";
    pub const PANEL: &'static str = "p-4 text-sm text-muted-foreground border-t border-border";
}
"#.to_string(),
                },
            ],
        },
    );

    // 4. Popover / Tooltip component
    components.insert(
        "popover".to_string(),
        RegistryComponent {
            name: "popover".to_string(),
            description: "Accessible anchored floating popover / tooltip with escape and outside-click dismissal.".to_string(),
            frameworks: vec!["leptos".to_string(), "dioxus".to_string(), "agnostic".to_string()],
            dependencies: vec![
                DependencyRequirement {
                    name: "glastor-aria-core".to_string(),
                    version: "0.1.0".to_string(),
                    features: vec!["std".to_string()],
                },
            ],
            files: vec![
                ComponentFile {
                    path: "popover.rs".to_string(),
                    content: r#"//! Accessible Popover & Tooltip Component.
pub use glastor_aria_core::primitives::popover::{
    PopoverAction, PopoverEvent, PopoverFsm, PopoverState, PopoverTrigger,
};

pub struct PopoverStyles;

impl PopoverStyles {
    pub const TRIGGER: &'static str = "inline-flex items-center justify-center rounded-md font-medium px-4 py-2 hover:bg-muted";
    pub const CONTENT: &'static str = "z-50 bg-background text-foreground rounded-md border border-border p-4 shadow-md outline-none w-72";
    pub const TOOLTIP: &'static str = "z-50 bg-primary text-primary-foreground text-xs rounded px-2 py-1 shadow-sm";
}
"#.to_string(),
                },
            ],
        },
    );

    RegistryManifest {
        version: "1.0.0".to_string(),
        name: "glastor-core-canonical".to_string(),
        description: "Official GLASTOR Core accessible UI component registry".to_string(),
        components,
    }
}
