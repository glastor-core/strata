//! W3C Design Token Community Group (DTCG) Synchronizer.
//!
//! Parses standard design token JSON files exported from Figma, Penpot, or Style Dictionary,
//! generating both synchronized CSS `:root` variables and strongly-typed Rust constants.

use std::collections::BTreeMap;
use std::fmt::Write;

/// Parsed design token with its hierarchical path and string value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedToken {
    /// Token category path, e.g. `['color', 'primary']` or `['radius', 'base']`.
    pub path: Vec<String>,
    /// Resolved value as a string (e.g. "#18181b", "0.5rem", "16px").
    pub value: String,
    /// Token type (e.g. "color", "dimension", "fontFamily").
    pub token_type: Option<String>,
}

/// Synchronizer for W3C Design Tokens.
#[derive(Debug, Default)]
pub struct TokensSynchronizer {
    tokens: Vec<ResolvedToken>,
}

impl TokensSynchronizer {
    /// Creates a new synchronizer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates that a CSS token value is free from rule-breaking or exfiltration vectors.
    pub fn is_safe_css_value(value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        !value.contains(';')
            && !value.contains('{')
            && !value.contains('}')
            && !value.contains('\\')
            && !value.contains('@')
            && !value.contains('\n')
            && !value.contains('\r')
            && !lower.contains("url(")
            && !lower.contains("expression(")
            && !lower.contains("javascript:")
    }

