//! Structured JSON inspection for the editor and the preview.
//!
//! Parsing is delegated to `serde_json`: a mature parser reports the byte
//! offset, line and column of the first syntax error, which is what the editor
//! diagnostics need. Nothing here evaluates JSON; values are only ever rendered
//! as text.

use serde::Serialize;
use serde_json::Value;

/// Hard limits so a pathological file cannot freeze the IPC bridge.
const MAX_NODES: usize = 20_000;
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, Serialize)]
pub struct JsonError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonNode {
    /// Object key, or the array index as a string.
    pub key: String,
    /// RFC 6901 JSON Pointer for this value (`/users/0/name`).
    pub pointer: String,
    pub kind: &'static str,
    /// Scalar rendering, or a compact summary for containers.
    pub preview: String,
    pub children: Vec<JsonNode>,
    /// Children existed but were not expanded because a limit was reached.
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonAnalysis {
    pub valid: bool,
    pub error: Option<JsonError>,
    pub root: Option<JsonNode>,
    pub keys: usize,
    pub nodes: usize,
    pub depth: usize,
    pub lines: usize,
    pub bytes: usize,
    /// True when the tree was cut short by MAX_NODES / MAX_DEPTH.
    pub truncated: bool,
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn scalar_preview(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => {
            // Keep the tree readable: collapse whitespace and truncate long
            // strings, but never silently coerce the value itself.
            let clean = text.replace(['\n', '\r', '\t'], " ");
            let mut shown: String = clean.chars().take(120).collect();
            if clean.chars().count() > 120 {
                shown.push('…');
            }
            format!("\"{shown}\"")
        }
        Value::Array(items) => format!("[{} items]", items.len()),
        Value::Object(map) => format!("{{{} keys}}", map.len()),
    }
}

fn pointer_child(base: &str, segment: &str) -> String {
    // RFC 6901 escaping: `~` becomes `~0`, `/` becomes `~1`.
    let escaped = segment.replace('~', "~0").replace('/', "~1");
    if base.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{base}/{escaped}")
    }
}

struct Budget {
    nodes: usize,
    keys: usize,
    depth: usize,
    truncated: bool,
}

fn build(value: &Value, key: &str, pointer: &str, depth: usize, budget: &mut Budget) -> JsonNode {
    budget.nodes += 1;
    budget.depth = budget.depth.max(depth);
    let mut children = Vec::new();
    let mut truncated = false;

    if budget.nodes <= MAX_NODES {
        match value {
            Value::Object(map) => {
                budget.keys += map.len();
                for (child_key, child) in map {
                    if budget.nodes >= MAX_NODES {
                        truncated = true;
                        budget.truncated = true;
                        break;
                    }
                    children.push(build(
                        child,
                        child_key,
                        &pointer_child(pointer, child_key),
                        depth + 1,
                        budget,
                    ));
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    if budget.nodes >= MAX_NODES {
                        truncated = true;
                        budget.truncated = true;
                        break;
                    }
                    let segment = index.to_string();
                    children.push(build(
                        child,
                        &segment,
                        &pointer_child(pointer, &segment),
                        depth + 1,
                        budget,
                    ));
                }
            }
            _ => {}
        }
    } else {
        truncated = true;
        budget.truncated = true;
    }

    if depth >= MAX_DEPTH {
        truncated = truncated || !children.is_empty();
        budget.truncated = budget.truncated || truncated;
        children.clear();
    }

    JsonNode {
        key: key.to_string(),
        pointer: pointer.to_string(),
        kind: kind_of(value),
        preview: scalar_preview(value),
        children,
        truncated,
    }
}

/// Validate and describe a JSON document. Never panics: every failure becomes a
/// diagnostic that the editor can show next to the offending line.
pub fn analyze(source: &str) -> JsonAnalysis {
    let lines = source.lines().count();
    match serde_json::from_str::<Value>(source) {
        Ok(value) => {
            let mut budget = Budget {
                nodes: 0,
                keys: 0,
                depth: 0,
                truncated: false,
            };
            let root = build(&value, "", "", 0, &mut budget);
            JsonAnalysis {
                valid: true,
                error: None,
                root: Some(root),
                keys: budget.keys,
                nodes: budget.nodes,
                depth: budget.depth,
                lines,
                bytes: source.len(),
                truncated: budget.truncated,
            }
        }
        Err(error) => JsonAnalysis {
            valid: false,
            error: Some(JsonError {
                message: error.to_string(),
                line: error.line(),
                column: error.column(),
                offset: {
                    // serde_json exposes a line/column pair; the editor only
                    // needs the position, so derive the offset from them.
                    let mut offset = 0;
                    for (index, line) in source.split_inclusive('\n').enumerate() {
                        if index + 1 == error.line() {
                            offset += error.column().saturating_sub(1);
                            break;
                        }
                        offset += line.len();
                    }
                    offset
                },
            }),
            root: None,
            keys: 0,
            nodes: 0,
            depth: 0,
            lines,
            bytes: source.len(),
            truncated: false,
        },
    }
}

