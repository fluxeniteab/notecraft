//! Document kinds and their capabilities.
//!
//! The archived application kept this table in `src/lib/documentTypes.ts` and
//! mirrored it in Rust. The native build owns the single Rust copy: every part
//! of the UI asks this module what a file is and what may be done with it, so
//! JSON never gets Markdown statistics and Markdown never gets a JSON tree.

use std::path::Path;

/// A supported document kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Markdown,
    Mdx,
    Json,
    Yaml,
    Html,
    Css,
    JavaScript,
    TypeScript,
    Text,
    Unknown,
}

/// Which family of statistics a document reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stats {
    /// Words and characters, plus reading-oriented affordances.
    Prose,
    /// Lines and characters; no word count.
    Data,
    /// Lines only.
    Code,
}

/// The validator a document can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validator {
    Json,
    Yaml,
    None,
}

/// Syntax highlighting dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Markdown,
    Mdx,
    Json,
    Yaml,
    Html,
    Css,
    Code,
    Plain,
}

/// Everything the UI needs to know about a document kind.
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub kind: Kind,
    pub label: &'static str,
    /// Short extension shown in the status bar and the save dialog.
    pub extension: &'static str,
    pub syntax: Syntax,
    pub stats: Stats,
    pub validator: Validator,
    /// The editor uses a monospace face and a line-number gutter.
    pub mono: bool,
    /// A rendered reading view is meaningful for this kind.
    pub preview: bool,
    /// Content is included in full-text search.
    pub indexable: bool,
    /// Frontmatter-style metadata is scanned for tags and properties.
    pub metadata: bool,
}

const MARKDOWN: Capabilities = Capabilities {
    kind: Kind::Markdown,
    label: "Markdown",
    extension: "md",
    syntax: Syntax::Markdown,
    stats: Stats::Prose,
    validator: Validator::None,
    mono: false,
    preview: true,
    indexable: true,
    metadata: true,
};

const MDX: Capabilities = Capabilities {
    kind: Kind::Mdx,
    label: "MDX",
    extension: "mdx",
    syntax: Syntax::Mdx,
    stats: Stats::Prose,
    validator: Validator::None,
    mono: false,
    preview: true,
    indexable: true,
    metadata: true,
};

const JSON: Capabilities = Capabilities {
    kind: Kind::Json,
    label: "JSON",
    extension: "json",
    syntax: Syntax::Json,
    stats: Stats::Data,
    validator: Validator::Json,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const YAML: Capabilities = Capabilities {
    kind: Kind::Yaml,
    label: "YAML",
    extension: "yaml",
    syntax: Syntax::Yaml,
    stats: Stats::Data,
    validator: Validator::Yaml,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const HTML: Capabilities = Capabilities {
    kind: Kind::Html,
    label: "HTML",
    extension: "html",
    syntax: Syntax::Html,
    stats: Stats::Code,
    validator: Validator::None,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const CSS: Capabilities = Capabilities {
    kind: Kind::Css,
    label: "CSS",
    extension: "css",
    syntax: Syntax::Css,
    stats: Stats::Code,
    validator: Validator::None,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const JAVASCRIPT: Capabilities = Capabilities {
    kind: Kind::JavaScript,
    label: "JavaScript",
    extension: "js",
    syntax: Syntax::Code,
    stats: Stats::Code,
    validator: Validator::None,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const TYPESCRIPT: Capabilities = Capabilities {
    kind: Kind::TypeScript,
    label: "TypeScript",
    extension: "ts",
    syntax: Syntax::Code,
    stats: Stats::Code,
    validator: Validator::None,
    mono: true,
    preview: false,
    indexable: true,
    metadata: false,
};

const TEXT: Capabilities = Capabilities {
    kind: Kind::Text,
    label: "Plain text",
    extension: "txt",
    syntax: Syntax::Plain,
    stats: Stats::Prose,
    validator: Validator::None,
    mono: false,
    preview: false,
    indexable: true,
    metadata: false,
};

const UNKNOWN: Capabilities = Capabilities {
    kind: Kind::Unknown,
    label: "File",
    extension: "",
    syntax: Syntax::Plain,
    stats: Stats::Code,
    validator: Validator::None,
    mono: true,
    preview: false,
    indexable: false,
    metadata: false,
};

/// Every kind the application can create, in menu order.
pub const CREATABLE: &[Capabilities] = &[
    MARKDOWN, MDX, JSON, YAML, TEXT, HTML, CSS, JAVASCRIPT, TYPESCRIPT,
];

impl Kind {
    pub fn capabilities(self) -> &'static Capabilities {
        match self {
            Kind::Markdown => &MARKDOWN,
            Kind::Mdx => &MDX,
            Kind::Json => &JSON,
            Kind::Yaml => &YAML,
            Kind::Html => &HTML,
            Kind::Css => &CSS,
            Kind::JavaScript => &JAVASCRIPT,
            Kind::TypeScript => &TYPESCRIPT,
            Kind::Text => &TEXT,
            Kind::Unknown => &UNKNOWN,
        }
    }

    /// Classify a path.
    ///
    /// A file with no extension at all (`README`, `Makefile`) is plain text, not
    /// an unknown format: it is perfectly readable and must not be offered a
    /// Markdown preview or a validator it does not have.
    pub fn from_path(path: &str) -> Kind {
        match extension_of(path) {
            Some(ext) => kind_from_extension(&ext),
            None => Kind::Text,
        }
    }
}

pub fn extension_of(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
}

pub fn kind_from_extension(ext: &str) -> Kind {
    match ext.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Kind::Markdown,
        "mdx" => Kind::Mdx,
        "json" => Kind::Json,
        "yaml" | "yml" => Kind::Yaml,
        "html" | "htm" => Kind::Html,
        "css" => Kind::Css,
        "js" | "mjs" | "cjs" | "jsx" => Kind::JavaScript,
        "ts" | "mts" | "cts" | "tsx" => Kind::TypeScript,
        "txt" | "text" | "" => Kind::Text,
        _ => Kind::Unknown,
    }
}

/// The capabilities for a path.
pub fn capabilities_for(path: &str) -> &'static Capabilities {
    Kind::from_path(path).capabilities()
}

/// Add the default `.md` extension only when the user gave none.
///
/// An explicit `.mdx`, `.json` or any other extension is preserved verbatim:
/// silently rewriting `notes.mdx` to `notes.md` was a real bug in the Tauri
/// build.
pub fn ensure_extension(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "Untitled.md".to_string();
    }
    if extension_of(trimmed).is_some() {
        trimmed.to_string()
    } else {
        format!("{trimmed}.md")
    }
}

/// A starting body for a newly created document.
pub fn template_for(kind: Kind) -> &'static str {
    match kind {
        Kind::Markdown => "# Untitled\n\n",
        Kind::Mdx => "# Untitled\n\n<Note>\n  MDX keeps JSX and expressions: {1 + 1}.\n</Note>\n",
        Kind::Json => "{\n  \n}\n",
        Kind::Yaml => "# YAML\n",
        Kind::Html => "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <title>Untitled</title>\n  </head>\n  <body>\n    \n  </body>\n</html>\n",
        Kind::Css => "/* styles */\n",
        Kind::JavaScript => "",
        Kind::TypeScript => "",
        Kind::Text | Kind::Unknown => "",
    }
}

