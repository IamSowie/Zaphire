//! Performance profiles.
//!
//! A profile has two halves:
//! * **Startup switches** – Chromium command-line flags. They only take effect
//!   when the browser process starts, so changing them needs a restart.
//! * **Live policy** – rules Zaphire applies while running (tab suspension,
//!   freezing background tabs, frame-rate caps). These change instantly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Lowest memory and CPU use: aggressive tab suspension, few processes.
    Saver,
    /// Default: sensible suspension, normal Chromium behaviour.
    #[default]
    Balanced,
    /// Maximum performance for web games: no throttling, GPU forced on,
    /// background tabs frozen so the active tab gets the whole machine.
    Game,
}

/// Rules applied while the browser runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicy {
    /// Unload a background tab after this many seconds without being shown.
    /// `None` keeps tabs loaded forever.
    pub suspend_after_secs: Option<u64>,
    /// Hide (freeze rendering of) every tab except the visible ones.
    pub freeze_background: bool,
    /// Mute audio of background tabs.
    pub mute_background: bool,
    /// Ask the OS for a higher scheduling priority for the browser process.
    pub high_priority: bool,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::Saver, Profile::Balanced, Profile::Game];

    /// Chromium switches as `(name, optional value)`.
    pub fn switches(self) -> Vec<(&'static str, Option<&'static str>)> {
        // Always on: GPU rasterization and zero-copy uploads are cheaper on
        // both CPU and memory than software paths on any modern GPU.
        let mut s = vec![
            ("enable-gpu-rasterization", None),
            ("enable-zero-copy", None),
        ];
        match self {
            Profile::Saver => {
                s.extend([
                    ("renderer-process-limit", Some("4")),
                    ("disable-background-networking", None),
                    ("enable-low-end-device-mode", None),
                    (
                        "enable-features",
                        Some("HighEfficiencyModeAvailable,IntensiveWakeUpThrottling"),
                    ),
                ]);
            }
            Profile::Balanced => {}
            Profile::Game => {
                s.extend([
                    ("ignore-gpu-blocklist", None),
                    ("disable-gpu-vsync", None),
                    ("disable-frame-rate-limit", None),
                    ("disable-background-timer-throttling", None),
                    ("disable-renderer-backgrounding", None),
                    ("disable-backgrounding-occluded-windows", None),
                    ("enable-unsafe-webgpu", None),
                    ("enable-features", Some("Vulkan,WebGPUService")),
                ]);
            }
        }
        s
    }

    pub fn live_policy(self) -> LivePolicy {
        match self {
            Profile::Saver => LivePolicy {
                suspend_after_secs: Some(5 * 60),
                freeze_background: true,
                mute_background: false,
                high_priority: false,
            },
            Profile::Balanced => LivePolicy {
                suspend_after_secs: Some(30 * 60),
                freeze_background: true,
                mute_background: false,
                high_priority: false,
            },
            Profile::Game => LivePolicy {
                suspend_after_secs: None,
                freeze_background: true,
                mute_background: true,
                high_priority: true,
            },
        }
    }

    /// True when switching from `self` to `other` needs a restart to fully apply.
    pub fn needs_restart_to(self, other: Profile) -> bool {
        self.switches() != other.switches()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_disables_throttling() {
        let s = Profile::Game.switches();
        assert!(
            s.iter()
                .any(|(n, _)| *n == "disable-background-timer-throttling")
        );
        assert!(s.iter().any(|(n, _)| *n == "disable-frame-rate-limit"));
        assert!(Profile::Game.live_policy().suspend_after_secs.is_none());
    }

    #[test]
    fn saver_limits_processes() {
        let s = Profile::Saver.switches();
        assert!(s.contains(&("renderer-process-limit", Some("4"))));
        assert!(Profile::Saver.live_policy().suspend_after_secs.unwrap() <= 300);
    }

    #[test]
    fn restart_detection() {
        assert!(Profile::Balanced.needs_restart_to(Profile::Game));
        assert!(!Profile::Game.needs_restart_to(Profile::Game));
    }
}
