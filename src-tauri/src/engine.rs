//! NoteCraft document engine.
//!
//! One self-contained renderer for every text note format the app opens:
//!
//! * **Markdown** — headings, paragraphs, nested lists and task lists,
//!   blockquotes, callouts, fenced code, tables, rules, footnotes, inline
//!   emphasis/strikethrough/highlight, links, images and `[[wiki links]]`.
//! * **MDX** — the same blocks plus JSX component blocks, `{expressions}` and
//!   ESM statements. Component blocks are surfaced as cards instead of being
//!   swallowed or rendered as broken HTML.
//! * **JSON** — parsed and converted to a readable outline, which is then
//!   rendered by the Markdown pipeline.
//!
//! The module uses the standard library only, so it can be compiled and unit
//! tested in isolation. Every byte of user text is HTML-escaped here; callers
//! may still sanitise the result as defence in depth.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dialect {
    Markdown,
    Mdx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Markdown,
    Mdx,
    Json,
}

impl Kind {
    /// Pick the engine dialect from a vault-relative path.
    pub fn from_path(path: &str) -> Kind {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".mdx") {
            Kind::Mdx
        } else if lower.ends_with(".json") {
            Kind::Json
        } else {
            Kind::Markdown
        }
    }
}

/// Render any supported document to preview HTML.
pub fn render_document(source: &str, kind: Kind) -> String {
    match kind {
        Kind::Json => render(&json_to_text(source), Dialect::Markdown),
        Kind::Mdx => render(source, Dialect::Mdx),
        Kind::Markdown => render(source, Dialect::Markdown),
    }
}

// ---------------------------------------------------------------- helpers ---

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    push_escaped(&mut out, text);
    out
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        push_escaped_char(out, ch);
    }
}

fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => out.push(ch),
    }
}

fn escape_attr(text: &str) -> String {
    escape_html(text)
}

fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// Remove up to `n` leading spaces, used to dedent list/quote contents.
fn dedent(line: &str, n: usize) -> String {
    let mut skip = 0;
    for ch in line.chars() {
        if skip >= n || (ch != ' ' && ch != '\t') {
            break;
        }
        skip += 1;
    }
    line.chars().skip(skip).collect()
}

fn fence_marker(line: &str) -> Option<(char, usize, String)> {
    let trimmed = line.trim_start();
    if indent_of(line) > 3 {
        return None;
    }
    let ch = trimmed.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|c| *c == ch).count();
    if len < 3 {
        return None;
    }
    let info: String = trimmed.chars().skip(len).collect();
    Some((ch, len, info.trim().to_string()))
}

fn heading_marker(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    if indent_of(line) > 3 {
        return None;
    }
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest: String = trimmed.chars().skip(level).collect();
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let mut text = rest.trim().to_string();
    while text.ends_with('#') {
        text.pop();
    }
    Some((level, text.trim().to_string()))
}

fn is_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if indent_of(line) > 3 || trimmed.len() < 3 {
        return false;
    }
    let first = match trimmed.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if first != '-' && first != '*' && first != '_' {
        return false;
    }
    trimmed.chars().all(|c| c == first)
}

fn table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut body = trimmed;
    if body.starts_with('|') {
        body = &body[1..];
    }
    if body.ends_with('|') {
        body = &body[..body.len() - 1];
    }
    body.split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn is_table_delimiter(line: &str) -> bool {
    if !line.contains('-') {
        return false;
    }
    let cells = table_cells(line);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|cell| {
        let body = cell.trim_start_matches(':').trim_end_matches(':');
        !body.is_empty() && body.chars().all(|c| c == '-')
    })
}

fn table_align(line: &str) -> Vec<&'static str> {
    table_cells(line)
        .iter()
        .map(|cell| {
            let left = cell.starts_with(':');
            let right = cell.ends_with(':');
            match (left, right) {
                (true, true) => "center",
                (true, false) => "left",
                (false, true) => "right",
                _ => "",
            }
        })
        .collect()
}

/// `(ordered, marker indent, byte index where the item content starts)`
fn list_marker(line: &str) -> Option<(bool, usize, usize)> {
    let indent = indent_of(line);
    if indent > 8 {
        return None;
    }
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let mut chars = trimmed.char_indices();
    let (_, first) = chars.next()?;
    let marker_byte = line.len() - trimmed.len();

    if first == '-' || first == '*' || first == '+' {
        let rest = &trimmed[first.len_utf8()..];
        if rest.is_empty() {
            return Some((false, indent, line.len()));
        }
        if rest.starts_with(' ') {
            let spaces = 1 + rest[1..].chars().take_while(|c| *c == ' ').count();
            return Some((false, indent, marker_byte + first.len_utf8() + spaces));
        }
        return None;
    }
    if first.is_ascii_digit() {
        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 9 {
            return None;
        }
        let rest = &trimmed[digits..];
        let delim = rest.chars().next()?;
        if delim != '.' && delim != ')' {
            return None;
        }
        let after = &rest[delim.len_utf8()..];
        if after.is_empty() {
            return Some((true, indent, line.len()));
        }
        if !after.starts_with(' ') {
            return None;
        }
        let spaces = 1 + after[1..].chars().take_while(|c| *c == ' ').count();
        return Some((
            true,
            indent,
            marker_byte + digits + delim.len_utf8() + spaces,
        ));
    }
    None
}

fn footnote_definition(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("[^") {
        return None;
    }
    let close = trimmed.find("]:")?;
    let id = trimmed[2..close].trim().to_string();
    if id.is_empty() {
        return None;
    }
    let body = trimmed[close + 2..].trim().to_string();
    Some((id, body))
}

