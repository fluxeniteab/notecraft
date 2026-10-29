//! Vector icons.
//!
//! The archived interface used `lucide-react`. Instead of shipping an icon font
//! we draw the same 1.5px stroke shapes with egui's painter, so icons stay crisp
//! at any DPI and inherit the current text colour.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // the icon set is complete; not every glyph is used yet
pub enum Icon {
    ChevronRight,
    ChevronDown,
    Folder,
    FolderOpen,
    FileText,
    Plus,
    Search,
    Settings,
    Sun,
    Moon,
    PanelLeft,
    PanelRight,
    Save,
    X,
    Check,
    Alert,
    Dot,
    Lock,
    History,
    Link,
    Tag,
    List,
    Trash,
    Pencil,
    Bold,
    Italic,
    Code,
    Heading,
    Quote,
    Table,
    Command,
    Refresh,
    ArrowLeft,
    ArrowRight,
    Download,
    GripVertical,
}

impl Icon {
    fn segments(self) -> &'static [Seg] {
        match self {
            Icon::ChevronRight => &[
                Seg::Line((0.4, 0.2), (0.65, 0.5)),
                Seg::Line((0.65, 0.5), (0.4, 0.8)),
            ],
            Icon::ChevronDown => &[
                Seg::Line((0.2, 0.4), (0.5, 0.65)),
                Seg::Line((0.5, 0.65), (0.8, 0.4)),
            ],
            Icon::Folder => &[
                Seg::Line((0.12, 0.28), (0.42, 0.28)),
                Seg::Line((0.42, 0.28), (0.5, 0.38)),
                Seg::Line((0.5, 0.38), (0.88, 0.38)),
                Seg::Rect((0.12, 0.28), (0.88, 0.82)),
            ],
            Icon::FolderOpen => &[
                Seg::Line((0.12, 0.3), (0.44, 0.3)),
                Seg::Line((0.44, 0.3), (0.52, 0.4)),
                Seg::Line((0.52, 0.4), (0.86, 0.4)),
                Seg::Line((0.12, 0.3), (0.12, 0.82)),
                Seg::Line((0.12, 0.82), (0.88, 0.82)),
                Seg::Line((0.88, 0.82), (0.74, 0.42)),
            ],
            Icon::FileText => &[
                Seg::Rect((0.22, 0.12), (0.78, 0.88)),
                Seg::Line((0.34, 0.36), (0.66, 0.36)),
                Seg::Line((0.34, 0.5), (0.66, 0.5)),
                Seg::Line((0.34, 0.64), (0.56, 0.64)),
            ],
            Icon::Plus => &[
                Seg::Line((0.5, 0.2), (0.5, 0.8)),
                Seg::Line((0.2, 0.5), (0.8, 0.5)),
            ],
            Icon::Search => &[
                Seg::Circle((0.44, 0.44), 0.24),
                Seg::Line((0.62, 0.62), (0.84, 0.84)),
            ],
            Icon::Settings => &[
                Seg::Circle((0.5, 0.5), 0.16),
                Seg::Circle((0.5, 0.5), 0.33),
                Seg::Line((0.5, 0.1), (0.5, 0.17)),
                Seg::Line((0.5, 0.83), (0.5, 0.9)),
                Seg::Line((0.1, 0.5), (0.17, 0.5)),
                Seg::Line((0.83, 0.5), (0.9, 0.5)),
                Seg::Line((0.22, 0.22), (0.27, 0.27)),
                Seg::Line((0.73, 0.73), (0.78, 0.78)),
                Seg::Line((0.78, 0.22), (0.73, 0.27)),
                Seg::Line((0.27, 0.73), (0.22, 0.78)),
            ],
            Icon::Sun => &[
                Seg::Circle((0.5, 0.5), 0.2),
                Seg::Line((0.5, 0.06), (0.5, 0.16)),
                Seg::Line((0.5, 0.84), (0.5, 0.94)),
                Seg::Line((0.06, 0.5), (0.16, 0.5)),
                Seg::Line((0.84, 0.5), (0.94, 0.5)),
                Seg::Line((0.18, 0.18), (0.25, 0.25)),
                Seg::Line((0.75, 0.75), (0.82, 0.82)),
                Seg::Line((0.82, 0.18), (0.75, 0.25)),
                Seg::Line((0.25, 0.75), (0.18, 0.82)),
            ],
            Icon::Moon => &[Seg::Arc((0.55, 0.5), 0.34)],
            Icon::PanelLeft => &[
                Seg::Rect((0.12, 0.18), (0.88, 0.82)),
                Seg::Line((0.38, 0.18), (0.38, 0.82)),
            ],
            Icon::PanelRight => &[
                Seg::Rect((0.12, 0.18), (0.88, 0.82)),
                Seg::Line((0.62, 0.18), (0.62, 0.82)),
            ],
            Icon::Save => &[
                Seg::Rect((0.18, 0.18), (0.82, 0.82)),
                Seg::Line((0.34, 0.18), (0.34, 0.42)),
                Seg::Line((0.34, 0.42), (0.66, 0.42)),
                Seg::Line((0.66, 0.18), (0.66, 0.42)),
                Seg::Rect((0.34, 0.58), (0.66, 0.82)),
            ],
            Icon::X => &[
                Seg::Line((0.26, 0.26), (0.74, 0.74)),
                Seg::Line((0.74, 0.26), (0.26, 0.74)),
            ],
            Icon::Check => &[
                Seg::Line((0.22, 0.52), (0.42, 0.72)),
                Seg::Line((0.42, 0.72), (0.78, 0.3)),
            ],
            Icon::Alert => &[
                Seg::Line((0.5, 0.16), (0.88, 0.82)),
                Seg::Line((0.88, 0.82), (0.12, 0.82)),
                Seg::Line((0.12, 0.82), (0.5, 0.16)),
                Seg::Line((0.5, 0.42), (0.5, 0.62)),
            ],
            Icon::Dot => &[Seg::Disc((0.5, 0.5), 0.16)],
            Icon::Lock => &[
                Seg::Rect((0.2, 0.46), (0.8, 0.86)),
                Seg::ArcUp((0.5, 0.46), 0.18),
            ],
            Icon::History => &[
                Seg::Circle((0.5, 0.5), 0.34),
                Seg::Line((0.5, 0.3), (0.5, 0.52)),
                Seg::Line((0.5, 0.52), (0.66, 0.62)),
            ],
            Icon::Link => &[
                Seg::Line((0.42, 0.34), (0.58, 0.18)),
                Seg::Line((0.58, 0.18), (0.82, 0.42)),
                Seg::Line((0.82, 0.42), (0.66, 0.58)),
                Seg::Line((0.58, 0.66), (0.42, 0.82)),
                Seg::Line((0.42, 0.82), (0.18, 0.58)),
                Seg::Line((0.18, 0.58), (0.34, 0.42)),
            ],
            Icon::Tag => &[
                Seg::Line((0.18, 0.18), (0.5, 0.18)),
                Seg::Line((0.5, 0.18), (0.86, 0.54)),
                Seg::Line((0.86, 0.54), (0.54, 0.86)),
                Seg::Line((0.54, 0.86), (0.18, 0.5)),
                Seg::Line((0.18, 0.5), (0.18, 0.18)),
                Seg::Disc((0.34, 0.34), 0.05),
            ],
            Icon::List => &[
                Seg::Line((0.34, 0.26), (0.86, 0.26)),
                Seg::Line((0.34, 0.5), (0.86, 0.5)),
                Seg::Line((0.34, 0.74), (0.86, 0.74)),
                Seg::Disc((0.18, 0.26), 0.05),
                Seg::Disc((0.18, 0.5), 0.05),
                Seg::Disc((0.18, 0.74), 0.05),
            ],
            Icon::Trash => &[
                Seg::Line((0.16, 0.28), (0.84, 0.28)),
                Seg::Line((0.38, 0.28), (0.4, 0.16)),
                Seg::Line((0.6, 0.28), (0.62, 0.16)),
                Seg::Line((0.4, 0.16), (0.62, 0.16)),
                Seg::Line((0.26, 0.28), (0.32, 0.86)),
                Seg::Line((0.74, 0.28), (0.68, 0.86)),
                Seg::Line((0.32, 0.86), (0.68, 0.86)),
            ],
            Icon::Pencil => &[
                Seg::Line((0.2, 0.8), (0.3, 0.58)),
                Seg::Line((0.3, 0.58), (0.7, 0.18)),
                Seg::Line((0.7, 0.18), (0.84, 0.32)),
                Seg::Line((0.84, 0.32), (0.44, 0.72)),
                Seg::Line((0.44, 0.72), (0.2, 0.8)),
            ],
            Icon::Bold => &[
                Seg::Line((0.32, 0.2), (0.32, 0.8)),
                Seg::ArcRight((0.32, 0.34), 0.16),
                Seg::ArcRight((0.32, 0.64), 0.17),
            ],
            Icon::Italic => &[
                Seg::Line((0.42, 0.2), (0.68, 0.2)),
                Seg::Line((0.34, 0.8), (0.6, 0.8)),
                Seg::Line((0.58, 0.2), (0.42, 0.8)),
            ],
            Icon::Code => &[
                Seg::Line((0.36, 0.3), (0.16, 0.5)),
                Seg::Line((0.16, 0.5), (0.36, 0.7)),
                Seg::Line((0.64, 0.3), (0.84, 0.5)),
                Seg::Line((0.84, 0.5), (0.64, 0.7)),
            ],
            Icon::Heading => &[
                Seg::Line((0.26, 0.2), (0.26, 0.8)),
                Seg::Line((0.72, 0.2), (0.72, 0.8)),
                Seg::Line((0.26, 0.5), (0.72, 0.5)),
            ],
            Icon::Quote => &[
                Seg::Line((0.22, 0.24), (0.22, 0.76)),
                Seg::Line((0.34, 0.3), (0.76, 0.3)),
                Seg::Line((0.34, 0.5), (0.7, 0.5)),
                Seg::Line((0.34, 0.7), (0.6, 0.7)),
            ],
            Icon::Table => &[
                Seg::Rect((0.14, 0.22), (0.86, 0.78)),
                Seg::Line((0.14, 0.42), (0.86, 0.42)),
                Seg::Line((0.44, 0.42), (0.44, 0.78)),
            ],
            Icon::Command => &[
                Seg::ArcDown((0.3, 0.3), 0.12),
                Seg::ArcDown((0.7, 0.3), 0.12),
                Seg::ArcUp((0.3, 0.7), 0.12),
                Seg::ArcUp((0.7, 0.7), 0.12),
                Seg::Line((0.42, 0.3), (0.58, 0.3)),
                Seg::Line((0.42, 0.7), (0.58, 0.7)),
                Seg::Line((0.3, 0.42), (0.3, 0.58)),
                Seg::Line((0.7, 0.42), (0.7, 0.58)),
            ],
            Icon::Refresh => &[
                Seg::Arc((0.5, 0.5), 0.32),
                Seg::Line((0.78, 0.18), (0.82, 0.44)),
                Seg::Line((0.82, 0.44), (0.58, 0.4)),
            ],
            Icon::ArrowLeft => &[
                Seg::Line((0.7, 0.24), (0.34, 0.5)),
                Seg::Line((0.34, 0.5), (0.7, 0.76)),
                Seg::Line((0.34, 0.5), (0.88, 0.5)),
            ],
            Icon::ArrowRight => &[
                Seg::Line((0.3, 0.24), (0.66, 0.5)),
                Seg::Line((0.66, 0.5), (0.3, 0.76)),
                Seg::Line((0.12, 0.5), (0.66, 0.5)),
            ],
            Icon::Download => &[
                Seg::Line((0.5, 0.16), (0.5, 0.62)),
                Seg::Line((0.28, 0.44), (0.5, 0.66)),
                Seg::Line((0.72, 0.44), (0.5, 0.66)),
                Seg::Line((0.18, 0.82), (0.82, 0.82)),
            ],
            Icon::GripVertical => &[
                Seg::Disc((0.4, 0.3), 0.045),
                Seg::Disc((0.6, 0.3), 0.045),
                Seg::Disc((0.4, 0.5), 0.045),
                Seg::Disc((0.6, 0.5), 0.045),
                Seg::Disc((0.4, 0.7), 0.045),
                Seg::Disc((0.6, 0.7), 0.045),
            ],
        }
    }
}

