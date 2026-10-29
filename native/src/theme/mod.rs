//! Design tokens for the native UI.
//!
//! Every value here is copied from the tokens in the archived frontend
//! (`deprecated/tauri-ui/src/styles.css`) so the native application keeps the
//! same visual identity: neutral charcoal chrome with a restrained blue accent
//! in the dark theme, and warm paper tones in the light theme.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

pub mod fonts;
pub mod icons;

pub use icons::Icon;

/// Which chrome palette is active.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeKind {
    #[default]
    Dark,
    Light,
}

impl ThemeKind {
    pub fn toggled(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Spacing scale, in points.
///
/// Kept complete even where a value is not referenced yet: these are the design
/// tokens the shell is built from.
#[allow(dead_code)]
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 10.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;

    pub const TITLEBAR: f32 = 44.0;
    pub const MENUBAR: f32 = 28.0;
    pub const TABSTRIP: f32 = 34.0;
    pub const TOOLBAR: f32 = 36.0;
    pub const STATUS: f32 = 26.0;

    pub const SIDEBAR_DEFAULT: f32 = 264.0;
    pub const SIDEBAR_MIN: f32 = 208.0;
    pub const SIDEBAR_MAX: f32 = 420.0;
    pub const CONTEXT_DEFAULT: f32 = 300.0;
}

/// Corner radii, in points, matching `--nc-r-*`.
#[allow(dead_code)]
pub mod radius {
    pub const XS: u8 = 4;
    pub const SM: u8 = 6;
    pub const MD: u8 = 9;
    pub const LG: u8 = 13;
}

/// A fully resolved palette.
#[derive(Debug, Clone)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub elev: Color32,
    pub raised: Color32,
    pub muted: Color32,
    pub hover: Color32,

    pub fg: Color32,
    pub fg_secondary: Color32,
    pub fg_muted: Color32,
    pub fg_faint: Color32,

    pub border: Color32,
    pub border_strong: Color32,

    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_fg: Color32,

    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,

    pub code_bg: Color32,
    pub selection: Color32,
}

impl Palette {
    pub fn dark(accent: Color32) -> Self {
        Self {
            bg: Color32::from_rgb(0x0b, 0x0b, 0x0d),
            surface: Color32::from_rgb(0x10, 0x10, 0x12),
            elev: Color32::from_rgb(0x15, 0x15, 0x17),
            raised: Color32::from_rgb(0x1a, 0x1a, 0x1d),
            muted: Color32::from_rgb(0x21, 0x21, 0x24),
            hover: Color32::from_rgb(0x29, 0x29, 0x2e),
            fg: Color32::from_rgb(0xe9, 0xe9, 0xec),
            fg_secondary: Color32::from_rgb(0xa6, 0xa6, 0xae),
            fg_muted: Color32::from_rgb(0x82, 0x82, 0x8b),
            fg_faint: Color32::from_rgb(0x59, 0x59, 0x61),
            border: Color32::from_rgb(0x23, 0x23, 0x28),
            border_strong: Color32::from_rgb(0x33, 0x33, 0x3a),
            accent,
            accent_hover: Color32::from_rgb(0x6b, 0x9b, 0xff),
            accent_fg: Color32::WHITE,
            success: Color32::from_rgb(0x34, 0xd3, 0x99),
            warning: Color32::from_rgb(0xfb, 0xbf, 0x24),
            danger: Color32::from_rgb(0xf8, 0x71, 0x71),
            code_bg: Color32::from_rgb(0x0d, 0x0d, 0x0f),
            selection: Color32::from_rgba_unmultiplied(0x4f, 0x86, 0xf7, 0x3d),
        }
    }

