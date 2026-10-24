// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use egui::{FontId, RichText, Ui};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Default, Clone)]
struct SpanStyle {
    bold: bool,
    italic: bool,
    code: bool,
    strike: bool,
}

fn flush_paragraph(
    ui: &mut Ui,
    spans: &mut Vec<(String, SpanStyle)>,
    heading: Option<u8>,
    blockquote: bool,
) {
    if spans.is_empty() {
        return;
    }
    let font_size = match heading {
        Some(1) => 22.0,
        Some(2) => 19.0,
        Some(3) => 17.0,
        Some(_) => 15.0,
        None => 14.0,
    };
    let mut job = egui::text::LayoutJob::default();
    for (t, st) in spans.drain(..) {
        let mut fmt = egui::TextFormat {
            font_id: FontId::proportional(font_size),
            color: egui::Color32::from_rgb(220, 225, 235),
            ..Default::default()
        };
        if st.code {
            fmt.font_id = FontId::monospace(font_size - 1.0);
            fmt.background = egui::Color32::from_rgb(40, 42, 50);
            fmt.color = egui::Color32::from_rgb(230, 230, 230);
        }
        if st.bold {
            // emulate bold by using strong color
            fmt.color = egui::Color32::from_rgb(245, 245, 255);
        }
        let section = egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: (job.text.len())..(job.text.len() + t.len()),
            format: fmt,
        };
        job.text.push_str(&t);
        job.sections.push(section);
        let _ = (st.italic, st.strike);
    }
    if blockquote {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            let _ = ui.label(
                RichText::new(job.text.clone())
                    .italics()
                    .color(egui::Color32::from_rgb(180, 180, 200)),
            );
        });
    } else {
        ui.label(job);
    }
    ui.add_space(if heading.is_some() { 6.0 } else { 4.0 });
}

pub fn render_markdown(ui: &mut Ui, text: &str) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, options);

    let mut spans: Vec<(String, SpanStyle)> = Vec::new();
    let mut style = SpanStyle::default();
    let mut heading: Option<u8> = None;
    let mut blockquote = false;
    let mut in_code_block = false;
    let mut code_buf = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                heading = Some(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                heading = None;
                ui.add_space(2.0);
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
            }
            Event::Start(Tag::Strong) => style.bold = true,
            Event::End(TagEnd::Strong) => style.bold = false,
            Event::Start(Tag::Emphasis) => style.italic = true,
            Event::End(TagEnd::Emphasis) => style.italic = false,
            Event::Start(Tag::Strikethrough) => style.strike = true,
            Event::End(TagEnd::Strikethrough) => style.strike = false,
            Event::Start(Tag::CodeBlock(_)) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                in_code_block = true;
                code_buf.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let code = code_buf.clone();
                // render code block
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(
                        RichText::new(code)
                            .monospace()
                            .color(egui::Color32::from_rgb(220, 220, 220)),
                    );
                });
                ui.add_space(6.0);
                code_buf.clear();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                blockquote = true;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                blockquote = false;
            }
            Event::Start(Tag::List(first)) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                list_stack.push(first);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
                ui.add_space(2.0);
            }
            Event::Start(Tag::Item) => {
                let depth = list_stack.len().saturating_sub(1);
                let indent = "  ".repeat(depth);
                let prefix = if let Some(Some(n)) = list_stack.last_mut() {
                    let p = format!("{}{}. ", indent, n);
                    *n += 1;
                    p
                } else {
                    format!("{}• ", indent)
                };
                spans.push((prefix, SpanStyle::default()));
            }
            Event::End(TagEnd::Item) => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
            }
            Event::Start(Tag::Link { .. }) => {
                // mark as link color via code? keep simple
            }
            Event::End(TagEnd::Link) => {}
            Event::Code(t) => {
                let mut s = style.clone();
                s.code = true;
                spans.push((t.to_string(), s));
            }
            Event::Text(t) => {
                if in_code_block {
                    code_buf.push_str(&t);
                } else {
                    spans.push((t.to_string(), style.clone()));
                }
            }
            Event::SoftBreak => {
                if !in_code_block {
                    spans.push((" ".to_string(), style.clone()));
                } else {
                    code_buf.push('\n');
                }
            }
            Event::HardBreak => {
                if !in_code_block {
                    spans.push(("\n".to_string(), style.clone()));
                } else {
                    code_buf.push('\n');
                }
            }
            Event::Rule => {
                flush_paragraph(ui, &mut spans, heading, blockquote);
                ui.separator();
                ui.add_space(4.0);
            }
            Event::TaskListMarker(checked) => {
                let mark = if checked { "[x] " } else { "[ ] " };
                spans.push((mark.to_string(), SpanStyle::default()));
            }
            _ => {}
        }
    }
    flush_paragraph(ui, &mut spans, heading, blockquote);
    // tables not specially rendered
}
