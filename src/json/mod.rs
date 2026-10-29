//! JSON support.
//!
//! Parsing is delegated to `serde_json`. It reports the exact line and column of
//! the first syntax error, which is what the editor diagnostics need, and with
//! the `preserve_order` feature the viewer shows object keys in document order
//! instead of re-sorting them.
//!
//! Nothing here evaluates JSON: values are only ever turned into text.

use serde_json::Value;

/// A syntax error with a human-readable position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

/// The result of parsing a document.
#[derive(Debug, Clone, Default)]
pub struct JsonDoc {
    pub root: Option<Value>,
    pub error: Option<JsonError>,
    /// Top-level type name for the status bar ("object", "array", …).
    pub shape: Option<&'static str>,
}

impl JsonDoc {
    /// True when the document parsed. Used by tests and by callers that only
    /// care about validity, without inspecting the tree.
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.root.is_some() && self.error.is_none()
    }

    pub fn parse(text: &str) -> Self {
        match serde_json::from_str::<Value>(text) {
            Ok(value) => Self {
                shape: Some(shape_name(&value)),
                root: Some(value),
                error: None,
            },
            Err(err) => Self {
                root: None,
                error: Some(JsonError {
                    message: strip_position(&err.to_string()),
                    line: err.line(),
                    column: err.column(),
                }),
                shape: None,
            },
        }
    }
}

fn shape_name(value: &Value) -> &'static str {
    match value {
        Value::Object(_) => "object",
        Value::Array(_) => "array",
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Null => "null",
    }
}

/// `serde_json` prefixes its message with "line N column M"; the UI shows the
/// position separately, so drop the duplicate prefix.
fn strip_position(message: &str) -> String {
    match message.find(" at line ") {
        Some(idx) => message[..idx].to_string(),
        None => message.to_string(),
    }
}

/// Re-indent a document. Invalid input is refused rather than rewritten.
pub fn format(text: &str, indent: usize) -> Result<String, JsonError> {
    let doc = JsonDoc::parse(text);
    let value = doc.root.ok_or_else(|| {
        doc.error.unwrap_or(JsonError {
            message: "invalid JSON".into(),
            line: 1,
            column: 1,
        })
    })?;
    let indent = indent.clamp(1, 8);
    let mut out = Vec::new();
    let indent_bytes = " ".repeat(indent);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    value.serialize(&mut ser).map_err(|err| JsonError {
        message: err.to_string(),
        line: 1,
        column: 1,
    })?;
    String::from_utf8(out).map_err(|_| JsonError {
        message: "formatter produced invalid UTF-8".into(),
        line: 1,
        column: 1,
    })
}

/// Collapse a document to one line.
pub fn minify(text: &str) -> Result<String, JsonError> {
    let doc = JsonDoc::parse(text);
    let value = doc.root.ok_or_else(|| {
        doc.error.unwrap_or(JsonError {
            message: "invalid JSON".into(),
            line: 1,
            column: 1,
        })
    })?;
    serde_json::to_string(&value).map_err(|err| JsonError {
        message: err.to_string(),
        line: 1,
        column: 1,
    })
}

use serde::Serialize;

