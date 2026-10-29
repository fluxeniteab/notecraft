//! Markdown parsing, the native reading view, outlines and statistics.
//!
//! Block and inline parsing is done by `pulldown-cmark` — a real CommonMark
//! parser rather than a regex pass. The renderer below turns those events into
//! egui text layouts, so the reading view keeps the typography of the original
//! application without a webview.
//!
//! MDX uses the same parser: JSX and `{expressions}` arrive as `InlineHtml` /
//! `Html` events and are shown as inert, clearly marked source. Nothing in this
//! module evaluates document content.

use std::collections::HashMap;

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Align, Color32, CornerRadius, Label, Rect, Sense, Stroke, Ui, Vec2};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::theme::Theme;

pub mod export;
pub mod highlight;

/// Document statistics shown in the status bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DocStats {
    pub words: usize,
    pub chars: usize,
    pub lines: usize,
}

pub fn stats(text: &str) -> DocStats {
    DocStats {
        words: text
            .split_whitespace()
            .filter(|w| w.chars().any(char::is_alphanumeric))
            .count(),
        chars: text.chars().count(),
        lines: text.lines().count().max(1),
    }
}

/// A heading in the outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub anchor: String,
    /// 1-based line number, for "go to line".
    pub line: usize,
}

/// Stable, duplicate-free anchors (`#my-heading`, `#my-heading-2`, …).
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Extract the outline. Fenced code is skipped so `# comment` inside a shell
/// snippet never appears as a heading.
pub fn outline(text: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut in_fence = false;
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if hashes == 0 || hashes > 6 {
            continue;
        }
        let rest = &trimmed[hashes..];
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
            continue;
        }
        let title = rest.trim().trim_end_matches('#').trim();
        if title.is_empty() {
            continue;
        }
        let base = slugify(title);
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        headings.push(Heading {
            level: hashes as u8,
            text: title.to_string(),
            anchor: if *count == 1 {
                base
            } else {
                format!("{base}-{count}")
            },
            line: index + 1,
        });
    }
    headings
}

/// What the preview asks the application to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewAction {
    /// A link was clicked: `file#anchor`, `#anchor` or an absolute URL.
    Open(String),
    /// A task checkbox was clicked; the byte offset points at the marker.
    ToggleTask(usize),
}

#[derive(Clone, Default)]
struct Span {
    text: String,
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
    raw: bool,
    link: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum BlockKind {
    #[default]
    Paragraph,
    Heading(u8),
    Item,
    Quote,
}

struct Marker {
    label: String,
    checked: Option<bool>,
    offset: usize,
}

struct Block {
    kind: BlockKind,
    spans: Vec<Span>,
    /// Nesting depth (lists + block quotes).
    indent: usize,
    marker: Option<Marker>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            kind: BlockKind::Paragraph,
            spans: Vec::new(),
            indent: 0,
            marker: None,
        }
    }
}

/// Inline style state, applied to every span as it is produced.
#[derive(Clone, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
    link: Option<String>,
}

fn parser_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SMART_PUNCTUATION
}

/// Render a document into `ui`.
pub fn render(ui: &mut Ui, theme: &Theme, text: &str) -> Option<PreviewAction> {
    let mut action = None;
    let width = ui.available_width().min(theme.reading_width);
    let left_pad = ((ui.available_width() - width) * 0.5).max(0.0);

    ui.horizontal_top(|ui| {
        if left_pad > 0.0 {
            ui.add_space(left_pad);
        }
        ui.allocate_ui_with_layout(
            Vec2::new(width, ui.available_height()),
            egui::Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing.y = 9.0;
                if let Some(found) = render_blocks(ui, theme, text) {
                    action = Some(found);
                }
            },
        );
    });
    action
}

fn heading_size(level: u8, base: f32) -> f32 {
    match level {
        1 => base * 1.8,
        2 => base * 1.42,
        3 => base * 1.2,
        4 => base * 1.07,
        _ => base,
    }
}

