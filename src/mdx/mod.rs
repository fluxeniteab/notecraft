//! MDX support.
//!
//! MDX is Markdown with JSX and JavaScript expressions. The native build takes a
//! deliberately conservative position:
//!
//! * **Editing and previewing never execute document code.** The preview renders
//!   Markdown, and shows JSX/expressions as inert, marked-up source.
//! * **Diagnostics are structural.** We validate the things a reader cares about
//!   — unbalanced JSX tags, unbalanced braces, unterminated strings and ESM
//!   syntax that cannot be rendered — and report a line and column so the editor
//!   can put a marker on the exact position.
//! * **No JavaScript runtime is embedded.** A safe execution sandbox for MDX is
//!   not something to fake; it is documented as not implemented rather than
//!   simulated with `eval`-style behaviour.

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A positioned problem in an MDX document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    fn error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            line,
            column,
            message: message.into(),
        }
    }

    fn warning(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            line,
            column,
            message: message.into(),
        }
    }

    pub fn describe(&self) -> String {
        format!(
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

/// The result of validating a document.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    /// JSX component names used by the document, for the "components" panel.
    pub components: Vec<String>,
    /// Expressions found in the document (source only).
    pub expressions: usize,
}

impl Report {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn first_error(&self) -> Option<&Diagnostic> {
        self.diagnostics
            .iter()
            .find(|d| d.severity == Severity::Error)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tag {
    Open(String),
    Close(String),
}

fn tag_name(inner: &str) -> Option<(String, bool)> {
    let inner = inner.trim();
    if inner.is_empty() || inner.starts_with('!') || inner.starts_with('?') {
        return None;
    }
    let closing = inner.starts_with('/');
    let body = inner.trim_start_matches('/').trim();
    let name: String = body
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_' || *c == ':')
        .collect();
    if name.is_empty() {
        return None;
    }
    let self_closing = inner.trim_end().ends_with('/');
    Some((name, closing || self_closing))
}

/// Counts JSX opening/closing tags, braces and quotes in one pass.
fn scan(text: &str) -> (Vec<Diagnostic>, Vec<String>, usize) {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<(Tag, usize, usize)> = Vec::new();
    let mut components: Vec<String> = Vec::new();
    let mut expressions = 0usize;
    let mut brace_depth: Vec<(usize, usize)> = Vec::new();
    let mut in_fence = false;
    let mut in_string: Option<(char, usize, usize)> = None;

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let mut chars = line.char_indices().peekable();
        while let Some((col, ch)) = chars.next() {
            if let Some((quote, _start_line, _start_col)) = in_string {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == quote {
                    in_string = None;
                }
                continue;
            }
            match ch {
                '"' | '\'' | '`' => in_string = Some((ch, line_no, col + 1)),
                '{' => {
                    brace_depth.push((line_no, col + 1));
                    expressions += 1;
                }
                '}' => {
                    if brace_depth.pop().is_none() {
                        diagnostics.push(Diagnostic::error(
                            line_no,
                            col + 1,
                            "closing brace without a matching opening brace",
                        ));
                    }
                }
                '<' => {
                    let rest: String = line[col..].to_string();
                    if let Some(close) = rest.find('>') {
                        let inner = &rest[1..close];
                        if let Some((name, closes)) = tag_name(inner) {
                            if closes {
                                // Closing or self-closing: match against the stack.
                                let expect = if inner.trim_start().starts_with('/') {
                                    Some(Tag::Close(name.clone()))
                                } else {
                                    None
                                };
                                if let Some(expect) = expect {
                                    match stack.pop() {
                                        Some((open, open_line, open_col)) => {
                                            if open != expect {
                                                let open_name = match &open {
                                                    Tag::Open(n) => n.clone(),
                                                    Tag::Close(n) => n.clone(),
                                                };
                                                diagnostics.push(Diagnostic::error(
                                                    line_no,
                                                    col + 1,
                                                    format!(
                                                        "closing tag `</{name}>` does not match `{open_name}` opened at line {open_line}, column {open_col}"
                                                    ),
                                                ));
                                            }
                                        }
                                        None => diagnostics.push(Diagnostic::error(
                                            line_no,
                                            col + 1,
                                            format!("closing tag `</{name}>` has no opening tag"),
                                        )),
                                    }
                                } else if !name.is_empty()
                                    && name.chars().next().is_some_and(|c| c.is_uppercase())
                                    && !components.contains(&name)
                                {
                                    components.push(name.clone());
                                }
                            } else {
                                stack.push((Tag::Open(name.clone()), line_no, col + 1));
                                if name.chars().next().is_some_and(|c| c.is_uppercase())
                                    && !components.contains(&name)
                                {
                                    components.push(name);
                                }
                            }
                            // Skip past the tag's `>`.
                            for _ in 0..rest[..=close].chars().count().saturating_sub(1) {
                                chars.next();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Strings do not span lines in our model; report and reset.
        if let Some((quote, start_line, start_col)) = in_string {
            if start_line != line_no {
                diagnostics.push(Diagnostic::error(
                    start_line,
                    start_col,
                    format!("unterminated string starting with {quote}"),
                ));
                in_string = None;
            }
        }
    }

    if let Some((quote, line, column)) = in_string {
        diagnostics.push(Diagnostic::error(
            line,
            column,
            format!("unterminated string starting with {quote}"),
        ));
    }
    for (tag, line, column) in stack {
        if let Tag::Open(name) = tag {
            diagnostics.push(Diagnostic::error(
                line,
                column,
                format!("unclosed component `<{name}>`"),
            ));
        }
    }
    for (line, column) in brace_depth {
        diagnostics.push(Diagnostic::error(line, column, "unclosed `{` expression"));
    }

    (diagnostics, components, expressions)
}

/// Validate an MDX document.
pub fn check(text: &str) -> Report {
    let (mut diagnostics, components, expressions) = scan(text);

    // ESM blocks cannot be evaluated here; warn once, without pretending they
    // will work in the preview.
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
            diagnostics.push(Diagnostic::warning(
                index + 1,
                1,
                "ESM is not evaluated in the preview; the statement is preserved as source",
            ));
        }
    }

    diagnostics.sort_by_key(|d| (d.line, d.column));
    Report {
        diagnostics,
        components,
        expressions,
    }
}

/// Whether document code may run in the preview.
///
/// Always `false`: MDX execution is intentionally disabled in this build, and
/// the UI states that instead of implying a sandbox exists.
#[allow(dead_code)]
pub const fn execution_enabled() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_components_pass() {
        let report = check("<Callout type=\"info\">\n  Text {1 + 1}\n</Callout>\n");
        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        assert_eq!(report.components, vec!["Callout"]);
        assert_eq!(report.expressions, 1);
    }

    #[test]
    fn self_closing_tags_are_accepted() {
        let report = check("<Icon name=\"star\" />\n<Badge />\n");
        assert!(!report.has_errors(), "{:?}", report.diagnostics);
    }

    #[test]
    fn unclosed_component_reports_its_line() {
        let report = check("# Title\n\n<Card>\n\nbody\n");
        let error = report.first_error().expect("expected an error");
        assert_eq!(error.line, 3);
        assert!(error.message.contains("unclosed"));
    }

    #[test]
    fn mismatched_tags_are_reported() {
        let report = check("<Card>\n</Panel>\n");
        let error = report.first_error().expect("expected an error");
        assert!(
            error.message.contains("does not match"),
            "{}",
            error.message
        );
    }

    #[test]
    fn stray_closing_tag_is_reported() {
        let report = check("text\n</Bogus>\n");
        assert!(report
            .first_error()
            .unwrap()
            .message
            .contains("no opening tag"));
    }

    #[test]
    fn unclosed_expression_reports_its_position() {
        let report = check("value: {1 + 2\n");
        let error = report.first_error().expect("expected an error");
        assert_eq!(error.line, 1);
        assert_eq!(error.column, 8);
    }

    #[test]
    fn code_fences_are_not_parsed_as_jsx() {
        let report = check("```jsx\n<Unclosed>\n```\n");
        assert!(!report.has_errors(), "{:?}", report.diagnostics);
    }

    #[test]
    fn esm_produces_a_warning_not_an_error() {
        let report = check("import x from 'y'\n\n# Title\n");
        assert!(!report.has_errors());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning));
    }

    #[test]
    fn quotes_inside_expressions_do_not_confuse_the_scanner() {
        let report = check("<p>{\"a } brace in a string\"}</p>\n");
        assert!(!report.has_errors(), "{:?}", report.diagnostics);
    }

    #[test]
    fn execution_is_disabled_by_policy() {
        assert!(!execution_enabled());
    }
}
