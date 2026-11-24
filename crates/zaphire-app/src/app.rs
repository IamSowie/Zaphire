//! CEF application: command-line switches, custom scheme, startup.

use crate::{scheme, shell};
use cef::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Mutex, OnceLock};
use zaphire_adblock::Blocker;
use zaphire_core::{ExtensionInfo, Library, Settings};

/// State shared with CEF's IO thread (network requests, scheme handler).
pub struct Global {
    pub blocker: Blocker,
    pub adblock_on: AtomicBool,
    /// Browser id of the UI view; only it may call `zaphire://app/api`.
    pub ui_browser_id: AtomicI32,
    /// History and bookmarks. Written on the UI thread, searched from the IO
    /// thread by the history page.
    pub library: Mutex<Library>,
    /// Installed extensions; icons are served from their folders.
    pub extensions: Mutex<Vec<ExtensionInfo>>,
    /// Chromium profile folder (`<config>/profile/Default`).
    pub profile_dir: PathBuf,
}

pub static GLOBAL: OnceLock<Global> = OnceLock::new();

pub fn global() -> &'static Global {
    GLOBAL.get().expect("initialized in on_context_initialized")
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}

impl AppState {
    pub fn load() -> Self {
        let settings = Settings::load(&Settings::default_path()).unwrap_or_else(|e| {
            eprintln!("zaphire: settings unreadable ({e}), using defaults");
            Settings::default()
        });
        Self { settings }
    }
}

// Scheme option bits from cef_scheme_options_t.
const SCHEME_STANDARD: i32 = 1;
const SCHEME_SECURE: i32 = 8;
const SCHEME_CORS_ENABLED: i32 = 16;
const SCHEME_FETCH_ENABLED: i32 = 64;

wrap_app! {
    pub struct ZaphireApp {
        state: AppState,
    }

    impl App {
        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            // Only the browser process decides switches; children inherit them.
            let is_browser = process_type.is_none_or(|p| p.to_string().is_empty());
            let (true, Some(cl)) = (is_browser, command_line) else { return };
            let mut features: Vec<&str> = vec![];
            for (name, value) in self.state.settings.profile.switches() {
                if name == "enable-features" {
                    features.extend(value.unwrap_or("").split(','));
                    continue;
                }
                match value {
                    Some(v) => cl.append_switch_with_value(Some(&name.into()), Some(&v.into())),
                    None => cl.append_switch(Some(&name.into())),
                }
            }
            if !features.is_empty() {
                cl.append_switch_with_value(Some(&"enable-features".into()), Some(&features.join(",").as_str().into()));
            }
            // Record which profile this process was started with.
            cl.append_switch_with_value(
                Some(&"zaphire-profile".into()),
                Some(&format!("{:?}", self.state.settings.profile).to_lowercase().as_str().into()),
            );
        }

        fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
            if let Some(r) = registrar {
                r.add_custom_scheme(
                    Some(&"zaphire".into()),
                    SCHEME_STANDARD | SCHEME_SECURE | SCHEME_CORS_ENABLED | SCHEME_FETCH_ENABLED,
                );
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(ZaphireBrowserProcess::new(self.state.clone()))
        }
    }
}

wrap_browser_process_handler! {
    struct ZaphireBrowserProcess {
        state: AppState,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let settings = self.state.settings.clone();
            let _ = GLOBAL.set(Global {
                blocker: Blocker::load(&Settings::config_dir().join("filters")),
                adblock_on: AtomicBool::new(settings.adblock),
                ui_browser_id: AtomicI32::new(-1),
                library: Mutex::new(Library::load(&Settings::config_dir())),
                extensions: Mutex::new(Vec::new()),
                profile_dir: Settings::config_dir().join("profile").join("Default"),
            });
            let mut factory = scheme::ZaphireSchemeFactory::new();
            register_scheme_handler_factory(Some(&"zaphire".into()), Some(&"app".into()), Some(&mut factory));

            let url = command_line_get_global()
                .map(|cl| CefString::from(&cl.switch_value(Some(&"url".into()))).to_string())
                .filter(|u| !u.is_empty());
            shell::create(settings, url);
        }
    }
}
