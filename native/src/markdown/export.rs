//! Export helpers.
//!
//! HTML export uses `pulldown-cmark`'s own HTML backend so the output follows
//! CommonMark rather than a hand-rolled escape pass. Raw HTML from the document
//! is escaped, never trusted, because the export is meant to be a document, not
//! an active page.

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

/// Only these schemes may appear as an `href` in an exported page.
fn safe_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    if lower.starts_with('#') || lower.starts_with('/') || lower.starts_with("./") {
        return true;
    }
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("file:")
        || !lower.contains(':') // a bare relative path such as `notes/a.md`
}

fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SMART_PUNCTUATION
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// A complete, self-contained HTML document with the app's reading typography.
pub fn to_html(text: &str, title: &str) -> String {
    // Raw HTML from the document is written as literal text, and link targets
    // with a script-capable scheme are neutralised: the export is a document to
    // read, not a page that inherits the document's privileges.
    let parser = Parser::new_ext(text, options()).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(CowStr::from(raw.to_string())),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest = if safe_url(&dest_url) {
                dest_url
            } else {
                CowStr::Borrowed("#")
            };
            Event::Start(Tag::Link {
                link_type,
                dest_url: dest,
                title,
                id,
            })
        }
        other => other,
    });
    let mut body = String::new();
    html::push_html(&mut body, parser);

    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title}</title>\n<style>\n\
:root {{ color-scheme: light dark; }}\n\
body {{ font-family: -apple-system, \"Segoe UI\", system-ui, sans-serif;\n\
  line-height: 1.7; max-width: 46rem; margin: 3rem auto; padding: 0 1.25rem;\n\
  color: #1b1917; background: #fffdf8; }}\n\
@media (prefers-color-scheme: dark) {{ body {{ color: #e9e9ec; background: #0b0b0d; }} }}\n\
pre {{ background: rgba(127,127,127,.14); padding: .9rem 1rem; border-radius: 8px; overflow: auto; }}\n\
code {{ font-family: \"Cascadia Code\", Consolas, monospace; font-size: .92em; }}\n\
blockquote {{ margin: 0; padding-left: 1rem; border-left: 3px solid currentColor; opacity: .85; }}\n\
table {{ border-collapse: collapse; }} td, th {{ border: 1px solid rgba(127,127,127,.35); padding: .35rem .6rem; }}\n\
img {{ max-width: 100%; }}\n\
</style>\n</head>\n<body>\n{body}</body>\n</html>\n",
        title = escape_html(title)
    )
}

/// Convert MDX or Markdown into readable plain text.
///
/// JSX tags are dropped, expressions are kept as their literal source inside
/// `{{ }}`, and Markdown formatting is removed — the result is meant for the
/// clipboard or a plain-text export, never for re-parsing.
pub fn to_plain_text(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let mut trimmed = line.trim_start();
        // Strip block markers.
        while let Some(rest) = trimmed
            .strip_prefix("# ")
            .or_else(|| trimmed.strip_prefix("> "))
            .or_else(|| trimmed.strip_prefix("- "))
            .or_else(|| trimmed.strip_prefix("* "))
        {
            trimmed = rest;
        }
        if trimmed.starts_with("```") {
            continue;
        }
        let mut cleaned = String::new();
        let mut chars = trimmed.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '*' | '_' | '`' => {}
                '<' => {
                    // Drop a JSX/HTML tag up to its closing bracket.
                    for next in chars.by_ref() {
                        if next == '>' {
                            break;
                        }
                    }
                }
                '{' => {
                    cleaned.push_str("{ ");
                    for next in chars.by_ref() {
                        if next == '}' {
                            break;
                        }
                        cleaned.push(next);
                    }
                    cleaned.push_str(" }");
                }
                _ => cleaned.push(ch),
            }
        }
        out.push_str(cleaned.trim_end());
        out.push('\n');
    }
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_export_escapes_document_html() {
        let html = to_html("<script>alert(1)</script>\n\n# Title\n", "t");
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("<h1>Title</h1>"));
    }

    #[test]
    fn script_capable_link_schemes_are_neutralised() {
        let html = to_html("[x](javascript:alert(1)) and [y](notes/a.md)", "t");
        assert!(!html.contains("javascript:"), "{html}");
        assert!(html.contains("notes/a.md"), "relative links are kept");
    }

    #[test]
    fn html_export_has_a_document_skeleton() {
        let html = to_html("# Hi\n", "Notes & Things");
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("<title>Notes &amp; Things</title>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn plain_text_drops_markup_but_keeps_prose() {
        let text = to_plain_text("# Title\n\nHello **world** and `<code>`.\n");
        assert_eq!(text, "Title\n\nHello world and .");
    }

    #[test]
    fn mdx_tags_are_removed_and_expressions_are_kept_readable() {
        let text = to_plain_text("<Callout type=\"warn\">\nTotal: {1 + 1}\n</Callout>\n");
        assert!(!text.contains("Callout"));
        assert!(text.contains("Total: { 1 + 1 }"));
    }

    #[test]
    fn code_fences_do_not_leak_into_plain_text() {
        let text = to_plain_text("before\n```sh\nls -la\n```\nafter\n");
        assert!(text.contains("before"));
        assert!(text.contains("after"));
        assert!(!text.contains("```"));
    }
}