/// One row of the structured view.
#[derive(Debug, Clone)]
pub struct JsonNode {
    /// JSON Pointer (`/users/0/name`), used for collapse state and copy-path.
    pub pointer: String,
    /// The key or index this node sits under.
    pub key: String,
    pub depth: usize,
    pub kind: NodeKind,
    pub children: usize,
    /// Compact preview of a scalar, empty for containers.
    pub preview: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Object => "object",
            NodeKind::Array => "array",
            NodeKind::String => "string",
            NodeKind::Number => "number",
            NodeKind::Bool => "boolean",
            NodeKind::Null => "null",
        }
    }

    pub fn is_container(self) -> bool {
        matches!(self, NodeKind::Object | NodeKind::Array)
    }

    /// CSS-style colour role for the viewer.
    pub fn color_role(self) -> ColorRole {
        match self {
            NodeKind::Object | NodeKind::Array => ColorRole::Structure,
            NodeKind::String => ColorRole::String,
            NodeKind::Number => ColorRole::Number,
            NodeKind::Bool => ColorRole::Bool,
            NodeKind::Null => ColorRole::Null,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    Structure,
    String,
    Number,
    Bool,
    Null,
}

fn escape_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

const MAX_PREVIEW: usize = 72;

fn scalar_preview(value: &Value) -> String {
    let text = match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".into(),
        _ => String::new(),
    };
    truncate(&text, MAX_PREVIEW)
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Flatten the document into display rows, skipping children of collapsed nodes.
pub fn flatten(root: &Value, collapsed: &std::collections::HashSet<String>) -> Vec<JsonNode> {
    let mut rows = Vec::new();
    push_node(root, "", "(root)", 0, collapsed, &mut rows);
    rows
}

fn push_node(
    value: &Value,
    pointer: &str,
    key: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    rows: &mut Vec<JsonNode>,
) {
    let kind = match value {
        Value::Object(_) => NodeKind::Object,
        Value::Array(_) => NodeKind::Array,
        Value::String(_) => NodeKind::String,
        Value::Number(_) => NodeKind::Number,
        Value::Bool(_) => NodeKind::Bool,
        Value::Null => NodeKind::Null,
    };
    let children = match value {
        Value::Object(map) => map.len(),
        Value::Array(arr) => arr.len(),
        _ => 0,
    };
    rows.push(JsonNode {
        pointer: if pointer.is_empty() {
            "/".into()
        } else {
            pointer.to_string()
        },
        key: key.to_string(),
        depth,
        kind,
        children,
        preview: scalar_preview(value),
    });
    if kind.is_container() && !collapsed.contains(pointer) {
        match value {
            Value::Object(map) => {
                for (child_key, child) in map {
                    let child_pointer = format!("{}/{}", pointer, escape_pointer_token(child_key));
                    push_node(child, &child_pointer, child_key, depth + 1, collapsed, rows);
                }
            }
            Value::Array(arr) => {
                for (index, child) in arr.iter().enumerate() {
                    let child_pointer = format!("{pointer}/{index}");
                    push_node(
                        child,
                        &child_pointer,
                        &index.to_string(),
                        depth + 1,
                        collapsed,
                        rows,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Every pointer in the document, so the UI can expand/collapse all.
pub fn all_containers(root: &Value) -> Vec<String> {
    fn walk(value: &Value, pointer: &str, out: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                out.push(if pointer.is_empty() {
                    "/".into()
                } else {
                    pointer.to_string()
                });
                for (k, v) in map {
                    walk(v, &format!("{}/{}", pointer, escape_pointer_token(k)), out);
                }
            }
            Value::Array(arr) => {
                out.push(if pointer.is_empty() {
                    "/".into()
                } else {
                    pointer.to_string()
                });
                for (i, v) in arr.iter().enumerate() {
                    walk(v, &format!("{pointer}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(root, "", &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn valid_documents_parse() {
        let doc = JsonDoc::parse(r#"{"a": 1, "b": [true, null, "x"]}"#);
        assert!(doc.is_valid());
        assert_eq!(doc.shape, Some("object"));
    }

    #[test]
    fn invalid_documents_report_a_position_and_keep_the_root_empty() {
        let doc = JsonDoc::parse("{\n  \"a\": ,\n}\n");
        assert!(!doc.is_valid());
        let err = doc.error.unwrap();
        assert_eq!(err.line, 2);
        assert!(err.column > 0, "column must be reported");
        assert!(
            !err.message.contains("line 2"),
            "the position is not duplicated"
        );
    }

    #[test]
    fn key_order_is_preserved() {
        let doc = JsonDoc::parse(r#"{"zebra": 1, "alpha": 2, "middle": 3}"#);
        let rows = flatten(doc.root.as_ref().unwrap(), &HashSet::new());
        let keys: Vec<_> = rows.iter().skip(1).map(|r| r.key.as_str()).collect();
        assert_eq!(keys, ["zebra", "alpha", "middle"]);
    }

    #[test]
    fn nested_pointers_are_escaped() {
        let doc = JsonDoc::parse(r#"{"a/b": {"c~d": 1}}"#);
        let rows = flatten(doc.root.as_ref().unwrap(), &HashSet::new());
        let pointers: Vec<_> = rows.iter().map(|r| r.pointer.clone()).collect();
        assert!(pointers.contains(&"/a~1b".to_string()));
        assert!(pointers.contains(&"/a~1b/c~0d".to_string()));
    }

    #[test]
    fn collapsed_nodes_hide_their_children() {
        let doc = JsonDoc::parse(r#"{"a": {"b": 1}, "c": 2}"#);
        let mut collapsed = HashSet::new();
        collapsed.insert("/a".to_string());
        let rows = flatten(doc.root.as_ref().unwrap(), &collapsed);
        assert!(rows.iter().any(|r| r.key == "a"));
        assert!(!rows.iter().any(|r| r.key == "b"));
    }

    #[test]
    fn formatting_refuses_invalid_input() {
        assert!(format("{\"a\": }", 2).is_err());
        let pretty = format(r#"{"a":[1,2]}"#, 2).unwrap();
        assert_eq!(pretty, "{\n  \"a\": [\n    1,\n    2\n  ]\n}");
    }

    #[test]
    fn minify_roundtrips() {
        let min = minify("{\n  \"a\": 1\n}\n").unwrap();
        assert_eq!(min, r#"{"a":1}"#);
    }

    #[test]
    fn unicode_and_escapes_survive() {
        let doc = JsonDoc::parse(r#"{"s": "tab\there \u00e9 \ud83d\ude00 \"quoted\""}"#);
        assert!(doc.is_valid());
        let rows = flatten(doc.root.as_ref().unwrap(), &HashSet::new());
        assert!(rows
            .iter()
            .any(|r| r.kind == NodeKind::String && r.preview.contains('é')));
    }

    #[test]
    fn deep_documents_do_not_panic() {
        let mut text = String::new();
        for _ in 0..400 {
            text.push('[');
        }
        text.push('1');
        for _ in 0..400 {
            text.push(']');
        }
        let doc = JsonDoc::parse(&text);
        assert!(doc.is_valid());
    }
}
