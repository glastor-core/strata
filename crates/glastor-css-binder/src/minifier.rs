//! High-speed CSS minifier.
//!
//! Compresses CSS stylesheets by stripping comments, collapsing whitespace,
//! and pruning redundant syntax tokens with zero external runtime dependencies.

/// CSS minification engine.
#[derive(Debug, Clone, Default)]
pub struct CssMinifier;

impl CssMinifier {
    /// Creates a new minifier instance.
    pub fn new() -> Self {
        Self
    }

    /// Minifies a CSS string into an optimized single-line payload.
    pub fn minify(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        let mut in_comment = false;
        let mut in_string: Option<char> = None;
        let mut last_char = ' ';

        while let Some(ch) = chars.next() {
            // Check comment start
            if !in_comment && in_string.is_none() && ch == '/' && chars.peek() == Some(&'*') {
                in_comment = true;
                chars.next(); // consume '*'
                continue;
            }

            // Check comment end
            if in_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    in_comment = false;
                    chars.next(); // consume '/'
                }
                continue;
            }

            // Check string literals
            if (ch == '"' || ch == '\'') && last_char != '\\' {
                if in_string == Some(ch) {
                    in_string = None;
                } else if in_string.is_none() {
                    in_string = Some(ch);
                }
            }

            if in_string.is_some() {
                result.push(ch);
                last_char = ch;
                continue;
            }

            // Compress whitespace
            if ch.is_whitespace() {
                if !Self::is_delimiter(last_char) && !result.is_empty() {
                    if let Some(&next) = chars.peek() {
                        if !Self::is_delimiter(next) {
                            result.push(' ');
                            last_char = ' ';
                        }
                    }
                }
                continue;
            }

            // Avoid redundant spaces around delimiters
            if Self::is_delimiter(ch) && last_char == ' ' {
                result.pop();
            }

            result.push(ch);
            last_char = ch;
        }

        result.trim().to_string()
    }

    fn is_delimiter(ch: char) -> bool {
        matches!(ch, '{' | '}' | ';' | ':' | ',' | '>' | '+' | '~')
    }
}
