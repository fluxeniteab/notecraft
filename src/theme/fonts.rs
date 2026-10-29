//! Fonts.
//!
//! The archived UI asked for Inter / Segoe UI / system-ui, with JetBrains Mono /
//! Cascadia Code / Consolas for code. None of those are vendored in the
//! repository, so we load the closest faces the operating system already has
//! (Segoe UI + Consolas on Windows, SF/Menlo on macOS, DejaVu on Linux) and fall
//! back to egui's bundled fonts when nothing is found.
//!
//! Bold and italic get their own families, because a reading view without real
//! bold/italic is a visible regression from the original typography.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// Candidate UI fonts, best first, per platform.
const UI_UI: &[&str] = &[
    "C:/Windows/Fonts/segoeui.ttf",
    "/System/Library/Fonts/SFNS.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

const UI_BOLD: &[&str] = &[
    "C:/Windows/Fonts/segoeuib.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
];

const UI_ITALIC: &[&str] = &[
    "C:/Windows/Fonts/segoeuii.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
];

const UI_BOLD_ITALIC: &[&str] = &[
    "C:/Windows/Fonts/segoeuiz.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf",
];

/// Candidate monospace fonts, best first, per platform.
const MONO_REGULAR: &[&str] = &[
    "C:/Windows/Fonts/CascadiaCode.ttf",
    "C:/Windows/Fonts/consola.ttf",
    "/System/Library/Fonts/Menlo.ttc",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
];

const MONO_BOLD: &[&str] = &[
    "C:/Windows/Fonts/consolab.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf",
];

pub const FAMILY_BOLD: &str = "nc-bold";
pub const FAMILY_ITALIC: &str = "nc-italic";
pub const FAMILY_BOLD_ITALIC: &str = "nc-bold-italic";
pub const FAMILY_MONO_BOLD: &str = "nc-mono-bold";

/// Which optional families this machine actually provides.
///
/// Asking for a family that was never registered would render nothing, so the
/// theme checks this before choosing a face.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Availability {
    pub bold: bool,
    pub italic: bool,
    pub bold_italic: bool,
    pub mono_bold: bool,
    pub system_ui: bool,
    pub system_mono: bool,
}

impl Availability {
    /// The closest available face for the requested emphasis.
    pub fn emphasis(&self, bold: bool, italic: bool) -> Option<&'static str> {
        match (bold, italic) {
            (true, true) => {
                if self.bold_italic {
                    Some(FAMILY_BOLD_ITALIC)
                } else if self.bold {
                    Some(FAMILY_BOLD)
                } else if self.italic {
                    Some(FAMILY_ITALIC)
                } else {
                    None
                }
            }
            (true, false) => self.bold.then_some(FAMILY_BOLD),
            (false, true) => self.italic.then_some(FAMILY_ITALIC),
            (false, false) => None,
        }
    }
}

fn first_readable(candidates: &[&str]) -> Option<Vec<u8>> {
    candidates
        .iter()
        .find_map(|path| std::fs::read(path).ok().filter(|bytes| !bytes.is_empty()))
}

fn add_family(fonts: &mut FontDefinitions, name: &str, bytes: Vec<u8>, fallbacks: &[&str]) {
    let key = name.to_string();
    fonts
        .font_data
        .insert(key.clone(), Arc::new(FontData::from_owned(bytes)));
    let mut chain = vec![key];
    chain.extend(fallbacks.iter().map(|f| (*f).to_string()));
    fonts.families.insert(FontFamily::Name(name.into()), chain);
}

/// Install the fonts into a context and report what was found.
pub fn install(ctx: &egui::Context) -> Availability {
    let (fonts, availability) = build();
    ctx.set_fonts(fonts);
    availability
}

/// Build font definitions with the best system faces available.
pub fn build() -> (FontDefinitions, Availability) {
    let mut fonts = FontDefinitions::default();
    let mut availability = Availability::default();

    if let Some(bytes) = first_readable(UI_UI) {
        let name = "nc-ui".to_string();
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, name);
        }
        availability.system_ui = true;
    }

    // Bold/italic faces fall back to the regular proportional face, so the
    // family always resolves even on platforms without separate files.
    let proportional: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let fallbacks: Vec<&str> = proportional.iter().map(String::as_str).collect();

    if let Some(bytes) = first_readable(UI_BOLD) {
        add_family(&mut fonts, FAMILY_BOLD, bytes, &fallbacks);
        availability.bold = true;
    }
    if let Some(bytes) = first_readable(UI_ITALIC) {
        add_family(&mut fonts, FAMILY_ITALIC, bytes, &fallbacks);
        availability.italic = true;
    }
    if let Some(bytes) = first_readable(UI_BOLD_ITALIC) {
        add_family(&mut fonts, FAMILY_BOLD_ITALIC, bytes, &fallbacks);
        availability.bold_italic = true;
    }

    if let Some(bytes) = first_readable(MONO_REGULAR) {
        let name = "nc-mono".to_string();
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        if let Some(list) = fonts.families.get_mut(&FontFamily::Monospace) {
            list.insert(0, name);
        }
        availability.system_mono = true;
    }

    let mono: Vec<String> = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    let mono_fallbacks: Vec<&str> = mono.iter().map(String::as_str).collect();
    if let Some(bytes) = first_readable(MONO_BOLD) {
        add_family(&mut fonts, FAMILY_MONO_BOLD, bytes, &mono_fallbacks);
        availability.mono_bold = true;
    }

    (fonts, availability)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_families_are_never_empty() {
        let (fonts, _availability) = build();
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let list = fonts.families.get(&family).expect("family present");
            assert!(!list.is_empty(), "family {family:?} must keep a fallback");
        }
    }

    #[test]
    fn every_declared_family_resolves_when_registered() {
        let (fonts, _availability) = build();
        for name in [
            FAMILY_BOLD,
            FAMILY_ITALIC,
            FAMILY_BOLD_ITALIC,
            FAMILY_MONO_BOLD,
        ] {
            if let Some(list) = fonts.families.get(&FontFamily::Name(name.into())) {
                assert!(!list.is_empty(), "{name} must have a fallback chain");
            }
        }
    }

    #[test]
    fn emphasis_never_names_an_unavailable_family() {
        let none = Availability::default();
        assert_eq!(none.emphasis(true, false), None);
        assert_eq!(none.emphasis(false, true), None);
        let bold_only = Availability {
            bold: true,
            ..Default::default()
        };
        assert_eq!(bold_only.emphasis(true, true), Some(FAMILY_BOLD));
    }
}
