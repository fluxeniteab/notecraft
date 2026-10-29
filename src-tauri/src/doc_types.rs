//! One registry for every file type NoteCraft opens.
//!
//! A note editor that only understands Markdown leaks Markdown assumptions into
//! file creation, indexing, statistics and the preview. This module is the
//! single place that answers, for any vault-relative path:
//!
//! * what kind of document is this,
//! * which editor language it wants,
//! * whether it is prose (words/characters) or structured data (validity,
//!   line count), and
//! * what a brand new document of that type should contain.
//!
//! Unknown extensions are preserved as ordinary text files and are never
//! reinterpreted or rewritten.

use serde::Serialize;

/// Everything NoteCraft treats as a document rather than an opaque blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocKind {
    Markdown,
    Mdx,
    Json,
    Yaml,
    Html,
    Css,
    JavaScript,
    TypeScript,
    PlainText,
}

/// How the preview should treat the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewMode {
    /// Rendered Markdown/MDX.
    Prose,
    /// Structured JSON tree with diagnostics.
    Json,
    /// Escaped, syntax-labelled source.
    Source,
    /// No preview at all.
    None,
}

/// How the status bar should summarise the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatMode {
    /// Human-language statistics: words and characters.
    Prose,
    /// Structured data: keys, nodes and validity.
    Data,
    /// Source code: lines only.
    Code,
}

/// The validator the editor should run while typing, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Validator {
    Json,
    Yaml,
    Mdx,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocCapabilities {
    pub kind: DocKind,
    pub label: &'static str,
    pub extension: &'static str,
    /// CodeMirror language id the editor should mount.
    pub language: &'static str,
    pub preview: PreviewMode,
    pub stats: StatMode,
    pub validator: Option<Validator>,
    /// Whether the text index should include this document's content.
    pub indexable: bool,
    /// Whether wiki links, tags and tasks are meaningful in this format.
    pub has_links: bool,
}

impl DocKind {
    /// Pick the kind from a path, case-insensitively.
    pub fn from_path(path: &str) -> DocKind {
        let lower = path.to_ascii_lowercase();
        if !lower.contains('.') {
            return DocKind::PlainText;
        }
        let ext = lower.rsplit('.').next().unwrap_or("");
        match ext {
            "md" | "markdown" | "mdown" | "mkd" => DocKind::Markdown,
            "mdx" => DocKind::Mdx,
            "json" | "jsonc" => DocKind::Json,
            "yaml" | "yml" => DocKind::Yaml,
            "html" | "htm" => DocKind::Html,
            "css" => DocKind::Css,
            "js" | "mjs" | "cjs" | "jsx" => DocKind::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => DocKind::TypeScript,
            _ => DocKind::PlainText,
        }
    }

    pub fn capabilities(self) -> DocCapabilities {
        match self {
            DocKind::Markdown => DocCapabilities {
                kind: self,
                label: "Markdown",
                extension: "md",
                language: "markdown",
                preview: PreviewMode::Prose,
                stats: StatMode::Prose,
                validator: None,
                indexable: true,
                has_links: true,
            },
            DocKind::Mdx => DocCapabilities {
                kind: self,
                label: "MDX",
                extension: "mdx",
                language: "mdx",
                preview: PreviewMode::Prose,
                stats: StatMode::Prose,
                validator: Some(Validator::Mdx),
                indexable: true,
                has_links: true,
            },
            DocKind::Json => DocCapabilities {
                kind: self,
                label: "JSON",
                extension: "json",
                language: "json",
                preview: PreviewMode::Json,
                stats: StatMode::Data,
                validator: Some(Validator::Json),
                indexable: true,
                has_links: false,
            },
            DocKind::Yaml => DocCapabilities {
                kind: self,
                label: "YAML",
                extension: "yaml",
                language: "yaml",
                preview: PreviewMode::Source,
                stats: StatMode::Data,
                validator: Some(Validator::Yaml),
                indexable: true,
                has_links: false,
            },
            DocKind::Html => DocCapabilities {
                kind: self,
                label: "HTML",
                extension: "html",
                language: "html",
                preview: PreviewMode::Source,
                stats: StatMode::Code,
                validator: None,
                indexable: true,
                has_links: false,
            },
            DocKind::Css => DocCapabilities {
                kind: self,
                label: "CSS",
                extension: "css",
                language: "css",
                preview: PreviewMode::Source,
                stats: StatMode::Code,
                validator: None,
                indexable: true,
                has_links: false,
            },
            DocKind::JavaScript => DocCapabilities {
                kind: self,
                label: "JavaScript",
                extension: "js",
                language: "javascript",
                preview: PreviewMode::Source,
                stats: StatMode::Code,
                validator: None,
                indexable: true,
                has_links: false,
            },
            DocKind::TypeScript => DocCapabilities {
                kind: self,
                label: "TypeScript",
                extension: "ts",
                language: "typescript",
                preview: PreviewMode::Source,
                stats: StatMode::Code,
                validator: None,
                indexable: true,
                has_links: false,
            },
            DocKind::PlainText => DocCapabilities {
                kind: self,
                label: "Text",
                extension: "txt",
                language: "text",
                preview: PreviewMode::Source,
                stats: StatMode::Code,
                validator: None,
                indexable: true,
                has_links: false,
            },
        }
    }

