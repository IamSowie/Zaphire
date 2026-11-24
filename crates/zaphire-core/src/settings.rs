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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// Accent colour as `#rrggbb`.
    pub accent: String,
    /// 0.0 (opaque) – 1.0 (very see-through) for the glass panels.
    pub glass: f32,
    /// Corner radius of panels in px.
    pub radius: u8,
    pub reduce_motion: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#3b82f6".into(),
            glass: 0.55,
            radius: 12,
            reduce_motion: false,
        }
    }
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

        let mut s = Settings::default();
        s.tab_layout = TabLayout::Horizontal;
        s.profile = Profile::Game;
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), s);

        // Unknown fields missing → defaults fill them in.
        std::fs::write(&path, "adblock = false\n").unwrap();
        let s = Settings::load(&path).unwrap();
        assert!(!s.adblock);
        assert_eq!(s.tab_layout, TabLayout::Vertical);
        std::fs::remove_dir_all(dir).ok();
    }
}
