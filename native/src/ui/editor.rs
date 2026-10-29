//! The editor.
//!
//! `egui::TextEdit` provides the editing behaviour people expect (selection,
//! IME, undo/redo, clipboard, keyboard navigation). We add the two things it
//! does not: per-line syntax highlighting through a custom layouter, and a line
//! number gutter for code and data documents.
//!
//! Prose documents wrap to the reading column like the original editor. Code and
//! data documents do not wrap, so the gutter and their long lines stay aligned.

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, CornerRadius, FontId, ScrollArea, Sense, Stroke, TextEdit, Ui, Vec2};

use crate::documents::Kind;
use crate::markdown::highlight::{self, Lang};
use crate::theme::Theme;

/// What the editor reports back to the application.
pub struct EditorOutput {
    pub changed: bool,
    /// 1-based cursor position.
    pub line: usize,
    pub column: usize,
}

fn lang_for(kind: Kind) -> Lang {
    Lang::from_syntax(kind.capabilities().syntax)
}

/// Convert a character index into a 1-based line and column.
pub fn line_col_at(text: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for (count, ch) in text.chars().enumerate() {
        if count >= char_index {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

/// Byte offset of the start of a 1-based line.
pub fn line_start(text: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    text.match_indices('\n')
        .nth(line - 2)
        .map(|(index, _)| index + 1)
        .unwrap_or(text.len())
}

fn prose_job(theme: &Theme, lang: Lang, line: &str, size: f32) -> LayoutJob {
    let p = &theme.palette;
    let tokens = highlight::tokenize(lang, line);
    let mut job = LayoutJob::default();
    let line_height = (size * theme.editor_line_height).round();

    let format = |role_color: Color32, bold: bool| TextFormat {
        font_id: theme.font(size, bold, false),
        color: role_color,
        line_height: Some(line_height),
        ..Default::default()
    };

    if tokens.is_empty() {
        job.append(line, 0.0, format(p.fg, false));
        return job;
    }
    let mut cursor = 0;
    for token in tokens {
        if token.start > cursor {
            job.append(&line[cursor..token.start], 0.0, format(p.fg, false));
        }
        let start = token.start.max(cursor);
        if token.end > start {
            let (color, bold, mono) = match token.role {
                highlight::Role::Heading => (p.fg, true, false),
                highlight::Role::Comment => (p.fg_faint, false, false),
                highlight::Role::Code => (p.success, false, true),
                highlight::Role::Link => (p.accent, false, false),
                highlight::Role::Mark => (p.warning, false, true),
                highlight::Role::Tag => (p.accent, false, true),
                highlight::Role::Emphasis => (p.fg_secondary, false, false),
                highlight::Role::Keyword => (p.accent, false, false),
                _ => (p.fg, false, false),
            };
            let mut fmt = TextFormat {
                font_id: if mono {
                    theme.mono(size * 0.94, false)
                } else {
                    theme.font(size, bold, false)
                },
                color,
                line_height: Some(line_height),
                ..Default::default()
            };
            if token.role == highlight::Role::Mark {
                fmt.background = p.code_bg;
            }
            job.append(&line[start..token.end], 0.0, fmt);
        }
        cursor = token.end.max(cursor);
    }
    if cursor < line.len() {
        job.append(&line[cursor..], 0.0, format(p.fg, false));
    }
    job
}

fn gutter(ui: &mut Ui, theme: &Theme, text: &str, size: f32, row_height: f32) {
    let p = &theme.palette;
    let line_count = text.lines().count().max(1);
    let digits = line_count.to_string().len().max(2);
    let width = (digits as f32 * size * 0.62 + 16.0).max(34.0);
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(width, line_count as f32 * row_height + row_height),
        Sense::hover(),
    );
    for index in 0..line_count {
        let y = rect.top() + (index as f32 + 0.5) * row_height;
        ui.painter().text(
            egui::pos2(rect.right() - 8.0, y),
            egui::Align2::RIGHT_CENTER,
            (index + 1).to_string(),
            FontId::monospace(size * 0.82),
            p.fg_faint,
        );
    }
}

/// Draw the editor for a document.
pub fn text_editor(
    ui: &mut Ui,
    theme: &Theme,
    id: egui::Id,
    text: &mut String,
    kind: Kind,
    show_line_numbers: bool,
) -> EditorOutput {
    let p = &theme.palette;
    let lang = lang_for(kind);
    let mono = kind.capabilities().mono;
    let font_size = theme.editor_font_size;
    let row_height = theme.row_height();
    let wrap_width = (ui.available_width() - 24.0).max(160.0);

    let max_line_chars = text
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let content_width = (max_line_chars as f32 * font_size * 0.62 + 48.0).max(wrap_width);

    let mut output: Option<EditorOutput> = None;

    let mut build = |ui: &mut Ui, text: &mut String| {
        let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, _wrap: f32| {
            let content = buffer.as_str().to_owned();
            let mut job = LayoutJob {
                wrap: TextWrapping {
                    max_width: if mono { f32::INFINITY } else { wrap_width },
                    ..Default::default()
                },
                ..Default::default()
            };
            for (index, line) in content.split('\n').enumerate() {
                if index > 0 {
                    job.append(
                        "\n",
                        0.0,
                        TextFormat {
                            font_id: if mono {
                                theme.mono(font_size, false)
                            } else {
                                theme.font(font_size, false, false)
                            },
                            color: p.fg,
                            line_height: Some(row_height),
                            ..Default::default()
                        },
                    );
                }
                if mono {
                    let row = highlight::highlight_row(theme, lang, line, font_size);
                    job.sections.extend(row.sections);
                    job.text.push_str(&row.text);
                } else {
                    let row = prose_job(theme, lang, line, font_size);
                    job.sections.extend(row.sections);
                    job.text.push_str(&row.text);
                }
            }
            let galley = ui.painter().layout_job(job);
            galley
        };

        let response = ui.add(
            TextEdit::multiline(text)
                .id(id)
                .desired_width(if mono { content_width } else { wrap_width })
                .desired_rows(24)
                .frame(egui::Frame::NONE)
                .margin(egui::Margin::symmetric(10, 8))
                .text_color(p.fg)
                .layouter(&mut layouter),
        );

        let (line, column) = cursor_position(ui, id, text);
        let changed = response.changed();
        output = Some(EditorOutput {
            changed,
            line,
            column,
        });
    };

    egui::Frame::new()
        .fill(p.bg)
        .inner_margin(egui::Margin::symmetric(0, 0))
        .show(ui, |ui| {
            if mono && show_line_numbers {
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            gutter(ui, theme, text, font_size, row_height);
                            let (line_rect, _) = ui.allocate_exact_size(
                                Vec2::new(1.0, ui.available_height()),
                                Sense::hover(),
                            );
                            ui.painter().line_segment(
                                [line_rect.left_top(), line_rect.left_bottom()],
                                Stroke::new(1.0, p.border),
                            );
                            build(ui, text);
                        });
                    });
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        build(ui, text);
                    });
            }
        });

    output.unwrap_or(EditorOutput {
        changed: false,
        line: 1,
        column: 1,
    })
}

