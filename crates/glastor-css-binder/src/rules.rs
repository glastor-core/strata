//! Utility CSS rules and design tokens generation engine.
//!
//! Transforms scanned utility class tokens into valid, standards-compliant CSS rules
//! with zero external Node.js dependencies. Supports pseudo-class variants
//! (`hover:`, `focus:`, `focus-visible:`, `dark:`, `active:`, `disabled:`).

use std::collections::BTreeMap;
use std::fmt::Write;

/// Predefined theme tokens compatible with modern design systems (Radix / shadcn).
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    /// Base border radius (e.g. "0.5rem").
    pub radius: String,
    /// Background color in light mode.
    pub background: String,
    /// Foreground text color in light mode.
    pub foreground: String,
    /// Primary brand color.
    pub primary: String,
    /// Primary contrast foreground text color.
    pub primary_foreground: String,
    /// Muted background.
    pub muted: String,
    /// Muted foreground text.
    pub muted_foreground: String,
    /// Border color.
    pub border: String,
    /// Focus ring color.
    pub ring: String,
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self {
            radius: "0.5rem".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#09090b".to_string(),
            primary: "#18181b".to_string(),
            primary_foreground: "#fafafa".to_string(),
            muted: "#f4f4f5".to_string(),
            muted_foreground: "#71717a".to_string(),
            border: "#e4e4e7".to_string(),
            ring: "#18181b".to_string(),
        }
    }
}

impl ThemeTokens {
    /// Generates root CSS variables block.
    pub fn to_css(&self) -> String {
        let mut css = String::new();
        let _ = writeln!(css, ":root {{");
        let _ = writeln!(css, "  --radius: {};", self.radius);
        let _ = writeln!(css, "  --background: {};", self.background);
        let _ = writeln!(css, "  --foreground: {};", self.foreground);
        let _ = writeln!(css, "  --primary: {};", self.primary);
        let _ = writeln!(css, "  --primary-foreground: {};", self.primary_foreground);
        let _ = writeln!(css, "  --muted: {};", self.muted);
        let _ = writeln!(css, "  --muted-foreground: {};", self.muted_foreground);
        let _ = writeln!(css, "  --border: {};", self.border);
        let _ = writeln!(css, "  --ring: {};", self.ring);
        let _ = writeln!(css, "}}");
        css
    }
}

/// Generates valid CSS stylesheets from extracted utility classes.
#[derive(Debug, Clone, Default)]
pub struct RulesEngine;

impl RulesEngine {
    /// Creates a new rules generator.
    pub fn new() -> Self {
        Self
    }

    /// Escapes a class name for use in a CSS selector (e.g. `w-[200px]` -> `w-\[200px\]`).
    pub fn escape_selector(class_name: &str) -> String {
        let mut escaped = String::with_capacity(class_name.len() + 4);
        for ch in class_name.chars() {
            match ch {
                ':' | '/' | '[' | ']' | '#' | '.' | '%' => {
                    escaped.push('\\');
                    escaped.push(ch);
                }
                _ => escaped.push(ch),
            }
        }
        escaped
    }

    /// Resolves a class (including variant prefixes like `hover:`, `focus:`, `dark:`)
    /// into a tuple of `(css_selector, declaration_body)`.
    pub fn resolve_entry(class: &str) -> Option<(String, String)> {
        // Check known variant prefixes
        if let Some(base) = class.strip_prefix("hover:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".{}:hover", Self::escape_selector(class));
            return Some((selector, decl));
        }

        if let Some(base) = class.strip_prefix("focus:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".{}:focus", Self::escape_selector(class));
            return Some((selector, decl));
        }

        if let Some(base) = class.strip_prefix("focus-visible:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".{}:focus-visible", Self::escape_selector(class));
            return Some((selector, decl));
        }

        if let Some(base) = class.strip_prefix("active:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".{}:active", Self::escape_selector(class));
            return Some((selector, decl));
        }

        if let Some(base) = class.strip_prefix("disabled:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".{}:disabled", Self::escape_selector(class));
            return Some((selector, decl));
        }

        if let Some(base) = class.strip_prefix("dark:") {
            let decl = Self::resolve_base_rule(base)?;
            let selector = format!(".dark .{}", Self::escape_selector(class));
            return Some((selector, decl));
        }

        // Standard un-prefixed utility
        let decl = Self::resolve_base_rule(class)?;
        let selector = format!(".{}", Self::escape_selector(class));
        Some((selector, decl))
    }