    /// Sanitizes an arbitrary string into a valid, safe CSS custom property segment (`[a-zA-Z0-9_-]`).
    fn sanitize_css_segment(segment: &str) -> String {
        let mut out = String::with_capacity(segment.len());
        for ch in segment.chars() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                out.push(ch);
            } else {
                out.push('-');
            }
        }
        out.trim_matches('-').to_string()
    }

    /// Sanitizes an arbitrary key into a strict Rust identifier (`[A-Za-z0-9_]`), preventing code injection.
    pub fn sanitize_rust_ident(raw: &str, uppercase: bool) -> String {
        let mut ident = String::with_capacity(raw.len() + 2);
        let mut prev_underscore = false;

        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() {
                ident.push(if uppercase {
                    ch.to_ascii_uppercase()
                } else {
                    ch.to_ascii_lowercase()
                });
                prev_underscore = false;
            } else if !prev_underscore && !ident.is_empty() {
                ident.push('_');
                prev_underscore = true;
            }
        }

        let trimmed = ident.trim_matches('_').to_string();
        let mut final_ident = if trimmed.is_empty() {
            if uppercase {
                "TOKEN".to_string()
            } else {
                "tokens".to_string()
            }
        } else if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            format!("_{trimmed}")
        } else {
            trimmed
        };

        if matches!(
            final_ident.as_str(),
            "mod"
                | "type"
                | "fn"
                | "struct"
                | "enum"
                | "const"
                | "let"
                | "pub"
                | "use"
                | "crate"
                | "self"
                | "super"
                | "match"
                | "impl"
                | "trait"
        ) {
            final_ident.push('_');
        }

        final_ident
    }

    /// Parses a W3C design token JSON string.
    pub fn parse_json(&mut self, json_str: &str) -> Result<usize, String> {
        let root: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {e}"))?;

        let mut current_path = Vec::new();
        self.traverse(&root, &mut current_path);
        Ok(self.tokens.len())
    }

    fn traverse(&mut self, val: &serde_json::Value, path: &mut Vec<String>) {
        if let serde_json::Value::Object(map) = val {
            // Check if this object is a token definition (has "$value" or "value")
            let value_opt = map.get("$value").or_else(|| map.get("value"));
            if let Some(v) = value_opt {
                let token_val = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => v.to_string(),
                };
                if !Self::is_safe_css_value(&token_val) {
                    return;
                }

                let token_type = map
                    .get("$type")
                    .or_else(|| map.get("type"))
                    .and_then(|t| t.as_str())
                    .map(String::from);

                self.tokens.push(ResolvedToken {
                    path: path.clone(),
                    value: token_val,
                    token_type,
                });
                return;
            }

            // Otherwise, recursively traverse child groups
            for (key, child) in map {
                if key.starts_with('$') {
                    continue; // Skip group metadata like "$description"
                }
                path.push(key.clone());
                self.traverse(child, path);
                path.pop();
            }
        }
    }

    /// Generates CSS custom properties block (`:root { --category-name: value; }`).
    pub fn to_css(&self) -> String {
        let mut css = String::new();
        let _ = writeln!(css, "/* Generated by `cargo ui sync-tokens` (W3C DTCG) */");
        let _ = writeln!(css, ":root {{");

        for token in &self.tokens {
            if !Self::is_safe_css_value(&token.value) {
                continue;
            }
            let segments: Vec<String> = token
                .path
                .iter()
                .map(|s| Self::sanitize_css_segment(s))
                .filter(|s| !s.is_empty())
                .collect();
            if segments.is_empty() {
                continue;
            }
            let var_name = segments.join("-");
            let _ = writeln!(css, "  --{}: {};", var_name, token.value);
        }

        let _ = writeln!(css, "}}");
        css
    }

    /// Generates strongly-typed Rust constants module (`pub const PRIMARY: &str = "...";`).
    pub fn to_rust_code(&self) -> String {
        let mut code = String::new();
        let _ = writeln!(
            code,
            "//! Strongly-typed design tokens auto-generated by `cargo ui sync-tokens`.\n"
        );

        // Group by top-level category
        let mut groups: BTreeMap<String, Vec<&ResolvedToken>> = BTreeMap::new();
        for token in &self.tokens {
            let cat = token
                .path
                .first()
                .cloned()
                .unwrap_or_else(|| "general".to_string());
            groups.entry(cat).or_default().push(token);
        }

        for (group_name, list) in groups {
            let mod_ident = Self::sanitize_rust_ident(&group_name, false);
            let _ = writeln!(code, "pub mod {mod_ident} {{");
            for token in list {
                let raw_const = if token.path.len() > 1 {
                    token.path[1..].join("_")
                } else {
                    token
                        .path
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "VALUE".to_string())
                };
                let const_name = Self::sanitize_rust_ident(&raw_const, true);
                let _ = writeln!(
                    code,
                    "    pub const {}: &str = {:?};",
                    const_name, token.value
                );
            }
            let _ = writeln!(code, "}}\n");
        }

        code
    }

    /// Returns the parsed tokens.
    pub fn tokens(&self) -> &[ResolvedToken] {
        &self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::TokensSynchronizer;

    #[test]
    fn test_parse_w3c_design_tokens() {
        let json_data = r##"{
            "color": {
                "primary": {
                    "$value": "#18181b",
                    "$type": "color"
                },
                "muted": {
                    "$value": "#71717a",
                    "$type": "color"
                }
            },
            "radius": {
                "sm": {
                    "$value": "0.25rem",
                    "$type": "dimension"
                }
            }
        }"##;

        let mut sync = TokensSynchronizer::new();
        let count = sync.parse_json(json_data).expect("Valid token JSON");
        assert_eq!(count, 3);

        let css = sync.to_css();
        assert!(css.contains("--color-primary: #18181b;"));
        assert!(css.contains("--radius-sm: #71717a;") || css.contains("--radius-sm: 0.25rem;"));

        let rs = sync.to_rust_code();
        assert!(rs.contains("pub mod color"));
        assert!(rs.contains(r"pub const PRIMARY: &str = "));
        assert!(rs.contains("pub mod radius"));
        assert!(rs.contains(r#"pub const SM: &str = "0.25rem";"#));
    }

    #[test]
    fn test_security_tokens_injection_prevention() {
        let malicious_json = r##"{
            "color; } body { background: url(https://evil.com) }": {
                "bad_val": {
                    "$value": "red; } * { display: none",
                    "$type": "color"
                },
                "safe}; include!(\"/etc/passwd\"); //": {
                    "$value": "#ffffff",
                    "$type": "color"
                }
            }
        }"##;

        let mut sync = TokensSynchronizer::new();
        let count = sync.parse_json(malicious_json).expect("Parse JSON");
        // Only the token with a safe CSS value is retained
        assert_eq!(count, 1);

        let css = sync.to_css();
        assert!(!css.contains("evil.com"));
        assert!(!css.contains("display: none"));

        let rs = sync.to_rust_code();
        assert!(!rs.contains("include!"));
        assert!(!rs.contains("/etc/passwd"));
        assert!(rs.contains("pub const SAFE_INCLUDE_ETC_PASSWD: &str = \"#ffffff\";"));
    }
}