#[allow(unused_assignments)]
fn render_blocks(ui: &mut Ui, theme: &Theme, text: &str) -> Option<PreviewAction> {
    let p = &theme.palette;
    let mut action = None;

    let mut block = Block::default();
    let mut style = Style::default();
    let mut quote_depth = 0usize;
    let mut lists: Vec<(bool, u64, usize)> = Vec::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut table: Option<Vec<Vec<String>>> = None;
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut header_rows = 0usize;

    macro_rules! flush {
        () => {
            if !block.spans.is_empty() || block.marker.is_some() {
                if let Some(found) = draw_block(ui, theme, &mut block) {
                    action = Some(found);
                }
            }
            block = Block::default();
        };
    }

    for (event, range) in Parser::new_ext(text, parser_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Paragraph) => {
                block.indent = lists.len() + quote_depth;
            }
            Event::End(TagEnd::Paragraph) => {
                flush!();
            }
            Event::Start(Tag::Heading { level, .. }) => {
                block.kind = BlockKind::Heading(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
                block.indent = 0;
            }
            Event::End(TagEnd::Heading(_)) => {
                flush!();
            }
            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => {
                quote_depth = quote_depth.saturating_sub(1);
                flush!();
            }
            Event::Start(Tag::List(start)) => lists.push((start.is_some(), start.unwrap_or(1), 0)),
            Event::End(TagEnd::List(_)) => {
                lists.pop();
                flush!();
            }
            Event::Start(Tag::Item) => {
                flush!();
                if let Some(top) = lists.last_mut() {
                    top.2 += 1;
                }
                block.kind = BlockKind::Item;
                block.indent = lists.len() + quote_depth;
                block.marker = Some(Marker {
                    label: match lists.last() {
                        Some((true, start, index)) => format!("{}.", *start as usize + *index - 1),
                        _ => "•".to_string(),
                    },
                    checked: None,
                    offset: range.start,
                });
            }
            Event::End(TagEnd::Item) => {
                flush!();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_buf.clear();
                code_lang = match kind {
                    CodeBlockKind::Fenced(info) => info.trim().to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                draw_code_block(ui, theme, &code_buf, &code_lang);
                code_buf.clear();
                code_lang.clear();
            }
            Event::Start(Tag::Table(_)) => {
                table = Some(Vec::new());
                header_rows = 0;
            }
            Event::End(TagEnd::Table) => {
                if let Some(rows) = table.take() {
                    draw_table(ui, theme, &rows, header_rows);
                }
            }
            Event::Start(Tag::TableHead) => row.clear(),
            Event::End(TagEnd::TableHead) => {
                if let Some(rows) = table.as_mut() {
                    rows.push(std::mem::take(&mut row));
                    header_rows = 1;
                }
            }
            Event::Start(Tag::TableRow) => row.clear(),
            Event::End(TagEnd::TableRow) => {
                if let Some(rows) = table.as_mut() {
                    rows.push(std::mem::take(&mut row));
                }
            }
            Event::Start(Tag::TableCell) => {}
            Event::End(TagEnd::TableCell) => row.push(std::mem::take(&mut cell).trim().to_string()),
            Event::Start(Tag::Emphasis) => style.italic = true,
            Event::End(TagEnd::Emphasis) => style.italic = false,
            Event::Start(Tag::Strong) => style.bold = true,
            Event::End(TagEnd::Strong) => style.bold = false,
            Event::Start(Tag::Strikethrough) => style.strike = true,
            Event::End(TagEnd::Strikethrough) => style.strike = false,
            Event::Start(Tag::Link { dest_url, .. }) => style.link = Some(dest_url.to_string()),
            Event::End(TagEnd::Link) => style.link = None,
            Event::Start(Tag::Image { dest_url, .. }) => block.spans.push(Span {
                text: format!("[image: {dest_url}]"),
                italic: true,
                ..Default::default()
            }),
            Event::End(TagEnd::Image) => {}
            Event::Start(Tag::FootnoteDefinition(name)) => {
                block.kind = BlockKind::Quote;
                block.spans.push(Span {
                    text: format!("[{name}] "),
                    bold: true,
                    ..Default::default()
                });
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                flush!();
            }
            Event::TaskListMarker(checked) => {
                if let Some(marker) = block.marker.as_mut() {
                    marker.checked = Some(checked);
                    let slice = &text[range.clone()];
                    marker.offset = range.start + slice.find('[').unwrap_or(0);
                }
            }
            Event::Text(value) => {
                if in_code {
                    code_buf.push_str(&value);
                } else if table.is_some() {
                    cell.push_str(&value);
                } else {
                    block.spans.push(Span {
                        text: value.to_string(),
                        bold: style.bold,
                        italic: style.italic,
                        strike: style.strike,
                        link: style.link.clone(),
                        ..Default::default()
                    });
                }
            }
            Event::Code(value) => {
                if table.is_some() {
                    cell.push_str(&value);
                } else {
                    block.spans.push(Span {
                        text: value.to_string(),
                        code: true,
                        link: style.link.clone(),
                        ..Default::default()
                    });
                }
            }
            Event::Html(value) | Event::InlineHtml(value) => {
                if in_code {
                    code_buf.push_str(&value);
                } else {
                    block.spans.push(Span {
                        text: value.to_string(),
                        raw: true,
                        ..Default::default()
                    });
                }
            }
            Event::SoftBreak => block.spans.push(Span {
                text: " ".into(),
                ..Default::default()
            }),
            Event::HardBreak => block.spans.push(Span {
                text: "\n".into(),
                ..Default::default()
            }),
            Event::Rule => {
                flush!();
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 17.0), Sense::hover());
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.left(), rect.center().y),
                        egui::pos2(rect.right(), rect.center().y),
                    ],
                    Stroke::new(1.0, p.border),
                );
            }
            Event::FootnoteReference(name) => block.spans.push(Span {
                text: format!("[{name}]"),
                bold: true,
                ..Default::default()
            }),
            Event::InlineMath(value) | Event::DisplayMath(value) => block.spans.push(Span {
                text: value.to_string(),
                code: true,
                raw: true,
                ..Default::default()
            }),
            Event::Start(_) | Event::End(_) => {}
        }
    }
    flush!();
    action
}

