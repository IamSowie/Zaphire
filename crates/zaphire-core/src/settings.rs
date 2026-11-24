//! User settings, stored as TOML in the platform config directory
//! (`~/.config/zaphire/settings.toml`, `%APPDATA%\zaphire\settings.toml`).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zaphire_perf::Profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TabLayout {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    /// Follow the operating system.
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Background {
    /// Soft accent-tinted gradient.
    #[default]
    Gradient,
    /// One flat colour.
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub mode: ThemeMode,
    pub background: Background,
    /// Custom base colour (`#rrggbb`) for the browser frame; `None` uses the
    /// theme's default.
    pub background_color: Option<String>,
    /// Accent colour as `#rrggbb`.
    pub accent: String,
    /// 0.0 (opaque) – 1.0 (very see-through) for the glass panels.
    pub glass: f32,
    /// Corner radius of panels in px.
    pub radius: u8,
    /// Interface scale, 0.85 – 1.25.
    pub scale: f32,
    pub density: Density,
    pub reduce_motion: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            background: Background::Gradient,
            background_color: None,
            accent: "#3b82f6".into(),
            glass: 0.55,
            radius: 12,
            scale: 1.0,
            density: Density::Comfortable,
            reduce_motion: false,
        }
    }
}

impl Theme {
    /// Opaque ARGB colour painted behind web views and between split panes,
    /// so the window frame matches the UI. `system_dark` resolves
    /// [`ThemeMode::System`].
    pub fn frame_color(&self, system_dark: bool) -> u32 {
        if let Some(c) = self.background_color.as_deref().and_then(parse_hex) {
            return c;
        }
        let dark = match self.mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => system_dark,
        };
        if dark { 0xff0b1020 } else { 0xffeef1f7 }
    }
}

/// `#rrggbb` → opaque ARGB.
pub fn parse_hex(s: &str) -> Option<u32> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    u32::from_str_radix(h, 16).ok().map(|rgb| 0xff00_0000 | rgb)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub tab_layout: TabLayout,
    pub profile: Profile,
    pub adblock: bool,
    /// Search URL with `{}` for the query.
    pub search_engine: String,
    pub homepage: String,
    /// Width of the vertical sidebar in px.
    pub sidebar_width: u32,
    pub theme: Theme,
    /// Reopen the previous session's tabs on start (they load on demand).
    pub restore_session: bool,
    /// Show the bookmarks list in the sidebar.
    pub show_bookmarks: bool,
    /// Show extension buttons in the toolbar.
    pub show_extensions: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tab_layout: TabLayout::Vertical,
            profile: Profile::Balanced,
            adblock: true,
            search_engine: "https://duckduckgo.com/?q={}".into(),
            homepage: "zaphire://app/newtab".into(),
            sidebar_width: 280,
            theme: Theme::default(),
            restore_session: true,
            show_bookmarks: true,
            show_extensions: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("serialize: {0}")]
    Ser(#[from] toml::ser::Error),
}

impl Settings {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("zaphire")
    }

    pub fn default_path() -> PathBuf {
        Self::config_dir().join("settings.toml")
    }

    /// Load settings; a missing file gives defaults, a broken one is an error.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(toml::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write then rename so a crash never leaves a half-written file.
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, toml::to_string_pretty(self)?)?;
        std::fs::rename(tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_partial_files() {
        let dir = std::env::temp_dir().join(format!("zaphire-test-{}", std::process::id()));
        let path = dir.join("settings.toml");
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());

        let s = Settings {
            tab_layout: TabLayout::Horizontal,
            profile: Profile::Game,
            ..Settings::default()
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), s);

        // Unknown fields missing → defaults fill them in.
        std::fs::write(&path, "adblock = false\n").unwrap();
        let s = Settings::load(&path).unwrap();
        assert!(!s.adblock);
        assert_eq!(s.tab_layout, TabLayout::Vertical);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn theme_frame_colour() {
        let mut t = Theme::default();
        assert_eq!(t.frame_color(false), 0xff0b1020);
        t.mode = ThemeMode::Light;
        assert_eq!(t.frame_color(true), 0xffeef1f7);
        t.mode = ThemeMode::System;
        assert_eq!(t.frame_color(true), 0xff0b1020);
        t.background_color = Some("#123456".into());
        assert_eq!(t.frame_color(true), 0xff123456);
        t.background_color = Some("nope".into());
        assert_eq!(t.frame_color(false), 0xffeef1f7);
    }

    #[test]
    fn old_theme_tables_still_load() {
        let s: Settings = toml::from_str("[theme]\naccent = \"#ff0000\"\n").unwrap();
        assert_eq!(s.theme.accent, "#ff0000");
        assert_eq!(s.theme.mode, ThemeMode::Dark);
        assert_eq!(s.theme.background, Background::Gradient);
        assert!(s.restore_session);
    }
}