/// Whether a file may be shown in the explorer.
pub fn is_supported_document(name: &str) -> bool {
    let kind = Kind::from_path(name);
    kind != Kind::Unknown || extension_of(name).is_none()
}

/// Mime/format label for the HTML exporter.
pub fn export_extension(kind: Kind) -> &'static str {
    match kind {
        Kind::Markdown => "md",
        Kind::Mdx => "mdx",
        Kind::Json => "json",
        Kind::Yaml => "yaml",
        Kind::Html => "html",
        Kind::Css => "css",
        Kind::JavaScript => "js",
        Kind::TypeScript => "ts",
        Kind::Text => "txt",
        Kind::Unknown => "txt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_extensions_are_preserved() {
        assert_eq!(ensure_extension("notes.mdx"), "notes.mdx");
        assert_eq!(ensure_extension("data.json"), "data.json");
        assert_eq!(ensure_extension("styles.css"), "styles.css");
        assert_eq!(ensure_extension("plain"), "plain.md");
        assert_eq!(ensure_extension("  "), "Untitled.md");
    }

    #[test]
    fn mdx_and_json_are_distinct_kinds() {
        assert_eq!(Kind::from_path("a.mdx"), Kind::Mdx);
        assert_eq!(Kind::from_path("a.json"), Kind::Json);
        assert_eq!(Kind::from_path("a.MDX"), Kind::Mdx);
        assert_eq!(Kind::from_path("README"), Kind::Text);
        assert!(Kind::Mdx.capabilities().preview);
        assert!(!Kind::Json.capabilities().preview);
        assert_eq!(Kind::Json.capabilities().stats, Stats::Data);
        assert_eq!(Kind::Markdown.capabilities().stats, Stats::Prose);
    }

    #[test]
    fn only_structured_formats_declare_a_validator() {
        assert_eq!(Kind::Json.capabilities().validator, Validator::Json);
        assert_eq!(Kind::Yaml.capabilities().validator, Validator::Yaml);
        assert_eq!(Kind::Markdown.capabilities().validator, Validator::None);
        // A CSS file is code, not data.
        assert_eq!(Kind::Css.capabilities().stats, Stats::Code);
    }

    #[test]
    fn every_creatable_kind_has_a_distinct_extension() {
        let mut seen = std::collections::HashSet::new();
        for caps in CREATABLE {
            assert!(
                seen.insert(caps.extension),
                "duplicate extension {}",
                caps.extension
            );
        }
    }
}