enum Seg {
    Line((f32, f32), (f32, f32)),
    Rect((f32, f32), (f32, f32)),
    Circle((f32, f32), f32),
    Disc((f32, f32), f32),
    /// Three-quarter arc, used for "moon" and "refresh".
    Arc((f32, f32), f32),
    /// Upper half arc, used for padlocks and letter bowls.
    ArcUp((f32, f32), f32),
    /// Lower half arc.
    ArcDown((f32, f32), f32),
    /// Right half arc.
    ArcRight((f32, f32), f32),
}

fn map_point(rect: Rect, x: f32, y: f32) -> Pos2 {
    Pos2::new(
        rect.left() + rect.width() * x,
        rect.top() + rect.height() * y,
    )
}

fn arc_points(center: Pos2, radius: f32, from: f32, to: f32, steps: usize) -> Vec<Pos2> {
    (0..=steps)
        .map(|i| {
            let t = from + (to - from) * (i as f32 / steps as f32);
            Pos2::new(center.x + radius * t.cos(), center.y + radius * t.sin())
        })
        .collect()
}

/// Paint `icon` inside `rect`.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    let unit = rect.width().min(rect.height());
    for seg in icon.segments() {
        match seg {
            Seg::Line(a, b) => {
                painter.line_segment(
                    [map_point(rect, a.0, a.1), map_point(rect, b.0, b.1)],
                    stroke,
                );
            }
            Seg::Rect(a, b) => {
                let r = Rect::from_min_max(map_point(rect, a.0, a.1), map_point(rect, b.0, b.1));
                painter.rect_stroke(
                    r,
                    egui::CornerRadius::same(2),
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
            Seg::Circle(c, radius) => {
                painter.circle_stroke(map_point(rect, c.0, c.1), radius * unit, stroke);
            }
            Seg::Disc(c, radius) => {
                painter.circle_filled(map_point(rect, c.0, c.1), radius * unit, color);
            }
            Seg::Arc((cx, cy), radius) => {
                let center = map_point(rect, *cx, *cy);
                let pts = arc_points(center, radius * unit, -1.1, 2.3, 22);
                painter.add(egui::Shape::line(pts, stroke));
            }
            Seg::ArcUp((cx, cy), radius) => {
                let center = map_point(rect, *cx, *cy);
                let pts = arc_points(
                    center,
                    radius * unit,
                    std::f32::consts::PI,
                    std::f32::consts::TAU,
                    18,
                );
                painter.add(egui::Shape::line(pts, stroke));
            }
            Seg::ArcDown((cx, cy), radius) => {
                let center = map_point(rect, *cx, *cy);
                let pts = arc_points(center, radius * unit, 0.0, std::f32::consts::PI, 18);
                painter.add(egui::Shape::line(pts, stroke));
            }
            Seg::ArcRight((cx, cy), radius) => {
                let center = map_point(rect, *cx, *cy);
                let pts = arc_points(
                    center,
                    radius * unit,
                    -std::f32::consts::FRAC_PI_2,
                    std::f32::consts::FRAC_PI_2,
                    18,
                );
                painter.add(egui::Shape::line(pts, stroke));
            }
        }
    }
}

/// Allocate `size` points and paint the icon centred in it.
pub fn show(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect, icon, color, (size / 14.0).max(1.0));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_geometry() {
        let icons = [
            Icon::ChevronRight,
            Icon::ChevronDown,
            Icon::Folder,
            Icon::FileText,
            Icon::Plus,
            Icon::Search,
            Icon::Settings,
            Icon::Sun,
            Icon::Moon,
            Icon::Lock,
            Icon::Check,
            Icon::Alert,
            Icon::X,
            Icon::Save,
        ];
        for icon in icons {
            assert!(!icon.segments().is_empty(), "{icon:?} must draw something");
        }
    }
}
