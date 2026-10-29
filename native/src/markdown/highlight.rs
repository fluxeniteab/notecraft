//! Syntax highlighting.
//!
//! A deliberately small, line-oriented tokenizer. It has no parser state between
//! lines, which is exactly what an egui text layouter can support per row, and
//! it is enough to make Markdown, MDX, JSON, YAML, HTML, CSS and JS/TS readable.
//! It never evaluates anything and never allocates per character.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

use crate::theme::Theme;

/// Language families the tokenizer understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Markdown,
    Mdx,
    Json,
    Yaml,
    Html,
    Css,
    Code,
    Plain,
}

impl Lang {
    pub fn from_fence(info: &str) -> Lang {
        let name = info
            .split(|c: char| c.is_whitespace() || c == ',' || c == '{')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match name.as_str() {
            "md" | "markdown" | "gfm" => Lang::Markdown,
            "mdx" => Lang::Mdx,
            "json" | "jsonc" | "json5" => Lang::Json,
            "yaml" | "yml" => Lang::Yaml,
            "html" | "htm" | "xml" | "svg" | "vue" | "svelte" => Lang::Html,
            "css" | "scss" | "less" => Lang::Css,
            "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx" | "mjs" | "cjs" | "rs"
            | "rust" | "py" | "python" | "sh" | "bash" | "zsh" | "ps1" | "go" | "c" | "cpp"
            | "java" | "kotlin" | "swift" | "sql" | "toml" | "ini" => Lang::Code,
            _ => Lang::Plain,
        }
    }

    pub fn from_syntax(syntax: crate::documents::Syntax) -> Lang {
        use crate::documents::Syntax;
        match syntax {
            Syntax::Markdown => Lang::Markdown,
            Syntax::Mdx => Lang::Mdx,
            Syntax::Json => Lang::Json,
            Syntax::Yaml => Lang::Yaml,
            Syntax::Html => Lang::Html,
            Syntax::Css => Lang::Css,
            Syntax::Code => Lang::Code,
            Syntax::Plain => Lang::Plain,
        }
    }
}

/// Colour roles, resolved against the active palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Text,
    Comment,
    Keyword,
    String,
    Number,
    Tag,
    Attribute,
    Heading,
    Emphasis,
    Code,
    Link,
    Mark,
    Punctuation,
}

fn color_for(theme: &Theme, role: Role) -> Color32 {
    let p = &theme.palette;
    match role {
        Role::Text => p.fg,
        Role::Comment => p.fg_faint,
        Role::Keyword => p.accent,
        Role::String => p.success,
        Role::Number => p.warning,
        Role::Tag => p.accent,
        Role::Attribute => p.fg_secondary,
        Role::Heading => p.fg,
        Role::Emphasis => p.fg_secondary,
        Role::Code => p.success,
        Role::Link => p.accent,
        Role::Mark => p.warning,
        Role::Punctuation => p.fg_muted,
    }
}

/// A coloured byte range within one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub role: Role,
    pub bold: bool,
}

const KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "defer",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "fn",
    "for",
    "from",
    "func",
    "function",
    "if",
    "impl",
    "import",
    "in",
    "interface",
    "let",
    "loop",
    "match",
    "mod",
    "mut",
    "new",
    "null",
    "package",
    "pub",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "switch",
    "this",
    "throw",
    "trait",
    "true",
    "try",
    "type",
    "undefined",
    "use",
    "var",
    "void",
    "where",
    "while",
    "yield",
    "def",
    "elif",
    "except",
    "lambda",
    "None",
    "True",
    "False",
    "and",
    "or",
    "not",
    "pass",
    "raise",
    "with",
];

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Tokenize a single line.
pub fn tokenize(lang: Lang, line: &str) -> Vec<Token> {
    match lang {
        Lang::Plain => Vec::new(),
        Lang::Markdown | Lang::Mdx => markdown_line(lang, line),
        Lang::Json => json_line(line),
        Lang::Yaml => yaml_line(line),
        Lang::Html => html_line(line),
        Lang::Css => css_line(line),
        Lang::Code => code_line(line),
    }
}

fn push(tokens: &mut Vec<Token>, start: usize, end: usize, role: Role, bold: bool) {
    if end > start {
        tokens.push(Token {
            start,
            end,
            role,
            bold,
        });
    }
}

