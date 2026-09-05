//! Theme application via `AdwStyleManager`.

use adw::ColorScheme;

use crate::domain::Theme;

pub fn apply_theme(theme: Theme) {
    let scheme = match theme {
        Theme::System => ColorScheme::Default,
        Theme::Light => ColorScheme::ForceLight,
        Theme::Dark => ColorScheme::ForceDark,
    };
    adw::StyleManager::default().set_color_scheme(scheme);
}

pub fn theme_from_index(index: u32) -> Theme {
    match index {
        1 => Theme::Light,
        2 => Theme::Dark,
        _ => Theme::System,
    }
}

pub fn theme_to_index(theme: Theme) -> u32 {
    match theme {
        Theme::System => 0,
        Theme::Light => 1,
        Theme::Dark => 2,
    }
}