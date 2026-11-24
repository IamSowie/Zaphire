//! UI ⇄ core protocol.
//!
//! * UI → core: the React app `POST`s a JSON [`Command`] to
//!   `zaphire://app/api`. Keep `ui/src/ipc.ts` in sync with this file.
//! * core → UI: the core runs `window.__zaphire.receive(<Event JSON>)` in
//!   the UI frame.

use serde::{Deserialize, Serialize};
use zaphire_core::{Session, Settings, TabId, WorkspaceId};
use zaphire_perf::Profile;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    /// UI finished loading and wants the full state.
    Ready,
    NewTab {
        url: Option<String>,
    },
    CloseTab {
        id: TabId,
    },
    ActivateTab {
        id: TabId,
    },
    PinTab {
        id: TabId,
        pinned: bool,
    },
    /// Navigate a tab (active tab if `id` is `None`) to address-bar input.
    Navigate {
        id: Option<TabId>,
        input: String,
    },
    Back,
    Forward,
    Reload,
    Stop,
    /// Show `id` beside the active tab; `None` closes split view.
    SetSplit {
        id: Option<TabId>,
    },
    NewWorkspace {
        name: String,
        icon: String,
        color: String,
    },
    SwitchWorkspace {
        id: WorkspaceId,
    },
    RemoveWorkspace {
        id: WorkspaceId,
    },
    MoveTab {
        id: TabId,
        workspace: WorkspaceId,
        index: usize,
    },
    UpdateSettings {
        settings: Settings,
    },
    SetProfile {
        profile: Profile,
    },
    /// Open chrome://extensions in a Chrome-style window.
    OpenExtensions,
    DevTools,
    /// Restart the browser to apply startup switches.
    Restart,
    // Window controls for the frameless title bar.
    Minimize,
    ToggleMaximize,
    CloseWindow,
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Event<'a> {
    State {
        session: &'a Session,
        settings: &'a Settings,
        blocked: u64,
        /// The running profile differs from the saved one.
        restart_needed: bool,
        platform: &'static str,
    },
}

impl Command {
    pub fn parse(json: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(json)
    }
}

impl Event<'_> {
    /// JavaScript that delivers this event to the UI.
    pub fn to_js(&self) -> String {
        let json = serde_json::to_string(self).expect("event serializes");
        // JSON is valid JS; escape U+2028/9 which JSON allows but old JS did not.
        let json = json
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029");
        format!("window.__zaphire&&window.__zaphire.receive({json})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ui_commands() {
        assert_eq!(
            Command::parse(br#"{"type":"ready"}"#).unwrap(),
            Command::Ready
        );
        assert_eq!(
            Command::parse(br#"{"type":"navigate","id":null,"input":"rust"}"#).unwrap(),
            Command::Navigate {
                id: None,
                input: "rust".into()
            }
        );
        assert_eq!(
            Command::parse(br#"{"type":"moveTab","id":3,"workspace":1,"index":0}"#).unwrap(),
            Command::MoveTab {
                id: 3,
                workspace: 1,
                index: 0
            }
        );
        assert_eq!(
            Command::parse(br#"{"type":"setProfile","profile":"game"}"#).unwrap(),
            Command::SetProfile {
                profile: Profile::Game
            }
        );
        assert!(Command::parse(br#"{"type":"rm -rf"}"#).is_err());
    }

    #[test]
    fn state_event_js() {
        let session = Session::default();
        let settings = Settings::default();
        let js = Event::State {
            session: &session,
            settings: &settings,
            blocked: 7,
            restart_needed: false,
            platform: "linux",
        }
        .to_js();
        assert!(js.starts_with("window.__zaphire&&window.__zaphire.receive({\"type\":\"state\""));
        assert!(js.contains("\"restartNeeded\":false"));
        assert!(js.contains("\"tab_layout\":\"vertical\""));
    }
}
