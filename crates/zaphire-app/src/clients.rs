//! CEF clients: one kind for web tabs, one for the UI view.

use crate::{app, shell};
use cef::*;
use std::sync::atomic::Ordering;
use zaphire_core::TabId;

fn s(v: Option<&CefString>) -> String {
    v.map(|v| v.to_string()).unwrap_or_default()
}

// ───────────────────────────── tabs ─────────────────────────────

wrap_client! {
    pub struct TabClient {
        tab: TabId,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(TabDisplay::new(self.tab))
        }
        fn load_handler(&self) -> Option<LoadHandler> {
            Some(TabLoad::new(self.tab))
        }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(TabLifeSpan::new(self.tab))
        }
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(BlockingRequestHandler::new())
        }
        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(Shortcuts::new())
        }
    }
}

wrap_display_handler! {
    struct TabDisplay {
        tab: TabId,
    }

    impl DisplayHandler {
        fn on_address_change(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, url: Option<&CefString>) {
            if frame.is_some_and(|f| f.is_main() == 1) {
                let url = s(url);
                shell::update_tab(self.tab, move |t| t.url = url);
            }
        }

        fn on_title_change(&self, _browser: Option<&mut Browser>, title: Option<&CefString>) {
            let title = s(title);
            shell::update_tab(self.tab, move |t| t.title = title);
        }

        fn on_favicon_urlchange(&self, _browser: Option<&mut Browser>, icon_urls: Option<&mut CefStringList>) {
            let first = icon_urls.and_then(|l| std::mem::take(l).into_iter().next()).map(|u| u.to_string());
            shell::update_tab(self.tab, move |t| t.favicon = first);
        }

        fn on_fullscreen_mode_change(&self, _browser: Option<&mut Browser>, fullscreen: i32) {
            shell::set_fullscreen(self.tab, fullscreen == 1);
        }
    }
}

wrap_load_handler! {
    struct TabLoad {
        tab: TabId,
    }

    impl LoadHandler {
        fn on_loading_state_change(&self, _browser: Option<&mut Browser>, is_loading: i32, can_go_back: i32, can_go_forward: i32) {
            shell::update_tab(self.tab, move |t| {
                t.loading = is_loading == 1;
                t.can_go_back = can_go_back == 1;
                t.can_go_forward = can_go_forward == 1;
            });
        }
    }
}

wrap_life_span_handler! {
    struct TabLifeSpan {
        tab: TabId,
    }

    impl LifeSpanHandler {
        #[allow(clippy::too_many_arguments)]
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: i32,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            target_disposition: WindowOpenDisposition,
            _user_gesture: i32,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut i32>,
        ) -> i32 {
            // Real popups (window.open with features, e.g. OAuth logins) keep
            // their own window so `window.opener` works. Everything else that
            // wants a new tab or window becomes a Zaphire tab.
            if target_disposition == WindowOpenDisposition::NEW_POPUP {
                return 0;
            }
            let url = s(target_url);
            let background = target_disposition == WindowOpenDisposition::NEW_BACKGROUND_TAB;
            crate::tasks::on_ui(0, move || shell::open_tab(&url, !background));
            1
        }

        fn on_after_created(&self, _browser: Option<&mut Browser>) {
            shell::browser_created();
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            shell::browser_closed();
        }
    }
}

wrap_request_handler! {
    struct BlockingRequestHandler;

    impl RequestHandler {
        #[allow(clippy::too_many_arguments)]
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            is_navigation: i32,
            _is_download: i32,
            _request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut i32>,
        ) -> Option<ResourceRequestHandler> {
            if is_navigation == 1 || !app::global().adblock_on.load(Ordering::Relaxed) {
                return None;
            }
            Some(AdblockResourceHandler::new())
        }
    }
}

fn request_kind(t: ResourceType) -> &'static str {
    match t {
        ResourceType::MAIN_FRAME => "document",
        ResourceType::SUB_FRAME => "subdocument",
        ResourceType::STYLESHEET => "stylesheet",
        ResourceType::SCRIPT => "script",
        ResourceType::IMAGE | ResourceType::FAVICON => "image",
        ResourceType::FONT_RESOURCE => "font",
        ResourceType::MEDIA => "media",
        ResourceType::XHR | ResourceType::PREFETCH => "xmlhttprequest",
        ResourceType::PING | ResourceType::CSP_REPORT => "ping",
        _ => "other",
    }
}