fn markdown_line(lang: Lang, line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        push(&mut tokens, 0, line.len(), Role::Comment, false);
        return tokens;
    }
    if trimmed.starts_with('#') && trimmed.chars().take_while(|c| *c == '#').count() <= 6 {
        push(&mut tokens, leading, line.len(), Role::Heading, true);
        return tokens;
    }
    if trimmed.starts_with('>') {
        push(&mut tokens, leading, line.len(), Role::Emphasis, false);
        return tokens;
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        push(&mut tokens, leading, leading + 1, Role::Keyword, true);
    }
    if let Some(rest) = trimmed
        .strip_prefix("- [ ] ")
        .or_else(|| trimmed.strip_prefix("- [x] "))
    {
        let _ = rest;
        push(&mut tokens, leading + 2, leading + 5, Role::Mark, false);
    }

    // Inline code, links, emphasis and MDX braces.
    let mut i = 0;
    while i < bytes.len() {
        let c = line[i..].chars().next().unwrap_or(' ');
        let len = c.len_utf8();
        match c {
            '`' => {
                let close = line[i + len..].find('`').map(|offset| i + len + offset);
                let end = close.map_or(line.len(), |e| e + 1);
                push(&mut tokens, i, end, Role::Code, false);
                i = end;
            }
            '[' => {
                if let Some(close) = line[i..].find("](") {
                    let end = line[i + close + 2..]
                        .find(')')
                        .map(|offset| i + close + 2 + offset + 1)
                        .unwrap_or(line.len());
                    push(&mut tokens, i, end.min(line.len()), Role::Link, false);
                    i = end.min(line.len());
                } else {
                    i += len;
                }
            }
            '{' if lang == Lang::Mdx => {
                let end = line[i..].find('}').map(|o| i + o + 1).unwrap_or(line.len());
                push(&mut tokens, i, end, Role::Mark, false);
                i = end;
            }
            '<' if lang == Lang::Mdx => {
                let end = line[i..].find('>').map(|o| i + o + 1).unwrap_or(line.len());
                push(&mut tokens, i, end, Role::Tag, true);
                i = end;
            }
            '*' | '_' => {
                push(&mut tokens, i, i + len, Role::Emphasis, false);
                i += len;
            }
            _ => i += len,
        }
    }
    tokens
}

/// End of the string literal that starts at `start` (inclusive of its quotes).
///
/// Scans by `char`, so a multi-byte character inside a literal can never leave
/// the token on a non-boundary byte.
fn string_end(line: &str, start: usize) -> usize {
    let Some(quote) = line[start..].chars().next() else {
        return start;
    };
    let mut escaped = false;
    for (offset, ch) in line[start + quote.len_utf8()..].char_indices() {
        let at = start + quote.len_utf8() + offset;
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            c if c == quote => return at + ch.len_utf8(),
            _ => {}
        }
    }
    line.len()
}

/// End of the identifier that starts at `start`.
fn ident_end(line: &str, start: usize) -> usize {
    for (offset, ch) in line[start..].char_indices() {
        if !is_ident_char(ch) {
            return start + offset;
        }
    }
    line.len()
}

fn json_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let Some(c) = line[i..].chars().next() else {
            break;
        };
        match c {
            '"' => {
                let end = string_end(line, i);
                let is_key = line[end..].trim_start().starts_with(':');
                push(
                    &mut tokens,
                    i,
                    end,
                    if is_key {
                        Role::Attribute
                    } else {
                        Role::String
                    },
                    is_key,
                );
                i = end;
                continue;
            }
            c if c.is_ascii_digit()
                || (c == '-'
                    && line[i + 1..]
                        .chars()
                        .next()
                        .is_some_and(|d| d.is_ascii_digit())) =>
            {
                let mut end = i;
                for (offset, ch) in line[i..].char_indices() {
                    if ch.is_ascii_digit() || matches!(ch, '.' | 'e' | 'E' | '+' | '-') {
                        end = i + offset + ch.len_utf8();
                    } else {
                        break;
                    }
                }
                push(&mut tokens, i, end, Role::Number, false);
                i = end.max(i + 1);
                continue;
            }
            c if c.is_alphabetic() => {
                let end = ident_end(line, i);
                if matches!(&line[i..end], "true" | "false" | "null") {
                    push(&mut tokens, i, end, Role::Keyword, false);
                }
                i = end.max(i + c.len_utf8());
                continue;
            }
            '{' | '}' | '[' | ']' | ':' | ',' => {
                push(&mut tokens, i, i + 1, Role::Punctuation, false);
            }
            _ => {}
        }
        i += c.len_utf8();
    }
    tokens
}