fn looks_like_jsx(line: &str) -> bool {
    let trimmed = line.trim_start();
    let mut chars = trimmed.chars();
    if chars.next() != Some('<') {
        return false;
    }
    match chars.next() {
        Some(c) => c.is_ascii_alphabetic() || c == '/',
        None => false,
    }
}

fn jsx_name(line: &str) -> String {
    let trimmed = line.trim_start();
    let rest = trimmed.trim_start_matches('<').trim_start_matches('/');
    rest.chars()
        .take_while(|c| {
            c.is_ascii_alphanumeric() || *c == '.' || *c == ':' || *c == '-' || *c == '_'
        })
        .collect()
}

// ------------------------------------------------------------ entry point ---

/// Render Markdown or MDX source to HTML.
pub fn render(source: &str, dialect: Dialect) -> String {
    let normalised = source.replace("\r\n", "\n").replace('\r', "\n");
    let (front, body) = split_frontmatter(&normalised);
    let mut ctx = Ctx {
        dialect,
        footnotes: Vec::new(),
        heading_ids: Vec::new(),
    };
    let mut out = String::with_capacity(body.len() + 256);
    if !front.trim().is_empty() {
        render_frontmatter(&front, &mut out);
    }
    let lines: Vec<&str> = body.lines().collect();
    let mut index = 0;
    render_blocks(&lines, &mut index, &mut ctx, &mut out);
    render_footnotes(&ctx, &mut out);
    out
}

struct Ctx {
    dialect: Dialect,
    footnotes: Vec<(String, String)>,
    heading_ids: Vec<String>,
}

fn split_frontmatter(source: &str) -> (String, String) {
    if !source.starts_with("---") {
        return (String::new(), source.to_string());
    }
    let mut lines = source.lines();
    let _opening = lines.next();
    let mut front = String::new();
    let mut rest = Vec::new();
    let mut closed = false;
    for line in lines {
        if !closed && line.trim() == "---" {
            closed = true;
            continue;
        }
        if closed {
            rest.push(line);
        } else {
            front.push_str(line);
            front.push('\n');
        }
    }
    if !closed {
        return (String::new(), source.to_string());
    }
    (front, rest.join("\n"))
}

fn render_frontmatter(front: &str, out: &mut String) {
    out.push_str(
        "<div class=\"nc-frontmatter\"><div class=\"nc-frontmatter-title\">Frontmatter</div><dl>",
    );
    for line in front.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('-') {
            out.push_str("<dd>");
            push_escaped(out, trimmed);
            out.push_str("</dd>");
            continue;
        }
        match trimmed.split_once(':') {
            Some((key, value)) => {
                out.push_str("<dt>");
                push_escaped(out, key.trim());
                out.push_str("</dt><dd>");
                push_escaped(out, value.trim());
                out.push_str("</dd>");
            }
            None => {
                out.push_str("<dt>");
                push_escaped(out, trimmed);
                out.push_str("</dt>");
            }
        }
    }
    out.push_str("</dl></div>");
}

fn render_footnotes(ctx: &Ctx, out: &mut String) {
    if ctx.footnotes.is_empty() {
        return;
    }
    out.push_str("<section class=\"nc-footnotes\"><ol>");
    for (id, body) in &ctx.footnotes {
        out.push_str("<li id=\"fn-");
        push_escaped(out, &slugify(id));
        out.push_str("\">");
        push_escaped(out, body);
        out.push_str("</li>");
    }
    out.push_str("</ol></section>");
}

// --------------------------------------------------------------- blocks -----

fn render_blocks(lines: &[&str], index: &mut usize, ctx: &mut Ctx, out: &mut String) {
    while *index < lines.len() {
        let line = lines[*index];
        if line.trim().is_empty() {
            *index += 1;
            continue;
        }

        if let Some((marker, len, info)) = fence_marker(line) {
            render_fence(lines, index, marker, len, &info, out);
            continue;
        }
        if let Some((level, text)) = heading_marker(line) {
            let id = unique_id(&slugify(&strip_inline(text.as_str())), ctx);
            out.push_str(&format!("<h{0} id=\"{1}\">", level, escape_attr(&id)));
            out.push_str(&render_inline(&text, ctx));
            out.push_str(&format!("</h{0}>", level));
            *index += 1;
            continue;
        }
        if is_rule(line) {
            out.push_str("<hr />");
            *index += 1;
            continue;
        }
        if line.trim_start().starts_with('>') && indent_of(line) <= 3 {
            render_quote(lines, index, ctx, out);
            continue;
        }
        if line.contains('|') && *index + 1 < lines.len() && is_table_delimiter(lines[*index + 1]) {
            render_table(lines, index, ctx, out);
            continue;
        }
        if list_marker(line).is_some() {
            render_list(lines, index, ctx, out);
            continue;
        }
        if let Some((id, body)) = footnote_definition(line) {
            let mut text = body;
            *index += 1;
            while *index < lines.len() {
                let next = lines[*index];
                if next.trim().is_empty() || indent_of(next) == 0 {
                    break;
                }
                text.push(' ');
                text.push_str(next.trim());
                *index += 1;
            }
            ctx.footnotes.push((id, text));
            continue;
        }
        if ctx.dialect == Dialect::Mdx {
            let trimmed = line.trim_start();
            if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
                *index += 1;
                continue;
            }
            if looks_like_jsx(trimmed) {
                render_jsx_block(lines, index, out);
                continue;
            }
            if trimmed.starts_with('{') {
                let mut source = String::new();
                while *index < lines.len() && !lines[*index].trim().is_empty() {
                    if !source.is_empty() {
                        source.push('\n');
                    }
                    source.push_str(lines[*index].trim());
                    *index += 1;
                }
                out.push_str("<div class=\"nc-mdx-block\"><div class=\"nc-mdx-label\">expression</div><pre class=\"nc-mdx-source\">");
                push_escaped(out, &source);
                out.push_str("</pre></div>");
                continue;
            }
        }
        if indent_of(line) >= 4 {
            let mut code = String::new();
            while *index < lines.len() {
                let next = lines[*index];
                if next.trim().is_empty() {
                    code.push('\n');
                    *index += 1;
                    continue;
                }
                if indent_of(next) < 4 {
                    break;
                }
                code.push_str(&dedent(next, 4));
                code.push('\n');
                *index += 1;
            }
            out.push_str("<pre><code>");
            push_escaped(out, &code);
            out.push_str("</code></pre>");
            continue;
        }
        render_paragraph(lines, index, ctx, out);
    }
}