wrap_resource_request_handler! {
    struct AdblockResourceHandler;

    impl ResourceRequestHandler {
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE };
            let url = CefString::from(&request.url()).to_string();
            let source = frame.map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default();
            if app::global().blocker.should_block(&url, &source, request_kind(request.resource_type())) {
                log::debug!("blocked {url}");
                return ReturnValue::CANCEL;
            }
            ReturnValue::CONTINUE
        }
    }
}

// ─────────────────────────── shortcuts ───────────────────────────

#[cfg(target_os = "windows")]
type OsEvent = cef::sys::MSG;
#[cfg(not(target_os = "windows"))]
type OsEvent = cef::sys::XEvent;

const FLAG_SHIFT: u32 = 1 << 1;
const FLAG_CONTROL: u32 = 1 << 2;

wrap_keyboard_handler! {
    struct Shortcuts;

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut OsEvent>,
            _is_keyboard_shortcut: Option<&mut i32>,
        ) -> i32 {
            let Some(e) = event else { return 0 };
            if e.type_ != KeyEventType::RAWKEYDOWN {
                return 0;
            }
            let ctrl = e.modifiers & FLAG_CONTROL != 0;
            let shift = e.modifiers & FLAG_SHIFT != 0;
            let action = match (ctrl, shift, e.windows_key_code) {
                (true, false, 0x54) => shell::Shortcut::NewTab,       // Ctrl+T
                (true, false, 0x57) => shell::Shortcut::CloseTab,     // Ctrl+W
                (true, false, 0x4C) => shell::Shortcut::FocusAddress, // Ctrl+L
                (true, false, 0x52) | (false, false, 0x74) => shell::Shortcut::Reload, // Ctrl+R / F5
                (true, false, 0x09) => shell::Shortcut::NextTab,      // Ctrl+Tab
                (true, true, 0x09) => shell::Shortcut::PrevTab,       // Ctrl+Shift+Tab
                (true, true, 0x47) => shell::Shortcut::ToggleGame,    // Ctrl+Shift+G
                (true, true, 0x53) => shell::Shortcut::ToggleSplit,   // Ctrl+Shift+S
                (false, false, 0x7B) => shell::Shortcut::DevTools,    // F12
                _ => return 0,
            };
            crate::tasks::on_ui(0, move || shell::shortcut(action));
            1
        }
    }
}

// ───────────────────────────── UI view ─────────────────────────────

wrap_client! {
    pub struct UiClient;

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(UiLifeSpan::new())
        }
        fn drag_handler(&self) -> Option<DragHandler> {
            Some(UiDrag::new())
        }
        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(Shortcuts::new())
        }
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(UiRequest::new())
        }
    }
}

wrap_life_span_handler! {
    struct UiLifeSpan;

    impl LifeSpanHandler {
        #[allow(clippy::too_many_arguments)]
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: i32,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: i32,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut i32>,
        ) -> i32 {
            let url = s(target_url);
            crate::tasks::on_ui(0, move || shell::open_tab(&url, true));
            1
        }

        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(b) = browser {
                app::global().ui_browser_id.store(b.identifier(), Ordering::Relaxed);
            }
            shell::browser_created();
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            shell::browser_closed();
        }
    }
}

wrap_request_handler! {
    struct UiRequest;

    impl RequestHandler {
        fn on_before_browse(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _user_gesture: i32,
            _is_redirect: i32,
        ) -> i32 {
            // The UI view must never leave zaphire://app. Links clicked in it
            // open as tabs instead.
            let url = request.map(|r| CefString::from(&r.url()).to_string()).unwrap_or_default();
            if frame.is_some_and(|f| f.is_main() == 1) && !url.starts_with("zaphire://app/") {
                crate::tasks::on_ui(0, move || shell::open_tab(&url, true));
                return 1;
            }
            0
        }
    }
}

wrap_drag_handler! {
    struct UiDrag;

    impl DragHandler {
        fn on_draggable_regions_changed(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            regions: Option<&[DraggableRegion]>,
        ) {
            shell::set_drag_regions(regions.map(|r| r.to_vec()).unwrap_or_default());
        }
    }
}