fn yaml_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    if trimmed.starts_with('#') {
        push(&mut tokens, leading, line.len(), Role::Comment, false);
        return tokens;
    }
    if let Some(colon) = trimmed.find(": ") {
        push(
            &mut tokens,
            leading,
            leading + colon + 1,
            Role::Attribute,
            true,
        );
        let rest_start = leading + colon + 2;
        let value = line[rest_start..].trim_end();
        let role = if value == "true" || value == "false" || value == "null" {
            Role::Keyword
        } else if value.parse::<f64>().is_ok() {
            Role::Number
        } else if value.starts_with('"') || value.starts_with('\'') {
            Role::String
        } else {
            Role::Text
        };
        push(
            &mut tokens,
            rest_start,
            rest_start + value.len(),
            role,
            false,
        );
        return tokens;
    }
    if trimmed.starts_with("- ") {
        push(&mut tokens, leading, leading + 1, Role::Punctuation, false);
    }
    tokens
}

fn html_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let Some(c) = line[i..].chars().next() else {
            break;
        };
        match c {
            '<' => {
                let end = line[i..].find('>').map(|o| i + o + 1).unwrap_or(line.len());
                push(&mut tokens, i, end, Role::Tag, false);
                i = end;
            }
            '&' => {
                let end = line[i..].find(';').map(|o| i + o + 1).unwrap_or(i + 1);
                push(&mut tokens, i, end, Role::Mark, false);
                i = end;
            }
            _ => i += c.len_utf8(),
        }
    }
    tokens
}

fn css_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    if trimmed.starts_with("/*") || trimmed.starts_with('*') {
        push(&mut tokens, 0, line.len(), Role::Comment, false);
        return tokens;
    }
    if let Some(colon) = trimmed.find(':') {
        push(
            &mut tokens,
            leading,
            leading + colon,
            Role::Attribute,
            false,
        );
        let value_start = leading + colon + 1;
        let value = line[value_start..].trim_end();
        let value_len = value.len();
        push(
            &mut tokens,
            value_start,
            value_start + value_len,
            Role::String,
            false,
        );
        return tokens;
    }
    if trimmed.contains('{') || trimmed.starts_with('@') {
        push(&mut tokens, leading, line.len(), Role::Tag, true);
    }
    tokens
}

fn code_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    if trimmed.starts_with("//") || trimmed.starts_with('#') && !trimmed.starts_with("#!") {
        push(&mut tokens, leading, line.len(), Role::Comment, false);
        return tokens;
    }

    let mut i = 0;
    while i < line.len() {
        let Some(c) = line[i..].chars().next() else {
            break;
        };
        // Line comment.
        if c == '/' && line[i + 1..].starts_with('/') {
            push(&mut tokens, i, line.len(), Role::Comment, false);
            break;
        }
        // Strings.
        if c == '"' || c == '\'' || c == '`' {
            let end = string_end(line, i);
            push(&mut tokens, i, end, Role::String, false);
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let mut end = i;
            for (offset, ch) in line[i..].char_indices() {
                if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_') {
                    end = i + offset + ch.len_utf8();
                } else {
                    break;
                }
            }
            push(&mut tokens, i, end, Role::Number, false);
            i = end.max(i + 1);
            continue;
        }
        if c.is_alphabetic() || c == '_' || c == '$' {
            let end = ident_end(line, i);
            let word = &line[i..end];
            if KEYWORDS.contains(&word) {
                push(&mut tokens, i, end, Role::Keyword, true);
            } else if line[end..].starts_with('(') {
                push(&mut tokens, i, end, Role::Tag, false);
            }
            i = end.max(i + c.len_utf8());
            continue;
        }
        i += c.len_utf8();
    }
    tokens
}

/// Build a syntax-coloured job for a whole code block.
pub fn highlight(theme: &Theme, lang: &str, code: &str, size: f32) -> LayoutJob {
    let parsed = Lang::from_fence(lang);
    let font = theme.mono(size, false);
    let mut job = LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        },
        ..Default::default()
    };
    let line_height = (size * 1.6).round();

    for (index, line) in code.lines().enumerate() {
        if index > 0 {
            job.append(
                "\n",
                0.0,
                base_format(theme, font.clone(), line_height, Role::Text),
            );
        }
        append_line(&mut job, theme, &font, line, line_height, parsed);
    }
    job
}

/// Build a syntax-coloured job for one editor row.
pub fn highlight_row(theme: &Theme, lang: Lang, line: &str, size: f32) -> LayoutJob {
    let font = theme.mono(size, false);
    let line_height = theme.row_height();
    let mut job = LayoutJob::default();
    append_line(&mut job, theme, &font, line, line_height, lang);
    if job.sections.is_empty() {
        job.append("", 0.0, base_format(theme, font, line_height, Role::Text));
    }
    job
}