    /// Evaluates a base utility class without variant prefixes.
    #[allow(clippy::too_many_lines)]
    pub fn resolve_base_rule(class: &str) -> Option<String> {
        // 1. Layout & Display
        match class {
            "flex" => return Some("display: flex;".into()),
            "inline-flex" => return Some("display: inline-flex;".into()),
            "grid" => return Some("display: grid;".into()),
            "block" => return Some("display: block;".into()),
            "inline-block" => return Some("display: inline-block;".into()),
            "hidden" => return Some("display: none;".into()),

            // Flexbox alignments
            "flex-col" => return Some("flex-direction: column;".into()),
            "flex-row" => return Some("flex-direction: row;".into()),
            "items-center" => return Some("align-items: center;".into()),
            "items-start" => return Some("align-items: flex-start;".into()),
            "items-end" => return Some("align-items: flex-end;".into()),
            "justify-center" => return Some("justify-content: center;".into()),
            "justify-between" => return Some("justify-content: space-between;".into()),
            "justify-start" => return Some("justify-content: flex-start;".into()),
            "justify-end" => return Some("justify-content: flex-end;".into()),

            // Positioning
            "relative" => return Some("position: relative;".into()),
            "absolute" => return Some("position: absolute;".into()),
            "fixed" => return Some("position: fixed;".into()),
            "sticky" => return Some("position: sticky;".into()),
            "inset-0" => return Some("top: 0; right: 0; bottom: 0; left: 0;".into()),

            // Sizing
            "w-full" => return Some("width: 100%;".into()),
            "w-screen" => return Some("width: 100vw;".into()),
            "w-auto" => return Some("width: auto;".into()),
            "h-full" => return Some("height: 100%;".into()),
            "h-screen" => return Some("height: 100vh;".into()),
            "min-h-screen" => return Some("min-height: 100vh;".into()),

            // Text alignments & Weights
            "text-center" => return Some("text-align: center;".into()),
            "text-left" => return Some("text-align: left;".into()),
            "text-right" => return Some("text-align: right;".into()),
            "font-normal" => return Some("font-weight: 400;".into()),
            "font-medium" => return Some("font-weight: 500;".into()),
            "font-semibold" => return Some("font-weight: 600;".into()),
            "font-bold" => return Some("font-weight: 700;".into()),

            // Outline & Focus ring
            "outline-none" => {
                return Some("outline: 2px solid transparent; outline-offset: 2px;".into())
            }
            "ring-2" => {
                return Some(
                    "box-shadow: 0 0 0 2px var(--ring, rgba(59, 130, 246, 0.5));".into(),
                )
            }

            // Borders & Rounded
            "border" => return Some("border-width: 1px; border-style: solid;".into()),
            "border-0" => return Some("border-width: 0px;".into()),
            "border-2" => return Some("border-width: 2px; border-style: solid;".into()),
            "border-b" => return Some("border-bottom-width: 1px; border-bottom-style: solid;".into()),
            "border-t" => return Some("border-top-width: 1px; border-top-style: solid;".into()),
            "rounded-none" => return Some("border-radius: 0px;".into()),
            "rounded-sm" => return Some("border-radius: 0.125rem;".into()),
            "rounded" => return Some("border-radius: 0.25rem;".into()),
            "rounded-md" => return Some("border-radius: 0.375rem;".into()),
            "rounded-lg" => return Some("border-radius: 0.5rem;".into()),
            "rounded-xl" => return Some("border-radius: 0.75rem;".into()),
            "rounded-2xl" => return Some("border-radius: 1rem;".into()),
            "rounded-full" => return Some("border-radius: 9999px;".into()),

            // Typography Scales
            "text-xs" => return Some("font-size: 0.75rem; line-height: 1rem;".into()),
            "text-sm" => return Some("font-size: 0.875rem; line-height: 1.25rem;".into()),
            "text-base" => return Some("font-size: 1rem; line-height: 1.5rem;".into()),
            "text-lg" => return Some("font-size: 1.125rem; line-height: 1.75rem;".into()),
            "text-xl" => return Some("font-size: 1.25rem; line-height: 1.75rem;".into()),
            "text-2xl" => return Some("font-size: 1.5rem; line-height: 2rem;".into()),
            "text-3xl" => return Some("font-size: 1.875rem; line-height: 2.25rem;".into()),
            "text-5xl" => return Some("font-size: 3rem; line-height: 1;".into()),
            "leading-tight" => return Some("line-height: 1.25;".into()),
            "leading-relaxed" => return Some("line-height: 1.625;".into()),
            "tracking-tight" => return Some("letter-spacing: -0.025em;".into()),
            "tracking-wider" => return Some("letter-spacing: 0.05em;".into()),
            "font-mono" => return Some("font-family: ui-monospace, monospace;".into()),
            "font-sans" => return Some("font-family: ui-sans-serif, system-ui, sans-serif;".into()),
            "font-extrabold" => return Some("font-weight: 800;".into()),
            "font-black" => return Some("font-weight: 900;".into()),

            // Layout & Overflow
            "max-w-6xl" => return Some("max-width: 72rem;".into()),
            "max-w-3xl" => return Some("max-width: 48rem;".into()),
            "max-w-md" => return Some("max-width: 28rem;".into()),
            "overflow-hidden" => return Some("overflow: hidden;".into()),
            "overflow-y-auto" => return Some("overflow-y: auto;".into()),
            "overflow-x-auto" => return Some("overflow-x: auto;".into()),
            "z-50" => return Some("z-index: 50;".into()),
            "backdrop-blur-sm" => return Some("backdrop-filter: blur(4px);".into()),

            // Shadows
            "shadow-sm" => return Some("box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);".into()),
            "shadow-md" => return Some("box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);".into()),
            "shadow-lg" => return Some("box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);".into()),
            "shadow-xl" => return Some("box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);".into()),
            "shadow-2xl" => return Some("box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);".into()),

            // CSS Grid
            "grid-cols-1" => return Some("grid-template-columns: repeat(1, minmax(0, 1fr));".into()),
            "grid-cols-2" => return Some("grid-template-columns: repeat(2, minmax(0, 1fr));".into()),
            "grid-cols-3" => return Some("grid-template-columns: repeat(3, minmax(0, 1fr));".into()),
            "grid-cols-4" => return Some("grid-template-columns: repeat(4, minmax(0, 1fr));".into()),

            // Theme Variables mapping
            "bg-background" => return Some("background-color: var(--background);".into()),
            "text-foreground" => return Some("color: var(--foreground);".into()),
            "bg-primary" => return Some("background-color: var(--primary);".into()),
            "text-primary-foreground" => {
                return Some("color: var(--primary-foreground);".into())
            }
            "bg-muted" => return Some("background-color: var(--muted);".into()),
            "text-muted-foreground" => return Some("color: var(--muted-foreground);".into()),
            "border-border" => return Some("border-color: var(--border);".into()),

            // Common Theme Colors
            "bg-transparent" => return Some("background-color: transparent;".into()),
            "bg-white" => return Some("background-color: #ffffff;".into()),
            "bg-black" => return Some("background-color: #000000;".into()),
            "bg-black/60" => return Some("background-color: rgba(0, 0, 0, 0.6);".into()),
            "bg-slate-900" => return Some("background-color: #0f172a;".into()),
            "bg-slate-800" => return Some("background-color: #1e293b;".into()),
            "bg-slate-100" => return Some("background-color: #f1f5f9;".into()),
            "text-white" => return Some("color: #ffffff;".into()),
            "text-black" => return Some("color: #000000;".into()),
            "text-slate-900" => return Some("color: #0f172a;".into()),
            "text-slate-500" => return Some("color: #64748b;".into()),

            // Cursor & Transition
            "cursor-pointer" => return Some("cursor: pointer;".into()),
            "cursor-not-allowed" => return Some("cursor: not-allowed;".into()),
            "transition-all" => return Some(
                "transition-property: all; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;".into()
            ),

            _ => {}
        }

        // 2. Parametric Spacing & Sizing utilities (p-*, m-*, gap-*)
        if let Some(rest) = class.strip_prefix("p-") {
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("padding: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("px-") {
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("padding-left: {val}; padding-right: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("py-") {
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("padding-top: {val}; padding-bottom: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("m-") {
            if rest == "auto" {
                return Some("margin: auto;".into());
            }
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("margin: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("mx-") {
            if rest == "auto" {
                return Some("margin-left: auto; margin-right: auto;".into());
            }
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("margin-left: {val}; margin-right: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("my-") {
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("margin-top: {val}; margin-bottom: {val};"));
            }
        }
        if let Some(rest) = class.strip_prefix("gap-") {
            if let Some(val) = Self::resolve_spacing_scale(rest) {
                return Some(format!("gap: {val};"));
            }
        }

        // 3. Arbitrary values (e.g. w-[300px], bg-[#fff], border-[#27272a])
        if let Some((prefix, arbitrary)) = Self::parse_arbitrary(class) {
            match prefix {
                "w" => return Some(format!("width: {arbitrary};")),
                "h" => return Some(format!("height: {arbitrary};")),
                "bg" => return Some(format!("background-color: {arbitrary};")),
                "text" => return Some(format!("color: {arbitrary};")),
                "p" => return Some(format!("padding: {arbitrary};")),
                "m" => return Some(format!("margin: {arbitrary};")),
                "gap" => return Some(format!("gap: {arbitrary};")),
                "z" => return Some(format!("z-index: {arbitrary};")),
                "border" => {
                    return Some(format!(
                        "border-color: {arbitrary}; border-width: 1px; border-style: solid;"
                    ));
                }
                _ => {}
            }
        }

        None
    }

    fn resolve_spacing_scale(key: &str) -> Option<&'static str> {
        match key {
            "0" => Some("0px"),
            "1" => Some("0.25rem"),
            "2" => Some("0.5rem"),
            "3" => Some("0.75rem"),
            "4" => Some("1rem"),
            "5" => Some("1.25rem"),
            "6" => Some("1.5rem"),
            "8" => Some("2rem"),
            "10" => Some("2.5rem"),
            "12" => Some("3rem"),
            "16" => Some("4rem"),
            "20" => Some("5rem"),
            _ => None,
        }
    }

    fn parse_arbitrary(class: &str) -> Option<(&str, &str)> {
        if let Some(open) = class.find("-[") {
            if class.ends_with(']') {
                let prefix = &class[..open];
                let arbitrary = &class[open + 2..class.len() - 1];

                // AppSec Sanitization: Reject rule termination and dangerous CSS injection vectors
                let lower = arbitrary.to_ascii_lowercase();
                if arbitrary.contains(';')
                    || arbitrary.contains('{')
                    || arbitrary.contains('}')
                    || arbitrary.contains('\\')
                    || arbitrary.contains('@')
                    || lower.contains("url(")
                    || lower.contains("expression(")
                    || lower.contains("javascript:")
                {
                    return None;
                }

                return Some((prefix, arbitrary));
            }
        }
        None
    }

    /// Compiles a set of class tokens into formatted CSS rules grouped in deterministic order.
    pub fn compile_stylesheet(&self, classes: &[String]) -> String {
        let mut sorted_rules: BTreeMap<String, String> = BTreeMap::new();

        for class in classes {
            if let Some((selector, decl)) = Self::resolve_entry(class) {
                sorted_rules.insert(selector, decl);
            }
        }

        let mut output = String::with_capacity(sorted_rules.len() * 48);
        output.push_str("/* Generated by glastor-css-binder (Zero-Node) */\n");

        for (selector, decl) in sorted_rules {
            let _ = writeln!(output, "{selector} {{ {decl} }}");
        }

        output
    }
}