    pub fn light(accent: Color32) -> Self {
        Self {
            bg: Color32::from_rgb(0xfa, 0xf7, 0xf1),
            surface: Color32::from_rgb(0xff, 0xfd, 0xf8),
            elev: Color32::WHITE,
            raised: Color32::from_rgb(0xff, 0xfe, 0xfa),
            muted: Color32::from_rgb(0xf2, 0xec, 0xe2),
            hover: Color32::from_rgb(0xea, 0xe3, 0xd7),
            fg: Color32::from_rgb(0x1b, 0x19, 0x17),
            fg_secondary: Color32::from_rgb(0x57, 0x53, 0x4e),
            fg_muted: Color32::from_rgb(0x7a, 0x73, 0x6b),
            fg_faint: Color32::from_rgb(0xa9, 0xa2, 0x9a),
            border: Color32::from_rgb(0xe7, 0xdf, 0xd3),
            border_strong: Color32::from_rgb(0xd7, 0xcc, 0xbb),
            accent,
            accent_hover: Color32::from_rgb(0x1a, 0x5f, 0xcc),
            accent_fg: Color32::WHITE,
            success: Color32::from_rgb(0x05, 0x96, 0x69),
            warning: Color32::from_rgb(0xb4, 0x53, 0x09),
            danger: Color32::from_rgb(0xdc, 0x26, 0x26),
            code_bg: Color32::from_rgb(0xf5, 0xef, 0xe5),
            selection: Color32::from_rgba_unmultiplied(0x1f, 0x6f, 0xeb, 0x33),
        }
    }

    pub fn accent_soft(&self, factor: f32) -> Color32 {
        let a = (255.0 * factor).clamp(0.0, 255.0) as u8;
        Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), a)
    }

    pub fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.border)
    }

    pub fn is_dark(&self) -> bool {
        self.bg.r() < 128
    }
}

/// Parse `#rrggbb` (the format settings stores) into a colour.
pub fn parse_hex_color(text: &str, fallback: Color32) -> Color32 {
    let t = text.trim().trim_start_matches('#');
    if t.len() != 6 {
        return fallback;
    }
    let r = u8::from_str_radix(&t[0..2], 16);
    let g = u8::from_str_radix(&t[2..4], 16);
    let b = u8::from_str_radix(&t[4..6], 16);
    match (r, g, b) {
        (Ok(r), Ok(g), Ok(b)) => Color32::from_rgb(r, g, b),
        _ => fallback,
    }
}

pub fn to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

/// The application theme: palette plus the typography metrics the user can tune.
#[derive(Debug, Clone)]
pub struct Theme {
    pub kind: ThemeKind,
    pub palette: Palette,
    pub editor_font_size: f32,
    pub editor_line_height: f32,
    pub reading_width: f32,
    pub reduced_motion: bool,
    pub fonts: fonts::Availability,
}

impl Theme {
    #[allow(dead_code)]
    pub fn build(kind: ThemeKind, accent: &str, settings: &crate::settings::Settings) -> Self {
        let default_accent = match kind {
            ThemeKind::Dark => Color32::from_rgb(0x4f, 0x86, 0xf7),
            ThemeKind::Light => Color32::from_rgb(0x1f, 0x6f, 0xeb),
        };
        let accent = parse_hex_color(accent, default_accent);
        Self {
            kind,
            palette: match kind {
                ThemeKind::Dark => Palette::dark(accent),
                ThemeKind::Light => Palette::light(accent),
            },
            editor_font_size: settings.editor_font_size as f32,
            editor_line_height: settings.editor_line_height.max(1.2),
            reading_width: settings.reading_width as f32,
            reduced_motion: settings.reduced_motion,
            fonts: fonts::Availability::default(),
        }
    }

    /// A font id for the requested size and emphasis, using real bold/italic
    /// faces when the machine provides them.
    pub fn font(&self, size: f32, bold: bool, italic: bool) -> FontId {
        let family = match self.fonts.emphasis(bold, italic) {
            Some(name) => FontFamily::Name(name.into()),
            None => FontFamily::Proportional,
        };
        FontId::new(size, family)
    }

    pub fn mono(&self, size: f32, bold: bool) -> FontId {
        let family = if bold && self.fonts.mono_bold {
            FontFamily::Name(fonts::FAMILY_MONO_BOLD.into())
        } else {
            FontFamily::Monospace
        };
        FontId::new(size, family)
    }