fn build_job(
    theme: &Theme,
    block: &Block,
    size: f32,
    wrap: Option<f32>,
    base_color: Color32,
) -> LayoutJob {
    let p = &theme.palette;
    let mut job = LayoutJob::default();
    if let Some(width) = wrap {
        job.wrap = TextWrapping {
            max_width: width,
            ..Default::default()
        };
    }
    let line_height = (size * theme.editor_line_height).round();

    for span in &block.spans {
        if span.text.is_empty() {
            continue;
        }
        let font = if span.code || span.raw {
            theme.mono(size * 0.93, false)
        } else {
            theme.font(size, span.bold, span.italic)
        };
        let color = if span.raw {
            p.warning
        } else if span.link.is_some() {
            p.accent
        } else {
            base_color
        };
        let mut format = TextFormat {
            font_id: font,
            color,
            line_height: Some(line_height),
            ..Default::default()
        };
        if span.strike {
            format.strikethrough = Stroke::new(1.0, color);
        }
        if span.code {
            format.background = p.code_bg;
        }
        job.append(&span.text, 0.0, format);
    }
    job
}

fn draw_block(ui: &mut Ui, theme: &Theme, block: &mut Block) -> Option<PreviewAction> {
    let p = &theme.palette;
    let indent = block.indent as f32 * 18.0;

    match block.kind {
        BlockKind::Heading(level) => {
            ui.add_space(if level <= 2 { 12.0 } else { 6.0 });
            let size = heading_size(level, theme.editor_font_size + 1.0);
            let mut job = build_job(theme, block, size, Some(ui.available_width()), p.fg);
            for section in &mut job.sections {
                section.format.font_id = theme.font(size, true, false);
                section.format.color = p.fg;
            }
            ui.add(Label::new(job));
            ui.add_space(2.0);
            None
        }
        BlockKind::Paragraph => {
            let job = build_job(
                theme,
                block,
                theme.editor_font_size,
                Some((ui.available_width() - indent).max(80.0)),
                p.fg,
            );
            if let Some(marker) = &block.marker {
                let marker = Marker {
                    label: marker.label.clone(),
                    checked: marker.checked,
                    offset: marker.offset,
                };
                let mut clicked = None;
                ui.horizontal_top(|ui| {
                    if indent > 18.0 {
                        ui.add_space(indent - 18.0);
                    }
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(20.0, theme.row_height()), Sense::click());
                    match marker.checked {
                        Some(checked) => {
                            draw_checkbox(ui, rect, checked, theme);
                            if resp.clicked() {
                                clicked = Some(PreviewAction::ToggleTask(marker.offset));
                            }
                        }
                        None => {
                            ui.painter().text(
                                egui::pos2(rect.right(), rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                &marker.label,
                                theme.font(theme.editor_font_size * 0.92, false, false),
                                p.fg_faint,
                            );
                        }
                    }
                    ui.add(Label::new(job));
                });
                return clicked;
            }
            let response = ui.add(Label::new(job).sense(Sense::click()));
            respond_to_link(block, &response)
        }
        BlockKind::Item => {
            let job = build_job(
                theme,
                block,
                theme.editor_font_size,
                Some((ui.available_width() - indent - 26.0).max(80.0)),
                p.fg,
            );
            let marker = block.marker.as_ref().map(|m| Marker {
                label: m.label.clone(),
                checked: m.checked,
                offset: m.offset,
            });
            let mut clicked = None;
            ui.horizontal_top(|ui| {
                if indent > 18.0 {
                    ui.add_space(indent - 18.0);
                }
                if let Some(marker) = &marker {
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(20.0, theme.row_height()), Sense::click());
                    match marker.checked {
                        Some(checked) => {
                            draw_checkbox(ui, rect, checked, theme);
                            if resp.clicked() {
                                clicked = Some(PreviewAction::ToggleTask(marker.offset));
                            }
                        }
                        None => {
                            ui.painter().text(
                                egui::pos2(rect.right(), rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                &marker.label,
                                theme.font(theme.editor_font_size * 0.92, false, false),
                                p.fg_faint,
                            );
                        }
                    }
                }
                ui.add(Label::new(job));
            });
            clicked
        }
        BlockKind::Quote => {
            let job = build_job(
                theme,
                block,
                theme.editor_font_size,
                Some((ui.available_width() - indent - 22.0).max(80.0)),
                p.fg_secondary,
            );
            ui.horizontal_top(|ui| {
                if indent > 0.0 {
                    ui.add_space(indent);
                }
                let (bar_rect, _) =
                    ui.allocate_exact_size(Vec2::new(13.0, theme.row_height()), Sense::hover());
                ui.painter().rect_filled(
                    Rect::from_min_size(bar_rect.min, Vec2::new(3.0, bar_rect.height())),
                    CornerRadius::same(2),
                    p.border_strong,
                );
                ui.add(Label::new(job));
            });
            None
        }
    }
}