fn cursor_position(ui: &Ui, id: egui::Id, text: &str) -> (usize, usize) {
    let index = egui::text_edit::TextEditState::load(ui.ctx(), id)
        .and_then(|state| state.cursor.char_range())
        .map(|range| range.primary.index.0)
        .unwrap_or(0);
    line_col_at(text, index)
}

/// A read-only code view used by the source preview of data documents.
pub fn code_view(ui: &mut Ui, theme: &Theme, id: egui::Id, text: &str, kind: Kind) {
    let p = &theme.palette;
    let lang = lang_for(kind);
    let size = theme.editor_font_size * 0.92;
    let job = highlight::highlight(
        theme,
        match lang {
            Lang::Json => "json",
            Lang::Yaml => "yaml",
            Lang::Html => "html",
            Lang::Css => "css",
            Lang::Code => "rust",
            _ => "",
        },
        text,
        size,
    );
    egui::Frame::new()
        .fill(p.code_bg)
        .corner_radius(CornerRadius::same(crate::theme::radius::SM))
        .stroke(Stroke::new(1.0, p.border))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ScrollArea::both()
                .id_salt(id)
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 8.0)
                .show(ui, |ui| {
                    ui.label(job);
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_positions_are_one_based() {
        assert_eq!(line_col_at("abc", 0), (1, 1));
        assert_eq!(line_col_at("abc", 2), (1, 3));
        assert_eq!(line_col_at("a\nbc", 2), (2, 1));
        assert_eq!(line_col_at("a\nbc", 3), (2, 2));
    }

    #[test]
    fn line_starts_are_correct() {
        let text = "one\ntwo\nthree";
        assert_eq!(line_start(text, 1), 0);
        assert_eq!(line_start(text, 2), 4);
        assert_eq!(line_start(text, 3), 8);
        assert_eq!(line_start(text, 99), text.len());
    }

    #[test]
    fn prose_jobs_split_roles() {
        let theme = Theme::build(
            crate::theme::ThemeKind::Dark,
            "#4f86f7",
            &crate::settings::Settings::default(),
        );
        let job = prose_job(&theme, Lang::Markdown, "## Title", 15.0);
        assert_eq!(job.text, "## Title");
        assert!(!job.sections.is_empty());
    }
}