/// Pretty-print with a configurable indent (2..=8 spaces).
pub fn format(source: &str, indent: usize) -> Result<String, String> {
    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let indent = indent.clamp(2, 8);
    let pad = " ".repeat(indent);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(pad.as_bytes());
    let mut buffer = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);
    value
        .serialize(&mut serializer)
        .map_err(|error| error.to_string())?;
    let mut text = String::from_utf8(buffer).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text)
}

/// Minify: parse first so invalid input is rejected instead of mangled.
pub fn minify(source: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    serde_json::to_string(&value).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_document_reports_structure() {
        let report = analyze(r#"{"name":"NoteCraft","count":3,"tags":["a","b"]}"#);
        assert!(report.valid);
        assert!(report.error.is_none());
        assert_eq!(report.keys, 3);
        let root = report.root.unwrap();
        assert_eq!(root.kind, "object");
        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].key, "name");
        assert_eq!(root.children[1].preview, "3");
        assert_eq!(root.children[2].pointer, "/tags");
        assert_eq!(root.children[2].children.len(), 2);
    }

    #[test]
    fn pointers_escape_json_pointer_metacharacters() {
        let report = analyze(r#"{"a/b":{"c~d":1}}"#);
        let root = report.root.unwrap();
        assert_eq!(root.children[0].pointer, "/a~1b");
        assert_eq!(root.children[0].children[0].pointer, "/a~1b/c~0d");
    }

    #[test]
    fn invalid_document_reports_a_position_and_keeps_the_source() {
        let source = "{\n  \"a\": 1,\n  \"b\": ,\n}";
        let report = analyze(source);
        assert!(!report.valid);
        assert!(report.root.is_none());
        let error = report.error.unwrap();
        assert!(error.line >= 3, "line was {}", error.line);
        assert!(!error.message.is_empty());
        // The caller still holds the original text; analysis never rewrites it.
        assert!(source.contains("\"b\": ,"));
    }

    #[test]
    fn unicode_and_escapes_round_trip() {
        let source = r#"{"emoji":"🎉","quote":"say \"hi\"","path":"a\\b"}"#;
        let report = analyze(source);
        assert!(report.valid);
        let root = report.root.unwrap();
        let by_key = |key: &str| {
            root.children
                .iter()
                .find(|node| node.key == key)
                .map(|node| node.preview.clone())
                .unwrap_or_default()
        };
        assert_eq!(by_key("emoji"), "\"🎉\"");
        assert_eq!(by_key("quote"), "\"say \\\"hi\\\"\"");
        assert_eq!(by_key("path"), "\"a\\b\"");
    }

    #[test]
    fn deep_nesting_is_bounded_not_fatal() {
        // Moderately nested input is valid and the reported depth is bounded.
        let mut source = String::new();
        for _ in 0..40 {
            source.push('[');
        }
        source.push('1');
        for _ in 0..40 {
            source.push(']');
        }
        let report = analyze(&source);
        assert!(report.valid);
        assert!(report.depth <= MAX_DEPTH + 1);

        // Pathological nesting must be reported, never panic.
        let mut pathological = String::new();
        for _ in 0..5000 {
            pathological.push('[');
        }
        for _ in 0..5000 {
            pathological.push(']');
        }
        let report = analyze(&pathological);
        assert!(!report.valid);
        assert!(report.error.is_some());
    }

    #[test]
    fn object_keys_keep_document_order() {
        // A JSON viewer must not alphabetise the file the user is editing.
        let report = analyze(r#"{"zeta":1,"alpha":2}"#);
        let root = report.root.unwrap();
        assert_eq!(root.children[0].key, "zeta");
        assert_eq!(root.children[1].key, "alpha");
    }

    #[test]
    fn formatting_and_minifying_preserve_values() {
        let source = r#"{"b":2,"a":[1,2]}"#;
        let pretty = format(source, 2).unwrap();
        assert!(pretty.contains("\n  \"b\": 2"));
        assert!(analyze(&pretty).valid);
        assert_eq!(minify(&pretty).unwrap(), source);
    }

    #[test]
    fn formatting_invalid_json_is_an_error_not_a_rewrite() {
        assert!(format("{oops", 2).is_err());
        assert!(minify("{oops").is_err());
    }
}