fn respond_to_link(block: &Block, response: &egui::Response) -> Option<PreviewAction> {
    let url = block.spans.iter().find_map(|s| s.link.clone())?;
    if response.hovered() {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        Some(PreviewAction::Open(url))
    } else {
        None
    }
}

fn draw_checkbox(ui: &Ui, rect: Rect, checked: bool, theme: &Theme) {
    let p = &theme.palette;
    let box_rect = Rect::from_center_size(rect.center(), Vec2::splat(13.0));
    let corner = CornerRadius::same(3);
    if checked {
        ui.painter().rect_filled(box_rect, corner, p.accent);
        crate::theme::icons::paint(
            ui.painter(),
            box_rect.shrink(2.0),
            crate::theme::Icon::Check,
            p.accent_fg,
            1.6,
        );
    } else {
        ui.painter().rect_stroke(
            box_rect,
            corner,
            Stroke::new(1.0, p.border_strong),
            egui::StrokeKind::Inside,
        );
    }
}

fn draw_code_block(ui: &mut Ui, theme: &Theme, code: &str, lang: &str) {
    let p = &theme.palette;
    let body = code.trim_end();
    if body.is_empty() {
        return;
    }
    let size = theme.editor_font_size * 0.92;
    let job = highlight::highlight(theme, lang, body, size);
    let galley = ui.painter().layout_job(job);
    let padding = 12.0;
    let height = galley.size().y + padding * 2.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

    ui.painter()
        .rect_filled(rect, CornerRadius::same(8), p.code_bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, p.border),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .galley(rect.min + Vec2::new(padding, padding), galley, p.fg);

    if response.hovered() {
        let copy_rect = Rect::from_min_size(
            egui::pos2(rect.right() - 54.0, rect.top() + 6.0),
            Vec2::new(48.0, 18.0),
        );
        ui.painter()
            .rect_filled(copy_rect, CornerRadius::same(4), p.raised);
        ui.painter().text(
            copy_rect.center(),
            egui::Align2::CENTER_CENTER,
            "copy",
            theme.tiny(),
            p.fg_secondary,
        );
        if response.clicked() {
            ui.ctx().copy_text(body.to_string());
        }
    }
    ui.add_space(2.0);
}