fn render_fence(
    lines: &[&str],
    index: &mut usize,
    marker: char,
    len: usize,
    info: &str,
    out: &mut String,
) {
    *index += 1;
    let mut code = String::new();
    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim_start();
        let run = trimmed.chars().take_while(|c| *c == marker).count();
        if run >= len && trimmed.chars().skip(run).all(|c| c == ' ') {
            *index += 1;
            break;
        }
        code.push_str(line);
        code.push('\n');
        *index += 1;
    }
    let language = info.split_whitespace().next().unwrap_or("");
    out.push_str("<pre><code");
    if !language.is_empty() {
        out.push_str(" class=\"language-");
        push_escaped(out, language);
        out.push('"');
    }
    out.push('>');
    push_escaped(out, &code);
    out.push_str("</code></pre>");
}

fn render_quote(lines: &[&str], index: &mut usize, ctx: &mut Ctx, out: &mut String) {
    let mut inner: Vec<String> = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('>') {
            inner.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
            *index += 1;
        } else if line.trim().is_empty() {
            if *index + 1 < lines.len() && lines[*index + 1].trim_start().starts_with('>') {
                inner.push(String::new());
                *index += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let mut start = 0;
    let mut callout: Option<String> = None;
    if let Some(first) = inner.first().cloned() {
        if first.starts_with("[!") {
            if let Some(end) = first.find(']') {
                let kind = first[2..end].trim().to_ascii_lowercase();
                if !kind.is_empty() {
                    callout = Some(kind);
                    let rest = first[end + 1..].trim().to_string();
                    if rest.is_empty() {
                        start = 1;
                    } else {
                        inner[0] = rest;
                    }
                }
            }
        }
    }

    let body: Vec<&str> = inner[start..].iter().map(|s| s.as_str()).collect();
    match callout {
        Some(kind) => {
            out.push_str("<div class=\"nc-callout nc-callout-");
            push_escaped(out, &kind);
            out.push_str("\"><div class=\"nc-callout-title\">");
            push_escaped(out, &kind.to_uppercase());
            out.push_str("</div><div class=\"nc-callout-body\">");
            let mut cursor = 0;
            render_blocks(&body, &mut cursor, ctx, out);
            out.push_str("</div></div>");
        }
        None => {
            out.push_str("<blockquote>");
            let mut cursor = 0;
            render_blocks(&body, &mut cursor, ctx, out);
            out.push_str("</blockquote>");
        }
    }
}

fn render_table(lines: &[&str], index: &mut usize, ctx: &mut Ctx, out: &mut String) {
    let header = table_cells(lines[*index]);
    let align = table_align(lines[*index + 1]);
    *index += 2;

    out.push_str("<table><thead><tr>");
    for (position, cell) in header.iter().enumerate() {
        let alignment = align.get(position).copied().unwrap_or("");
        out.push_str("<th");
        if !alignment.is_empty() {
            out.push_str(&format!(" align=\"{}\"", alignment));
        }
        out.push('>');
        out.push_str(&render_inline(cell, ctx));
        out.push_str("</th>");
    }
    out.push_str("</tr></thead><tbody>");

    while *index < lines.len() {
        let line = lines[*index];
        if line.trim().is_empty() || !line.contains('|') {
            break;
        }
        let cells = table_cells(line);
        out.push_str("<tr>");
        for (position, cell) in cells.iter().enumerate() {
            let alignment = align.get(position).copied().unwrap_or("");
            out.push_str("<td");
            if !alignment.is_empty() {
                out.push_str(&format!(" align=\"{}\"", alignment));
            }
            out.push('>');
            out.push_str(&render_inline(cell, ctx));
            out.push_str("</td>");
        }
        out.push_str("</tr>");
        *index += 1;
    }
    out.push_str("</tbody></table>");
}

fn render_list(lines: &[&str], index: &mut usize, ctx: &mut Ctx, out: &mut String) {
    let first = match list_marker(lines[*index]) {
        Some(marker) => marker,
        None => return,
    };
    let ordered = first.0;
    let base = first.1;
    let mut items: Vec<Vec<String>> = Vec::new();

    while *index < lines.len() {
        let line = lines[*index];
        if line.trim().is_empty() {
            let mut look = *index + 1;
            while look < lines.len() && lines[look].trim().is_empty() {
                look += 1;
            }
            if look >= lines.len() {
                break;
            }
            let next = lines[look];
            let next_indent = indent_of(next);
            let sibling_item =
                matches!(list_marker(next), Some((_, marker_indent, _)) if marker_indent >= base);
            if next_indent > base || sibling_item {
                if let Some(last) = items.last_mut() {
                    last.push(String::new());
                }
                *index += 1;
                continue;
            }
            break;
        }
        let indent = indent_of(line);
        match list_marker(line) {
            Some((_, marker_indent, content_start)) => {
                if marker_indent < base {
                    break;
                }
                if marker_indent == base {
                    items.push(vec![line[content_start..].to_string()]);
                    *index += 1;
                } else if let Some(last) = items.last_mut() {
                    last.push(dedent(line, (base + 2).min(indent)));
                    *index += 1;
                } else {
                    break;
                }
            }
            None => {
                if indent > base {
                    if let Some(last) = items.last_mut() {
                        last.push(dedent(line, (base + 2).min(indent)));
                        *index += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    let tag = if ordered { "ol" } else { "ul" };
    out.push_str("<");
    out.push_str(tag);
    out.push_str(" class=\"nc-list\">");
    for item in items {
        let mut item_lines: Vec<&str> = item.iter().map(|line| line.as_str()).collect();
        let mut task: Option<bool> = None;
        if let Some(first_line) = item_lines.first().copied() {
            let trimmed = first_line.trim_start();
            if trimmed.len() >= 4
                && trimmed.starts_with('[')
                && (trimmed.starts_with("[ ] ")
                    || trimmed.starts_with("[x] ")
                    || trimmed.starts_with("[X] "))
            {
                task = Some(trimmed.as_bytes()[1] == b'x' || trimmed.as_bytes()[1] == b'X');
                item_lines[0] = &trimmed[4..];
            }
        }

        out.push_str("<li");
        match task {
            Some(true) => out.push_str(" class=\"nc-task nc-task-done\""),
            Some(false) => out.push_str(" class=\"nc-task\""),
            None => {}
        }
        out.push('>');
        match task {
            Some(true) => out.push_str("<input type=\"checkbox\" checked disabled />"),
            Some(false) => out.push_str("<input type=\"checkbox\" disabled />"),
            None => {}
        }

        let mut inner = String::new();
        let mut cursor = 0;
        render_blocks(&item_lines, &mut cursor, ctx, &mut inner);
        let trimmed_inner = inner.trim();
        let single_paragraph = trimmed_inner.starts_with("<p>")
            && trimmed_inner.ends_with("</p>")
            && !trimmed_inner[3..trimmed_inner.len() - 4].contains("<p>");
        if single_paragraph {
            out.push_str(&trimmed_inner[3..trimmed_inner.len() - 4]);
        } else {
            out.push_str(trimmed_inner);
        }
        out.push_str("</li>");
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn render_jsx_block(lines: &[&str], index: &mut usize, out: &mut String) {
    let first = lines[*index];
    let name = jsx_name(first);
    let mut source = String::new();
    while *index < lines.len() && !lines[*index].trim().is_empty() {
        if !source.is_empty() {
            source.push('\n');
        }
        source.push_str(lines[*index].trim_end());
        *index += 1;
    }
    out.push_str("<div class=\"nc-mdx-block\"><div class=\"nc-mdx-label\">");
    push_escaped(out, if name.is_empty() { "component" } else { &name });
    out.push_str("</div><pre class=\"nc-mdx-source\">");
    push_escaped(out, &source);
    out.push_str("</pre></div>");
}

fn render_paragraph(lines: &[&str], index: &mut usize, ctx: &mut Ctx, out: &mut String) {
    let mut buffer = String::new();
    while *index < lines.len() {
        let line = lines[*index];
        if line.trim().is_empty() {
            break;
        }
        if !buffer.is_empty() && starts_block(line) {
            break;
        }
        if !buffer.is_empty() {
            buffer.push('\n');
        }
        buffer.push_str(line.trim_end());
        *index += 1;
    }
    if buffer.trim().is_empty() {
        return;
    }
    out.push_str("<p>");
    out.push_str(&render_inline(&buffer, ctx));
    out.push_str("</p>");
}

fn starts_block(line: &str) -> bool {
    let trimmed = line.trim_start();
    if indent_of(line) >= 4 {
        return true;
    }
    if fence_marker(line).is_some() || heading_marker(line).is_some() || is_rule(line) {
        return true;
    }
    if trimmed.starts_with('>') || list_marker(line).is_some() {
        return true;
    }
    if footnote_definition(line).is_some() || looks_like_jsx(trimmed) {
        return true;
    }
    false
}

// --------------------------------------------------------------- inline -----

fn strip_inline(text: &str) -> String {
    text.chars()
        .filter(|c| *c != '*' && *c != '_' && *c != '`' && *c != '~' && *c != '=')
        .collect()
}

fn render_inline(text: &str, ctx: &mut Ctx) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(chars.len() + 32);
    inline_into(&chars, 0, chars.len(), ctx, &mut out, 0);
    out
}

fn find_sequence(
    chars: &[char],
    from: usize,
    end: usize,
    needle: char,
    count: usize,
) -> Option<usize> {
    let mut position = from;
    while position + count <= end {
        if chars[position] == needle {
            let run = chars[position..end]
                .iter()
                .take_while(|c| **c == needle)
                .count();
            if run >= count {
                return Some(position);
            }
            position += run;
        } else {
            position += 1;
        }
    }
    None
}

fn find_char(chars: &[char], from: usize, end: usize, needle: char) -> Option<usize> {
    (from..end).find(|position| chars[*position] == needle)
}

fn safe_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    let blocked = lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("data:text/html");
    if blocked {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn split_destination(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if let Some(space) = trimmed.find(char::is_whitespace) {
        let url = trimmed[..space].to_string();
        let title = trimmed[space..].trim();
        let title = title
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_string();
        if title.is_empty() {
            (url, None)
        } else {
            (url, Some(title))
        }
    } else {
        (trimmed.to_string(), None)
    }
}

fn wikilink_html(inner: &str) -> String {
    let (target_part, alias) = match inner.find('|') {
        Some(position) => (inner[..position].trim(), Some(inner[position + 1..].trim())),
        None => (inner.trim(), None),
    };
    let mut target = target_part;
    let mut heading: Option<&str> = None;
    let mut block: Option<&str> = None;
    if let Some(position) = target_part.find('#') {
        target = target_part[..position].trim();
        let rest = &target_part[position + 1..];
        if let Some(stripped) = rest.strip_prefix('^') {
            block = Some(stripped.trim());
        } else {
            heading = Some(rest.trim());
        }
    }
    if target.is_empty() && heading.is_none() && block.is_none() {
        return format!("[[{}]]", escape_html(inner));
    }
    let label = alias
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| {
            if target.is_empty() {
                heading.or(block).unwrap_or("").to_string()
            } else {
                target.to_string()
            }
        });

    let mut href = if target.is_empty() {
        String::new()
    } else {
        let lower = target.to_ascii_lowercase();
        if lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".mdx") {
            target.to_string()
        } else {
            format!("{}.md", target)
        }
    };
    if let Some(section) = heading {
        href.push('#');
        href.push_str(&section.replace(' ', "%20"));
    } else if let Some(anchor) = block {
        href.push('#');
        href.push_str(&format!("^{}", anchor.replace(' ', "%20")));
    }

    format!(
        "<a class=\"nc-wikilink\" href=\"{}\">{}</a>",
        escape_attr(&href),
        escape_html(&label)
    )
}

#[allow(clippy::too_many_arguments)]
fn inline_into(
    chars: &[char],
    start: usize,
    end: usize,
    ctx: &mut Ctx,
    out: &mut String,
    depth: usize,
) {
    if depth > 12 {
        push_escaped(out, &chars[start..end].iter().collect::<String>());
        return;
    }
    let mut i = start;
    while i < end {
        let ch = chars[i];
        match ch {
            '\\' => {
                if i + 1 < end {
                    let next = chars[i + 1];
                    if next == '\n' {
                        out.push_str("<br>");
                        i += 2;
                        continue;
                    }
                    if next.is_ascii_punctuation() {
                        push_escaped_char(out, next);
                        i += 2;
                        continue;
                    }
                }
                out.push('\\');
                i += 1;
            }
            '`' => {
                let run = chars[i..end].iter().take_while(|c| **c == '`').count();
                if let Some(close) = find_sequence(chars, i + run, end, '`', run) {
                    let raw: String = chars[i + run..close].iter().collect();
                    let content = if raw.starts_with(' ') && raw.ends_with(' ') && raw.len() > 1 {
                        raw[1..raw.len() - 1].to_string()
                    } else {
                        raw
                    };
                    out.push_str("<code>");
                    push_escaped(out, &content);
                    out.push_str("</code>");
                    i = close + run;
                } else {
                    out.push('`');
                    i += 1;
                }
            }
            '!' if i + 1 < end && chars[i + 1] == '[' => {
                if let Some(consumed_end) = parse_image(chars, i, end, ctx, out) {
                    i = consumed_end;
                } else {
                    out.push('!');
                    i += 1;
                }
            }
            '[' => {
                if i + 1 < end && chars[i + 1] == '[' {
                    if let Some(close) = find_pair(chars, i + 2, end, ']', ']') {
                        let inner: String = chars[i + 2..close].iter().collect();
                        out.push_str(&wikilink_html(&inner));
                        i = close + 2;
                        continue;
                    }
                }
                if i + 2 < end && chars[i + 1] == '^' {
                    if let Some(close) = find_char(chars, i + 2, end, ']') {
                        let id: String = chars[i + 2..close].iter().collect();
                        out.push_str("<sup class=\"nc-fn-ref\"><a href=\"#fn-");
                        push_escaped(out, &slugify(&id));
                        out.push_str("\">");
                        push_escaped(out, &id);
                        out.push_str("</a></sup>");
                        i = close + 1;
                        continue;
                    }
                }
                if let Some(consumed_end) = parse_link(chars, i, end, ctx, out) {
                    i = consumed_end;
                } else {
                    out.push('[');
                    i += 1;
                }
            }
            '<' => {
                if let Some(close) = find_char(chars, i + 1, end, '>') {
                    let inner: String = chars[i + 1..close].iter().collect();
                    let lower = inner.to_ascii_lowercase();
                    if lower.starts_with("http://")
                        || lower.starts_with("https://")
                        || lower.starts_with("mailto:")
                    {
                        out.push_str("<a href=\"");
                        out.push_str(&escape_attr(&inner));
                        out.push_str("\" class=\"nc-external\">");
                        push_escaped(out, &inner);
                        out.push_str("</a>");
                        i = close + 1;
                        continue;
                    }
                }
                out.push_str("&lt;");
                i += 1;
            }
            '*' | '_' => {
                let run = chars[i..end].iter().take_while(|c| **c == ch).count();
                if run >= 2 {
                    if let Some(close) = find_sequence(chars, i + 2, end, ch, 2) {
                        if close > i + 2 {
                            out.push_str("<strong>");
                            inline_into(chars, i + 2, close, ctx, out, depth + 1);
                            out.push_str("</strong>");
                            i = close + 2;
                            continue;
                        }
                    }
                }
                if let Some(close) = find_char(chars, i + 1, end, ch) {
                    if close > i + 1
                        && !chars[i + 1].is_whitespace()
                        && !chars[close - 1].is_whitespace()
                    {
                        out.push_str("<em>");
                        inline_into(chars, i + 1, close, ctx, out, depth + 1);
                        out.push_str("</em>");
                        i = close + 1;
                        continue;
                    }
                }
                push_escaped_char(out, ch);
                i += 1;
            }
            '~' if i + 1 < end && chars[i + 1] == '~' => {
                if let Some(close) = find_sequence(chars, i + 2, end, '~', 2) {
                    out.push_str("<del>");
                    inline_into(chars, i + 2, close, ctx, out, depth + 1);
                    out.push_str("</del>");
                    i = close + 2;
                } else {
                    out.push('~');
                    i += 1;
                }
            }
            '=' if i + 1 < end && chars[i + 1] == '=' => {
                if let Some(close) = find_sequence(chars, i + 2, end, '=', 2) {
                    out.push_str("<mark>");
                    inline_into(chars, i + 2, close, ctx, out, depth + 1);
                    out.push_str("</mark>");
                    i = close + 2;
                } else {
                    out.push('=');
                    i += 1;
                }
            }
            '{' if ctx.dialect == Dialect::Mdx => {
                if let Some(close) = find_char(chars, i + 1, end, '}') {
                    let inner: String = chars[i + 1..close].iter().collect();
                    out.push_str("<code class=\"nc-mdx-expr\">{");
                    push_escaped(out, &inner);
                    out.push_str("}</code>");
                    i = close + 1;
                } else {
                    out.push('{');
                    i += 1;
                }
            }
            '\n' => {
                let hard = i >= 2 && chars[i - 1] == ' ' && chars[i - 2] == ' ';
                out.push_str(if hard { "<br>" } else { "\n" });
                i += 1;
            }
            _ => {
                push_escaped_char(out, ch);
                i += 1;
            }
        }
    }
}

fn find_pair(chars: &[char], from: usize, end: usize, first: char, second: char) -> Option<usize> {
    let mut position = from;
    while position + 1 < end {
        if chars[position] == first && chars[position + 1] == second {
            return Some(position);
        }
        position += 1;
    }
    None
}

fn parse_link(
    chars: &[char],
    start: usize,
    end: usize,
    ctx: &mut Ctx,
    out: &mut String,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut close = None;
    for position in start..end {
        match chars[position] {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close = Some(position);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    if close + 1 >= end || chars[close + 1] != '(' {
        return None;
    }
    let target_close = find_char(chars, close + 2, end, ')')?;
    let raw: String = chars[close + 2..target_close].iter().collect();
    let (url, title) = split_destination(&raw);
    let url = safe_url(&url)?;
    out.push_str("<a href=\"");
    out.push_str(&escape_attr(&url));
    out.push('"');
    if let Some(title) = title {
        out.push_str(" title=\"");
        out.push_str(&escape_attr(&title));
        out.push('"');
    }
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        out.push_str(" class=\"nc-external\" rel=\"noreferrer noopener\"");
    }
    out.push('>');
    inline_into(chars, start + 1, close, ctx, out, 1);
    out.push_str("</a>");
    Some(target_close + 1)
}

fn parse_image(
    chars: &[char],
    start: usize,
    end: usize,
    _ctx: &mut Ctx,
    out: &mut String,
) -> Option<usize> {
    let close = find_char(chars, start + 2, end, ']')?;
    if close + 1 >= end || chars[close + 1] != '(' {
        return None;
    }
    let target_close = find_char(chars, close + 2, end, ')')?;
    let raw: String = chars[close + 2..target_close].iter().collect();
    let (url, title) = split_destination(&raw);
    let url = safe_url(&url)?;
    let alt: String = chars[start + 2..close].iter().collect();
    out.push_str("<img src=\"");
    out.push_str(&escape_attr(&url));
    out.push_str("\" alt=\"");
    out.push_str(&escape_attr(&alt));
    out.push('"');
    if let Some(title) = title {
        out.push_str(" title=\"");
        out.push_str(&escape_attr(&title));
        out.push('"');
    }
    out.push_str(" loading=\"lazy\" />");
    Some(target_close + 1)
}

fn unique_id(base: &str, ctx: &mut Ctx) -> String {
    let mut candidate = base.to_string();
    let mut counter = 2;
    while ctx.heading_ids.iter().any(|id| id == &candidate) {
        candidate = format!("{}-{}", base, counter);
        counter += 1;
    }
    ctx.heading_ids.push(candidate.clone());
    candidate
}

// ----------------------------------------------------------------- JSON -----

#[derive(Debug, Clone)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    Text(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

/// Convert a JSON document into a readable Markdown outline.
pub fn json_to_text(source: &str) -> String {
    match parse_json(source) {
        Ok(value) => {
            let mut out = String::new();
            outline(&value, 0, &mut out);
            if out.trim().is_empty() {
                out.push_str("The document is empty.\n");
            }
            out
        }
        Err(error) => format!(
            "This file is not valid JSON: {}\n\n```\n{}\n```\n",
            error,
            source.trim()
        ),
    }
}

fn outline(value: &Json, depth: usize, out: &mut String) {
    match value {
        Json::Object(entries) => {
            if entries.is_empty() {
                out.push_str("- (empty object)\n\n");
                return;
            }
            for (key, value) in entries {
                match value {
                    Json::Object(inner) if !inner.is_empty() => {
                        out.push_str(&format!(
                            "{} {}\n\n",
                            "#".repeat((depth + 2).min(6)),
                            escape_markdown(key)
                        ));
                        outline(value, depth + 1, out);
                    }
                    Json::Array(inner) if !inner.is_empty() => {
                        out.push_str(&format!(
                            "{} {}\n\n",
                            "#".repeat((depth + 2).min(6)),
                            escape_markdown(key)
                        ));
                        outline(value, depth + 1, out);
                    }
                    _ => {
                        out.push_str(&format!(
                            "- **{}**: {}\n",
                            escape_markdown(key),
                            scalar_text(value)
                        ));
                    }
                }
            }
            out.push('\n');
        }
        Json::Array(items) => {
            if items.is_empty() {
                out.push_str("- (empty list)\n\n");
                return;
            }
            for (position, item) in items.iter().enumerate() {
                match item {
                    Json::Object(entries) if !entries.is_empty() => {
                        out.push_str(&format!(
                            "{} {}\n\n",
                            "#".repeat((depth + 2).min(6)),
                            position + 1
                        ));
                        outline(item, depth + 1, out);
                    }
                    Json::Array(inner) if !inner.is_empty() => {
                        out.push_str(&format!("- item {}\n", position + 1));
                        outline(item, depth + 1, out);
                    }
                    _ => out.push_str(&format!("- {}\n", scalar_text(item))),
                }
            }
            out.push('\n');
        }
        other => {
            out.push_str(&scalar_text(other));
            out.push('\n');
        }
    }
}

fn scalar_text(value: &Json) -> String {
    match value {
        Json::Null => "—".to_string(),
        Json::Bool(true) => "true".to_string(),
        Json::Bool(false) => "false".to_string(),
        Json::Number(number) => number.clone(),
        Json::Text(text) => {
            if text.contains('\n') {
                text.lines()
                    .map(escape_markdown)
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                escape_markdown(text)
            }
        }
        Json::Array(items) => format!("[{} items]", items.len()),
        Json::Object(entries) => format!("{{{} fields}}", entries.len()),
    }
}

fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '*' | '_' | '`' | '[' | ']' | '<' | '>' | '#' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn parse_json(source: &str) -> Result<Json, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut position = 0;
    let value = json_value(&chars, &mut position, 0)?;
    skip_whitespace(&chars, &mut position);
    if position < chars.len() {
        return Err(format!("unexpected text at character {}", position + 1));
    }
    Ok(value)
}

fn skip_whitespace(chars: &[char], position: &mut usize) {
    while *position < chars.len() && chars[*position].is_whitespace() {
        *position += 1;
    }
}

fn json_value(chars: &[char], position: &mut usize, depth: usize) -> Result<Json, String> {
    if depth > 96 {
        return Err("this document is nested too deeply".to_string());
    }
    skip_whitespace(chars, position);
    let ch = *chars
        .get(*position)
        .ok_or_else(|| "the document ends unexpectedly".to_string())?;
    match ch {
        '{' => {
            *position += 1;
            let mut entries: Vec<(String, Json)> = Vec::new();
            skip_whitespace(chars, position);
            if chars.get(*position) == Some(&'}') {
                *position += 1;
                return Ok(Json::Object(entries));
            }
            loop {
                skip_whitespace(chars, position);
                let key = match json_value(chars, position, depth + 1)? {
                    Json::Text(text) => text,
                    _ => return Err("object keys must be strings".to_string()),
                };
                skip_whitespace(chars, position);
                if chars.get(*position) != Some(&':') {
                    return Err(format!(
                        "expected ':' after key \"{}\" at character {}",
                        key,
                        *position + 1
                    ));
                }
                *position += 1;
                let value = json_value(chars, position, depth + 1)?;
                entries.push((key, value));
                skip_whitespace(chars, position);
                match chars.get(*position) {
                    Some(',') => {
                        *position += 1;
                    }
                    Some('}') => {
                        *position += 1;
                        break;
                    }
                    _ => {
                        return Err(format!(
                            "expected ',' or '}}' at character {}",
                            *position + 1
                        ))
                    }
                }
            }
            Ok(Json::Object(entries))
        }
        '[' => {
            *position += 1;
            let mut items = Vec::new();
            skip_whitespace(chars, position);
            if chars.get(*position) == Some(&']') {
                *position += 1;
                return Ok(Json::Array(items));
            }
            loop {
                let value = json_value(chars, position, depth + 1)?;
                items.push(value);
                skip_whitespace(chars, position);
                match chars.get(*position) {
                    Some(',') => {
                        *position += 1;
                    }
                    Some(']') => {
                        *position += 1;
                        break;
                    }
                    _ => {
                        return Err(format!(
                            "expected ',' or ']' at character {}",
                            *position + 1
                        ))
                    }
                }
            }
            Ok(Json::Array(items))
        }
        '"' => json_string(chars, position),
        't' => {
            expect_word(chars, position, "true")?;
            Ok(Json::Bool(true))
        }
        'f' => {
            expect_word(chars, position, "false")?;
            Ok(Json::Bool(false))
        }
        'n' => {
            expect_word(chars, position, "null")?;
            Ok(Json::Null)
        }
        _ => json_number(chars, position),
    }
}

fn expect_word(chars: &[char], position: &mut usize, word: &str) -> Result<(), String> {
    for expected in word.chars() {
        if chars.get(*position) != Some(&expected) {
            return Err(format!(
                "expected \"{}\" at character {}",
                word,
                *position + 1
            ));
        }
        *position += 1;
    }
    Ok(())
}

fn json_number(chars: &[char], position: &mut usize) -> Result<Json, String> {
    let start = *position;
    while *position < chars.len() {
        let ch = chars[*position];
        if ch.is_ascii_digit() || ch == '-' || ch == '+' || ch == '.' || ch == 'e' || ch == 'E' {
            *position += 1;
        } else {
            break;
        }
    }
    if start == *position {
        return Err(format!(
            "unexpected character at position {}",
            *position + 1
        ));
    }
    let raw: String = chars[start..*position].iter().collect();
    if raw.parse::<f64>().is_err() {
        return Err(format!("\"{}\" is not a number", raw));
    }
    Ok(Json::Number(raw))
}

fn json_string(chars: &[char], position: &mut usize) -> Result<Json, String> {
    *position += 1;
    let mut text = String::new();
    loop {
        let ch = *chars
            .get(*position)
            .ok_or_else(|| "a string is not closed".to_string())?;
        *position += 1;
        match ch {
            '"' => break,
            '\\' => {
                let escape = *chars
                    .get(*position)
                    .ok_or_else(|| "an escape sequence is incomplete".to_string())?;
                *position += 1;
                match escape {
                    '"' => text.push('"'),
                    '\\' => text.push('\\'),
                    '/' => text.push('/'),
                    'b' => text.push('\u{8}'),
                    'f' => text.push('\u{c}'),
                    'n' => text.push('\n'),
                    'r' => text.push('\r'),
                    't' => text.push('\t'),
                    'u' => {
                        let code = json_hex4(chars, position)?;
                        if (0xD800..=0xDBFF).contains(&code) {
                            if chars.get(*position) == Some(&'\\')
                                && chars.get(*position + 1) == Some(&'u')
                            {
                                *position += 2;
                                let low = json_hex4(chars, position)?;
                                let combined =
                                    0x10000 + ((code - 0xD800) << 10) + (low.wrapping_sub(0xDC00));
                                text.push(char::from_u32(combined).unwrap_or('\u{FFFD}'));
                            } else {
                                text.push('\u{FFFD}');
                            }
                        } else {
                            text.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                    }
                    other => {
                        return Err(format!("unknown escape sequence \"\\{}\"", other));
                    }
                }
            }
            other => text.push(other),
        }
    }
    Ok(Json::Text(text))
}

fn json_hex4(chars: &[char], position: &mut usize) -> Result<u32, String> {
    let mut value = 0u32;
    for _ in 0..4 {
        let ch = *chars
            .get(*position)
            .ok_or_else(|| "an incomplete \\u escape".to_string())?;
        let digit = ch
            .to_digit(16)
            .ok_or_else(|| format!("\"{}\" is not a hex digit", ch))?;
        value = value * 16 + digit;
        *position += 1;
    }
    Ok(value)
}

// ----------------------------------------------------------------- tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_get_stable_ids() {
        let html = render("# Hello World\n\n## Hello World", Dialect::Markdown);
        assert!(html.contains("<h1 id=\"hello-world\">Hello World</h1>"));
        assert!(html.contains("<h2 id=\"hello-world-2\">"));
    }

    #[test]
    fn emphasis_and_code() {
        let html = render("A **bold** and *soft* `code` line.", Dialect::Markdown);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>soft</em>"));
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn nested_lists_and_tasks() {
        let src = "- [ ] parent\n  - child\n- [x] done";
        let html = render(src, Dialect::Markdown);
        assert!(html.contains("nc-task"));
        assert!(html.contains("<ul class=\"nc-list\">"));
        assert_eq!(html.matches("<li").count(), 3);
        assert!(html.contains("checked"));
    }

    #[test]
    fn fenced_code_is_escaped_and_labelled() {
        let html = render("```rust\nlet x = 1 < 2;\n```", Dialect::Markdown);
        assert!(html.contains("class=\"language-rust\""));
        assert!(html.contains("1 &lt; 2"));
    }

    #[test]
    fn tables_render_with_alignment() {
        let html = render("| a | b |\n| :-- | --: |\n| 1 | 2 |", Dialect::Markdown);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th align=\"left\">a</th>"));
        assert!(html.contains("<td align=\"right\">2</td>"));
    }

    #[test]
    fn wiki_links_become_anchors() {
        let html = render(
            "See [[Other]] and [[Note#My Heading|alias]].",
            Dialect::Markdown,
        );
        assert!(html.contains("href=\"Other.md\""));
        assert!(html.contains("href=\"Note.md#My%20Heading\""));
        assert!(html.contains(">alias</a>"));
    }

    #[test]
    fn callouts_render() {
        let html = render("> [!warning] Careful\n> Watch out.", Dialect::Markdown);
        assert!(html.contains("nc-callout-warning"));
        assert!(html.contains("WARNING"));
        assert!(html.contains("Watch out."));
    }

    #[test]
    fn javascript_urls_are_dropped() {
        let html = render("[click](javascript:alert(1))", Dialect::Markdown);
        // The link is not created; the source is shown as inert text instead.
        assert!(!html.contains("href=\"javascript:"));
        assert!(html.contains("[click](javascript:alert(1))"));
    }

    #[test]
    fn mdx_components_and_expressions_are_surfaced() {
        let src =
            "import Chart from './chart'\n\n# Report\n\n<Chart data={data} />\n\nValue: {count}";
        let html = render(src, Dialect::Mdx);
        assert!(!html.contains("import Chart"));
        assert!(html.contains("nc-mdx-block"));
        assert!(html.contains("nc-mdx-expr"));
        assert!(html.contains("<h1 id=\"report\">"));
    }

    #[test]
    fn json_becomes_a_readable_outline() {
        let src = r#"{"name":"NoteCraft","count":3,"tags":["a","b"],"owner":{"name":"Ada"}}"#;
        let text = json_to_text(src);
        assert!(text.contains("**name**: NoteCraft"));
        assert!(text.contains("**count**: 3"));
        assert!(text.contains("## owner"));
        let html = render_document(src, Kind::Json);
        assert!(!html.contains("<table"));
        assert!(html.contains("NoteCraft"));
    }

    #[test]
    fn invalid_json_is_reported_not_panicked() {
        let text = json_to_text("{oops");
        assert!(text.contains("not valid JSON"));
    }

    #[test]
    fn kind_from_path() {
        assert_eq!(Kind::from_path("a/b.mdx"), Kind::Mdx);
        assert_eq!(Kind::from_path("a/b.JSON"), Kind::Json);
        assert_eq!(Kind::from_path("a/b.md"), Kind::Markdown);
    }

    #[test]
    fn frontmatter_is_rendered_as_metadata() {
        let html = render("---\ntitle: Hi\ntags: [a]\n---\n\nBody", Dialect::Markdown);
        assert!(html.contains("nc-frontmatter"));
        assert!(html.contains("<dt>title</dt><dd>Hi</dd>"));
        assert!(html.contains("<p>Body</p>"));
    }

    #[test]
    fn footnotes_collect() {
        let html = render("Text[^1]\n\n[^1]: Note body", Dialect::Markdown);
        assert!(html.contains("nc-fn-ref"));
        assert!(html.contains("nc-footnotes"));
        assert!(html.contains("Note body"));
    }
}
