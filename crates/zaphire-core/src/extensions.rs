//! Installed Chrome extensions, read from the Chromium profile.
//!
//! The Chrome runtime keeps no public API for listing extensions, but it
//! records them in the profile's `Preferences` / `Secure Preferences` under
//! `extensions.settings.<id>`. Each entry has a `path` (absolute for unpacked
//! and command-line extensions, relative to `<profile>/Extensions` for store
//! installs) where `manifest.json` lives.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    /// Popup page relative to the extension root (`action.default_popup`).
    pub popup: Option<String>,
    /// Absolute path of the best icon (≈32 px), served by the core.
    #[serde(skip)]
    pub icon_path: Option<PathBuf>,
    pub has_icon: bool,
}

// Chromium `ManifestLocation` values that are part of the browser itself.
const LOCATION_COMPONENT: u64 = 5;
const LOCATION_EXTERNAL_COMPONENT: u64 = 10;

/// List enabled, user-visible extensions of the profile at `profile_dir`
/// (the `Default` folder). Unreadable entries are skipped.
pub fn installed(profile_dir: &Path) -> Vec<ExtensionInfo> {
    let mut out = vec![];
    for file in ["Secure Preferences", "Preferences"] {
        let Ok(text) = std::fs::read_to_string(profile_dir.join(file)) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(settings) = json
            .pointer("/extensions/settings")
            .and_then(Value::as_object)
        else {
            continue;
        };
        for (id, entry) in settings {
            if out.iter().any(|e: &ExtensionInfo| &e.id == id) {
                continue;
            }
            if let Some(info) = read_entry(profile_dir, id, entry) {
                out.push(info);
            }
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

fn read_entry(profile_dir: &Path, id: &str, entry: &Value) -> Option<ExtensionInfo> {
    let location = entry.get("location").and_then(Value::as_u64).unwrap_or(0);
    if location == LOCATION_COMPONENT || location == LOCATION_EXTERNAL_COMPONENT {
        return None;
    }
    let disabled = entry.get("disable_reasons").is_some_and(|r| {
        r.as_array().is_some_and(|a| !a.is_empty()) || r.as_u64().is_some_and(|n| n != 0)
    }) || entry.get("state").and_then(Value::as_u64) == Some(0);
    if disabled {
        return None;
    }
    let rel = entry.get("path").and_then(Value::as_str)?;
    let root = if Path::new(rel).is_absolute() {
        PathBuf::from(rel)
    } else {
        profile_dir.join("Extensions").join(rel)
    };
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).ok()?).ok()?;
    // Themes and apps have no toolbar presence.
    if manifest.get("theme").is_some() || manifest.get("app").is_some() {
        return None;
    }
    let action = manifest
        .get("action")
        .or_else(|| manifest.get("browser_action"));
    let popup = action
        .and_then(|a| a.get("default_popup"))
        .and_then(Value::as_str)
        .filter(|p| !p.is_empty())
        .map(|p| p.trim_start_matches('/').to_string());
    let icon_rel = action
        .and_then(|a| a.get("default_icon"))
        .and_then(best_icon)
        .or_else(|| manifest.get("icons").and_then(best_icon));
    let icon_path = icon_rel
        .map(|r| root.join(r.trim_start_matches('/')))
        .filter(|p| p.exists());
    let name = localized_name(&root, &manifest).unwrap_or_else(|| id.to_string());
    Some(ExtensionInfo {
        id: id.to_string(),
        name,
        popup,
        has_icon: icon_path.is_some(),
        icon_path,
    })
}

/// Icon closest to 32 px from a manifest icon value (string or size map).
fn best_icon(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    let map = v.as_object()?;
    map.iter()
        .filter_map(|(k, v)| Some((k.parse::<i64>().ok()?, v.as_str()?)))
        .min_by_key(|(size, _)| (size - 32).abs() + if *size < 32 { 100 } else { 0 })
        .map(|(_, p)| p.to_string())
}

/// Resolve `__MSG_name__` names through `_locales/<lang>/messages.json`.
fn localized_name(root: &Path, manifest: &Value) -> Option<String> {
    let name = manifest.get("name")?.as_str()?;
    let Some(key) = name
        .strip_prefix("__MSG_")
        .and_then(|n| n.strip_suffix("__"))
    else {
        return Some(name.to_string());
    };
    let default = manifest
        .get("default_locale")
        .and_then(Value::as_str)
        .unwrap_or("en");
    for lang in ["es", default, "en"] {
        let file = root.join("_locales").join(lang).join("messages.json");
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(msgs) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let found = msgs.as_object().and_then(|m| {
            m.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .and_then(|(_, v)| v.get("message")?.as_str())
        });
        if let Some(n) = found {
            return Some(n.to_string());
        }
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_profile_extensions() {
        let dir = std::env::temp_dir().join(format!("zaphire-ext-{}", std::process::id()));
        let profile = dir.join("Default");
        let unpacked = dir.join("unpacked");
        let store = profile.join("Extensions/abc/1.0_0");
        std::fs::create_dir_all(&unpacked).unwrap();
        std::fs::create_dir_all(store.join("_locales/en")).unwrap();
        std::fs::write(
            unpacked.join("manifest.json"),
            r#"{"name":"Dev tool","action":{"default_popup":"/p.html","default_icon":{"16":"i16.png","48":"i48.png"}}}"#,
        )
        .unwrap();
        std::fs::write(unpacked.join("i48.png"), b"png").unwrap();
        std::fs::write(
            store.join("manifest.json"),
            r#"{"name":"__MSG_appName__","default_locale":"en","browser_action":{}}"#,
        )
        .unwrap();
        std::fs::write(
            store.join("_locales/en/messages.json"),
            r#"{"appName":{"message":"uBlock"}}"#,
        )
        .unwrap();
        let prefs = serde_json::json!({"extensions":{"settings":{
            "dev": {"location": 4, "path": unpacked.to_string_lossy()},
            "abc": {"location": 1, "path": "abc/1.0_0"},
            "pdf": {"location": 5, "path": "/nowhere"},
            "off": {"location": 4, "path": unpacked.to_string_lossy(), "disable_reasons": [1]}
        }}});
        std::fs::write(profile.join("Preferences"), prefs.to_string()).unwrap();

        let list = installed(&profile);
        let names: Vec<_> = list.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Dev tool", "uBlock"]);
        assert_eq!(list[0].popup.as_deref(), Some("p.html"));
        assert_eq!(
            list[0].icon_path.as_ref().unwrap().file_name().unwrap(),
            "i48.png"
        );
        assert!(list[0].has_icon);
        assert_eq!(list[1].popup, None);
        std::fs::remove_dir_all(dir).ok();
    }
}