    /// Starter content for a new document. JSON gets valid JSON rather than a
    /// Markdown heading, which used to make every new `.json` file invalid.
    pub fn template(self, stem: &str) -> String {
        let name = if stem.trim().is_empty() {
            "Untitled"
        } else {
            stem.trim()
        };
        match self {
            DocKind::Markdown => format!("# {name}\n\n"),
            DocKind::Mdx => format!("# {name}\n\n"),
            DocKind::Json => format!(
                "{{\n  \"title\": {}\n}}\n",
                serde_json::to_string(name).unwrap_or_else(|_| "\"Untitled\"".into())
            ),
            DocKind::Yaml => format!("title: {name}\n"),
            DocKind::Html => format!(
                "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <title>{name}</title>\n  </head>\n  <body>\n  </body>\n</html>\n"
            ),
            DocKind::Css => format!("/* {name} */\n\n"),
            DocKind::JavaScript => format!("// {name}\n\n"),
            DocKind::TypeScript => format!("// {name}\n\n"),
            DocKind::PlainText => String::new(),
        }
    }

    /// Every kind the registry knows, for the "new document" menu.
    pub fn all() -> [DocKind; 9] {
        [
            DocKind::Markdown,
            DocKind::Mdx,
            DocKind::Json,
            DocKind::Yaml,
            DocKind::Html,
            DocKind::Css,
            DocKind::JavaScript,
            DocKind::TypeScript,
            DocKind::PlainText,
        ]
    }
}

/// Extensions the generic "open supported files" filter accepts.
pub const DOCUMENT_EXTENSIONS: [&str; 16] = [
    "md", "markdown", "mdx", "json", "yaml", "yml", "html", "htm", "css", "js", "mjs", "cjs",
    "jsx", "ts", "tsx", "txt",
];

/// File name without its directory and without its extension.
pub fn stem_of(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rfind('.') {
        Some(index) if index > 0 => &name[..index],
        _ => name,
    }
}

/// True when the path carries one of the registry's extensions.
pub fn has_document_extension(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let Some(ext) = lower.rsplit('.').next() else {
        return false;
    };
    lower.contains('.') && DOCUMENT_EXTENSIONS.contains(&ext)
}

/// True when the document should be included in the text index. Unknown
/// extensions are deliberately excluded so the index never tries to tokenise
/// binaries.
pub fn is_indexable(path: &str) -> bool {
    has_document_extension(path) && DocKind::from_path(path).capabilities().indexable
}

/// True when wiki links, tags and tasks apply.
pub fn has_links(path: &str) -> bool {
    has_document_extension(path) && DocKind::from_path(path).capabilities().has_links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_detected_case_insensitively() {
        assert_eq!(DocKind::from_path("a/b.MD"), DocKind::Markdown);
        assert_eq!(DocKind::from_path("a/b.mdx"), DocKind::Mdx);
        assert_eq!(DocKind::from_path("a/b.JSON"), DocKind::Json);
        assert_eq!(DocKind::from_path("a/b.yml"), DocKind::Yaml);
        assert_eq!(DocKind::from_path("a/b.tsx"), DocKind::TypeScript);
        assert_eq!(DocKind::from_path("a/b.bin"), DocKind::PlainText);
        assert_eq!(DocKind::from_path("README"), DocKind::PlainText);
    }

    #[test]
    fn only_prose_formats_have_links() {
        assert!(has_links("note.md"));
        assert!(has_links("page.mdx"));
        assert!(!has_links("data.json"));
        assert!(!has_links("style.css"));
        assert!(!has_links("image.png"));
    }

    #[test]
    fn json_template_is_valid_json() {
        let text = DocKind::Json.template("report");
        assert!(serde_json::from_str::<serde_json::Value>(&text).is_ok());
        assert!(text.contains("\"title\""));
    }

    #[test]
    fn markdown_template_keeps_a_heading() {
        assert_eq!(DocKind::Markdown.template("Ideas"), "# Ideas\n\n");
    }

    #[test]
    fn binary_files_are_not_indexable() {
        assert!(is_indexable("notes/a.md"));
        assert!(is_indexable("data/a.json"));
        assert!(!is_indexable("photo.png"));
        assert!(!is_indexable("archive.zip"));
    }

    #[test]
    fn every_kind_reports_its_extension() {
        for kind in DocKind::all() {
            let caps = kind.capabilities();
            assert_eq!(DocKind::from_path(&format!("x.{}", caps.extension)), kind);
            assert!(!caps.language.is_empty());
        }
    }
}