fn base_format(theme: &Theme, font: FontId, line_height: f32, role: Role) -> TextFormat {
    TextFormat {
        font_id: font,
        color: color_for(theme, role),
        line_height: Some(line_height),
        ..Default::default()
    }
}

fn append_line(
    job: &mut LayoutJob,
    theme: &Theme,
    font: &FontId,
    line: &str,
    line_height: f32,
    lang: Lang,
) {
    let tokens = tokenize(lang, line);
    if tokens.is_empty() {
        job.append(
            line,
            0.0,
            base_format(theme, font.clone(), line_height, Role::Text),
        );
        return;
    }
    let mut cursor = 0;
    for token in tokens {
        if token.start > cursor {
            job.append(
                &line[cursor..token.start],
                0.0,
                base_format(theme, font.clone(), line_height, Role::Text),
            );
        }
        let start = token.start.max(cursor);
        if token.end > start {
            job.append(
                &line[start..token.end],
                0.0,
                TextFormat {
                    font_id: if token.bold && theme.fonts.mono_bold {
                        theme.mono(theme.editor_font_size * 0.92, true)
                    } else {
                        font.clone()
                    },
                    color: color_for(theme, token.role),
                    line_height: Some(line_height),
                    ..Default::default()
                },
            );
        }
        cursor = token.end.max(cursor);
    }
    if cursor < line.len() {
        job.append(
            &line[cursor..],
            0.0,
            base_format(theme, font.clone(), line_height, Role::Text),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> Theme {
        Theme::build(
            crate::theme::ThemeKind::Dark,
            "#4f86f7",
            &crate::settings::Settings::default(),
        )
    }

    fn roles(lang: Lang, line: &str) -> Vec<Role> {
        tokenize(lang, line).into_iter().map(|t| t.role).collect()
    }

    #[test]
    fn fence_languages_are_recognised() {
        assert_eq!(Lang::from_fence("md"), Lang::Markdown);
        assert_eq!(Lang::from_fence("mdx"), Lang::Mdx);
        assert_eq!(Lang::from_fence("json"), Lang::Json);
        assert_eq!(Lang::from_fence("yaml"), Lang::Yaml);
        assert_eq!(Lang::from_fence("ts {1,2}"), Lang::Code);
        assert_eq!(Lang::from_fence(""), Lang::Plain);
    }

    #[test]
    fn json_keys_and_strings_differ() {
        let tokens = tokenize(Lang::Json, r#"  "key": "value","#);
        assert_eq!(tokens[0].role, Role::Attribute);
        assert!(tokens.iter().any(|t| t.role == Role::String));
    }

    #[test]
    fn json_numbers_and_literals() {
        let r = roles(Lang::Json, r#"{"a": -12.5, "b": true, "c": null}"#);
        assert!(r.contains(&Role::Number));
        assert!(r.contains(&Role::Keyword));
    }

    #[test]
    fn comments_are_muted() {
        assert_eq!(roles(Lang::Code, "// note")[0], Role::Comment);
        assert_eq!(roles(Lang::Yaml, "# note")[0], Role::Comment);
        assert_eq!(roles(Lang::Css, "/* note */")[0], Role::Comment);
    }

    #[test]
    fn markdown_headings_and_code_are_marked() {
        assert_eq!(roles(Lang::Markdown, "## Title")[0], Role::Heading);
        assert_eq!(roles(Lang::Markdown, "```rust")[0], Role::Comment);
        assert!(roles(Lang::Markdown, "use `code` here").contains(&Role::Code));
        assert!(roles(Lang::Markdown, "see [docs](https://x.dev)").contains(&Role::Link));
    }

    #[test]
    fn mdx_marks_jsx_and_expressions() {
        let r = roles(Lang::Mdx, "<Callout>{2 + 2}</Callout>");
        assert!(r.contains(&Role::Tag));
        assert!(r.contains(&Role::Mark));
    }

    #[test]
    fn tokens_never_overlap_or_split_utf8() {
        for line in ["héllo wörld", "日本語 = \"テスト\"", "let x = 'é';"] {
            let tokens = tokenize(Lang::Code, line);
            let mut last = 0;
            for token in tokens {
                assert!(token.start >= last);
                assert!(token.end <= line.len());
                assert!(line.is_char_boundary(token.start));
                assert!(line.is_char_boundary(token.end));
                last = token.end;
            }
        }
    }

    #[test]
    fn highlighting_a_block_produces_text() {
        let job = highlight(&theme(), "rust", "fn main() {\n    // hi\n}", 13.0);
        assert!(job.text.contains("fn main"));
        assert!(!job.sections.is_empty());
    }
}