    #[allow(dead_code)]
    pub fn ui_font(&self) -> FontId {
        FontId::new(13.0, FontFamily::Proportional)
    }

    #[allow(dead_code)]
    pub fn small(&self) -> FontId {
        FontId::new(11.0, FontFamily::Proportional)
    }

    pub fn tiny(&self) -> FontId {
        FontId::new(10.0, FontFamily::Proportional)
    }

    #[allow(dead_code)]
    pub fn editor_font(&self, mono: bool) -> FontId {
        let family = if mono {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        };
        FontId::new(self.editor_font_size, family)
    }

    pub fn row_height(&self) -> f32 {
        (self.editor_font_size * self.editor_line_height).round()
    }
}

/// Install the palette and typography into an egui context.
pub fn apply(ctx: &egui::Context, theme: &Theme) {
    let p = &theme.palette;
    let egui_theme = match theme.kind {
        ThemeKind::Dark => egui::Theme::Dark,
        ThemeKind::Light => egui::Theme::Light,
    };
    let mut style = (*ctx.style_of(egui_theme)).clone();

    style.spacing.item_spacing = egui::vec2(space::SM, space::SM);
    style.spacing.button_padding = egui::vec2(space::MD, 4.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.indent = 14.0;
    style.spacing.interact_size.y = 24.0;
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.floating = false;
    if theme.reduced_motion {
        // Respect the preference instead of animating anyway.
        style.animation_time = 0.0;
    }

    let visuals = &mut style.visuals;
    visuals.dark_mode = p.is_dark();
    visuals.override_text_color = Some(p.fg);
    visuals.panel_fill = p.bg;
    visuals.window_fill = p.elev;
    visuals.extreme_bg_color = p.code_bg;
    visuals.faint_bg_color = p.raised;
    visuals.code_bg_color = p.code_bg;
    visuals.selection.bg_fill = p.selection;
    visuals.selection.stroke = Stroke::new(1.0, p.accent);
    visuals.window_stroke = Stroke::new(1.0, p.border_strong);
    visuals.window_corner_radius = CornerRadius::same(radius::LG);
    visuals.menu_corner_radius = CornerRadius::same(radius::MD);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(110),
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.hyperlink_color = p.accent;
    visuals.warn_fg_color = p.warning;
    visuals.error_fg_color = p.danger;

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.fg_secondary);
    w.noninteractive.corner_radius = CornerRadius::same(radius::SM);

    w.inactive.bg_fill = Color32::TRANSPARENT;
    w.inactive.weak_bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke = Stroke::new(1.0, p.fg_secondary);
    w.inactive.corner_radius = CornerRadius::same(radius::SM);

    w.hovered.bg_fill = p.hover;
    w.hovered.weak_bg_fill = p.hover;
    w.hovered.bg_stroke = Stroke::new(1.0, p.border_strong);
    w.hovered.fg_stroke = Stroke::new(1.0, p.fg);
    w.hovered.corner_radius = CornerRadius::same(radius::SM);

    w.active.bg_fill = p.muted;
    w.active.weak_bg_fill = p.muted;
    w.active.bg_stroke = Stroke::new(1.0, p.accent);
    w.active.fg_stroke = Stroke::new(1.0, p.fg);
    w.active.corner_radius = CornerRadius::same(radius::SM);

    w.open.bg_fill = p.muted;
    w.open.weak_bg_fill = p.muted;
    w.open.bg_stroke = Stroke::new(1.0, p.border_strong);
    w.open.fg_stroke = Stroke::new(1.0, p.fg);
    w.open.corner_radius = CornerRadius::same(radius::SM);

    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(17.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        ),
    ]
    .into();

    // Popups follow the same palette whichever system theme is active.
    let other = match egui_theme {
        egui::Theme::Dark => egui::Theme::Light,
        egui::Theme::Light => egui::Theme::Dark,
    };
    ctx.set_style_of(egui_theme, style.clone());
    ctx.set_style_of(other, style);
}
