//! Simple keyword-based syntax highlighter for Rust.
//!
//! Produces colored TextRuns for each line. A real implementation would use
//! tree-sitter, but this is enough to exercise the rich-text rendering path
//! and visually verify that per-token coloring works.

use crate::model::Language;
use fission_core::op::Color;

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub color: Color,
}

/// Tokenize a line of source code into colored spans.
pub fn highlight_line(line: &str, language: Language) -> Vec<StyledSpan> {
    match language {
        Language::Rust => highlight_rust_line(line),
        Language::Toml => highlight_toml_line(line),
        _ => vec![StyledSpan {
            text: line.to_string(),
            color: Color { r: 212, g: 212, b: 212, a: 255 },
        }],
    }
}

// --- Rust colors (VS Code Dark+ inspired) ---

const KEYWORD: Color = Color { r: 86, g: 156, b: 214, a: 255 };    // blue
const STRING_LIT: Color = Color { r: 206, g: 145, b: 120, a: 255 }; // brown/orange
const COMMENT: Color = Color { r: 106, g: 153, b: 85, a: 255 };     // green
const NUMBER: Color = Color { r: 181, g: 206, b: 168, a: 255 };     // light green
const TYPE_COLOR: Color = Color { r: 78, g: 201, b: 176, a: 255 };  // teal
const MACRO_COLOR: Color = Color { r: 220, g: 220, b: 170, a: 255 };// yellow
const PUNCT: Color = Color { r: 212, g: 212, b: 212, a: 255 };      // white/gray
const IDENT: Color = Color { r: 156, g: 220, b: 254, a: 255 };      // light blue
const DEFAULT: Color = Color { r: 212, g: 212, b: 212, a: 255 };

const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait",
    "for", "while", "loop", "if", "else", "match", "return", "break", "continue",
    "const", "static", "type", "where", "as", "in", "ref", "self", "Self",
    "super", "crate", "async", "await", "dyn", "move", "unsafe", "extern",
    "true", "false",
];

const RUST_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "f32", "f64", "bool", "char", "str",
    "String", "Vec", "Option", "Result", "Box", "Arc", "Rc",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet",
];

fn highlight_rust_line(line: &str) -> Vec<StyledSpan> {
    let trimmed = line.trim_start();

    // Full-line comment
    if trimmed.starts_with("//") {
        return vec![StyledSpan { text: line.to_string(), color: COMMENT }];
    }

    let mut spans = Vec::new();
    let mut chars = line.char_indices().peekable();
    let mut current_start = 0;

    while let Some(&(i, ch)) = chars.peek() {
        // String literal
        if ch == '"' {
            // Flush preceding text
            if i > current_start {
                push_tokens(&line[current_start..i], &mut spans);
            }
            chars.next();
            let str_start = i;
            let mut escaped = false;
            while let Some(&(j, c)) = chars.peek() {
                chars.next();
                if escaped {
                    escaped = false;
                    continue;
                }
                if c == '\\' {
                    escaped = true;
                    continue;
                }
                if c == '"' {
                    spans.push(StyledSpan {
                        text: line[str_start..=j].to_string(),
                        color: STRING_LIT,
                    });
                    current_start = j + 1;
                    break;
                }
            }
            continue;
        }

        // Line comment
        if ch == '/' {
            let next = line.as_bytes().get(i + 1).copied();
            if next == Some(b'/') {
                if i > current_start {
                    push_tokens(&line[current_start..i], &mut spans);
                }
                spans.push(StyledSpan {
                    text: line[i..].to_string(),
                    color: COMMENT,
                });
                return spans;
            }
        }

        chars.next();
    }

    // Flush remainder
    if current_start < line.len() {
        push_tokens(&line[current_start..], &mut spans);
    }

    if spans.is_empty() {
        spans.push(StyledSpan { text: line.to_string(), color: DEFAULT });
    }

    spans
}

fn push_tokens(text: &str, spans: &mut Vec<StyledSpan>) {
    // Split on word boundaries and classify
    let mut i = 0;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        // Skip whitespace, keep it as-is
        if bytes[i].is_ascii_whitespace() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            spans.push(StyledSpan {
                text: text[start..i].to_string(),
                color: DEFAULT,
            });
            continue;
        }

        // Identifier or keyword
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &text[start..i];

            // Check for macro (word followed by !)
            let is_macro = i < bytes.len() && bytes[i] == b'!';

            let color = if is_macro {
                MACRO_COLOR
            } else if RUST_KEYWORDS.contains(&word) {
                KEYWORD
            } else if RUST_TYPES.contains(&word) {
                TYPE_COLOR
            } else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                TYPE_COLOR
            } else if word.chars().all(|c| c.is_ascii_digit() || c == '_') {
                NUMBER
            } else {
                IDENT
            };

            let end = if is_macro { i + 1 } else { i };
            if is_macro { i += 1; }
            spans.push(StyledSpan {
                text: text[start..end].to_string(),
                color,
            });
            continue;
        }

        // Number literal
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'_') {
                i += 1;
            }
            spans.push(StyledSpan {
                text: text[start..i].to_string(),
                color: NUMBER,
            });
            continue;
        }

        // Punctuation
        let start = i;
        i += 1;
        spans.push(StyledSpan {
            text: text[start..i].to_string(),
            color: PUNCT,
        });
    }
}

fn highlight_toml_line(line: &str) -> Vec<StyledSpan> {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        return vec![StyledSpan { text: line.to_string(), color: COMMENT }];
    }

    if trimmed.starts_with('[') {
        return vec![StyledSpan { text: line.to_string(), color: KEYWORD }];
    }

    // key = value
    if let Some(eq_pos) = line.find('=') {
        let key = &line[..eq_pos];
        let rest = &line[eq_pos..];
        return vec![
            StyledSpan { text: key.to_string(), color: TYPE_COLOR },
            StyledSpan { text: rest.to_string(), color: STRING_LIT },
        ];
    }

    vec![StyledSpan { text: line.to_string(), color: DEFAULT }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_keyword_highlighted() {
        let spans = highlight_line("fn main() {", Language::Rust);
        assert!(spans.iter().any(|s| s.text == "fn" && s.color == KEYWORD));
    }

    #[test]
    fn rust_comment_highlighted() {
        let spans = highlight_line("// this is a comment", Language::Rust);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].color, COMMENT);
    }

    #[test]
    fn rust_string_highlighted() {
        let spans = highlight_line("let x = \"hello\";", Language::Rust);
        assert!(spans.iter().any(|s| s.text.contains("hello") && s.color == STRING_LIT));
    }

    #[test]
    fn toml_section_highlighted() {
        let spans = highlight_line("[package]", Language::Toml);
        assert_eq!(spans[0].color, KEYWORD);
    }

    #[test]
    fn plain_text_no_crash() {
        let spans = highlight_line("just some text", Language::Plain);
        assert!(!spans.is_empty());
    }
}