fn draw_table(ui: &mut Ui, theme: &Theme, rows: &[Vec<String>], header_rows: usize) {
    let p = &theme.palette;
    if rows.is_empty() {
        return;
    }
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return;
    }
    egui::Frame::new()
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            egui::Grid::new(ui.next_auto_id())
                .num_columns(columns)
                .spacing([18.0, 6.0])
                .show(ui, |ui| {
                    for (row_index, row) in rows.iter().enumerate() {
                        let is_header = row_index < header_rows;
                        for column in 0..columns {
                            let text = row.get(column).cloned().unwrap_or_default();
                            let mut job = LayoutJob::default();
                            job.append(
                                &text,
                                0.0,
                                TextFormat {
                                    font_id: theme.font(
                                        theme.editor_font_size * 0.95,
                                        is_header,
                                        false,
                                    ),
                                    color: if is_header { p.fg } else { p.fg_secondary },
                                    line_height: Some(theme.row_height()),
                                    ..Default::default()
                                },
                            );
                            ui.add(Label::new(job));
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_count_words_and_lines() {
        let s = stats("# Title\n\nhello world\n");
        assert_eq!(s.words, 3);
        assert_eq!(s.lines, 4);
    }

    #[test]
    fn outline_skips_fenced_code() {
        let headings = outline("# One\n\n```sh\n# not a heading\n```\n\n## Two\n");
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].text, "One");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].line, 8);
    }

    #[test]
    fn duplicate_headings_get_unique_anchors() {
        let headings = outline("# Same\n\n# Same\n\n# Same\n");
        assert_eq!(headings[0].anchor, "same");
        assert_eq!(headings[1].anchor, "same-2");
        assert_eq!(headings[2].anchor, "same-3");
    }

    #[test]
    fn anchors_are_url_safe() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  Trailing  "), "trailing");
        assert_eq!(slugify("日本語 テスト"), "日本語-テスト");
    }

    fn block(spans: Vec<Span>) -> Block {
        Block {
            kind: BlockKind::Paragraph,
            spans,
            indent: 0,
            marker: None,
        }
    }

    fn theme() -> Theme {
        Theme::build(
            crate::theme::ThemeKind::Dark,
            "#4f86f7",
            &crate::settings::Settings::default(),
        )
    }

    #[test]
    fn inline_styles_survive_into_the_job() {
        let job = build_job(
            &theme(),
            &block(vec![
                Span {
                    text: "plain ".into(),
                    ..Default::default()
                },
                Span {
                    text: "bold".into(),
                    bold: true,
                    ..Default::default()
                },
                Span {
                    text: " code".into(),
                    code: true,
                    ..Default::default()
                },
                Span {
                    text: "gone".into(),
                    strike: true,
                    ..Default::default()
                },
            ]),
            15.0,
            Some(600.0),
            egui::Color32::WHITE,
        );
        assert_eq!(job.sections.len(), 4);
        assert_eq!(job.text, "plain bold codegone");
        assert!(job.sections[3].format.strikethrough.width > 0.0);
    }

    #[test]
    fn links_are_coloured_with_the_accent() {
        let theme = theme();
        let job = build_job(
            &theme,
            &block(vec![Span {
                text: "docs".into(),
                link: Some("https://example.com".into()),
                ..Default::default()
            }]),
            15.0,
            None,
            theme.palette.fg,
        );
        assert_eq!(job.sections[0].format.color, theme.palette.accent);
    }
}
