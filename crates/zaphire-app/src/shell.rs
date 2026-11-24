//! The browser window: UI view + content area, tab views, commands.
//!
//! Everything here runs on the CEF UI thread. State lives in a thread-local
//! so CEF callbacks can reach it; callbacks that may fire while the state is
//! borrowed (layout, creation) go through atomics or `tasks::on_ui` instead.

use crate::clients::{PopupClient, TabClient, UiClient};
use crate::{app, tasks};
use cef::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use zaphire_core::{Session, Settings, Tab, TabId, TabLayout, url::resolve_input};
use zaphire_ipc::{Command, Event};
use zaphire_perf::Profile;

const TOPBAR_HEIGHT: i32 = 88;
const SUSPEND_CHECK_MS: i64 = 30_000;
const SAVE_EVERY_MS: i64 = 20_000;
const BG_COLOR: u32 = 0xff0b1020;

/// Size the UI view asks for (width in vertical-tab mode, height otherwise).
static UI_WIDTH: AtomicI32 = AtomicI32::new(280);
static UI_HEIGHT: AtomicI32 = AtomicI32::new(TOPBAR_HEIGHT);
/// Browsers alive, used to quit only after every browser closed.
static LIVE_BROWSERS: AtomicUsize = AtomicUsize::new(0);
static CLOSING: AtomicBool = AtomicBool::new(false);
static PUSH_PENDING: AtomicBool = AtomicBool::new(false);
/// OS prefers dark colours (reported by the UI, for the `system` theme).
static SYSTEM_DARK: AtomicBool = AtomicBool::new(true);
static LAST_POPUP_CLOSE: std::sync::Mutex<Option<(String, std::time::Instant)>> =
    std::sync::Mutex::new(None);
const POPUP_DEFAULT: Size = Size {
    width: 360,
    height: 480,
};
/// Latest state event as JSON, served to pages at `zaphire://app/api/state`.
pub static LAST_STATE: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

thread_local! {
    static SHELL: RefCell<Option<Shell>> = const { RefCell::new(None) };
}

struct Shell {
    window: Option<Window>,
    ui: BrowserView,
    content: Panel,
    views: HashMap<TabId, BrowserView>,
    session: Session,
    settings: Settings,
    /// Profile the process was started with (startup switches).
    running: Profile,
    fullscreen: Option<TabId>,
    /// Chrome-style windows (extensions, Web Store). See `open_chrome_window`.
    chrome_windows: Vec<Window>,
    /// Open extension popup, floating over the window.
    popup: Option<Popup>,
}

struct Popup {
    id: String,
    controller: OverlayController,
    view: BrowserView,
    anchor: Point,
}

fn with_shell<R>(f: impl FnOnce(&mut Shell) -> R) -> Option<R> {
    SHELL.with(|s| match s.try_borrow_mut() {
        Ok(mut s) => s.as_mut().map(f),
        Err(_) => {
            log::error!("shell re-entered; dropping call");
            None
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Shortcut {
    NewTab,
    CloseTab,
    FocusAddress,
    Reload,
    NextTab,
    PrevTab,
    ToggleGame,
    ToggleSplit,
    DevTools,
    Bookmark,
    History,
}

// ─────────────────────────── creation ───────────────────────────

pub fn create(settings: Settings, url: Option<String>) {
    let running = command_line_get_global()
        .map(|cl| {
            CefString::from(&cl.switch_value(Some(&CefString::from("zaphire-profile")))).to_string()
        })
        .and_then(|p| serde_json::from_value(serde_json::Value::String(p)).ok())
        .unwrap_or(settings.profile);
    if running == Profile::Game {
        raise_priority();
    }
    UI_WIDTH.store(settings.sidebar_width as i32, Ordering::Relaxed);

    let mut ui_client = UiClient::new();
    let ui_settings = BrowserSettings {
        background_color: settings.theme.frame_color(true),
        ..Default::default()
    };
    let ui = browser_view_create(
        Some(&mut ui_client),
        Some(&CefString::from("zaphire://app/")),
        Some(&ui_settings),
        None,
        None,
        Some(&mut UiViewDelegate::new()),
    )
    .expect("UI view");

    let content = panel_create(None).expect("content panel");
    content.set_background_color(BG_COLOR);
    content.set_to_box_layout(Some(&BoxLayoutSettings {
        horizontal: 1,
        between_child_spacing: 6,
        default_flex: 1,
        cross_axis_alignment: AxisAlignment::STRETCH,
        ..Default::default()
    }));

    // Restored tabs come back unloaded; only the visible ones load.
    let restored = settings
        .restore_session
        .then(|| app::global().library.lock().ok()?.load_session())
        .flatten();
    let first_url = match (&restored, url) {
        (_, Some(u)) => Some(u),
        (None, None) => Some(settings.homepage.clone()),
        (Some(_), None) => None,
    };
    let frame = settings
        .theme
        .frame_color(SYSTEM_DARK.load(Ordering::Relaxed));
    content.set_background_color(frame);
    SHELL.with(|s| {
        *s.borrow_mut() = Some(Shell {
            window: None,
            ui,
            content,
            views: HashMap::new(),
            session: restored.unwrap_or_default(),
            settings,
            running,
            fullscreen: None,
            chrome_windows: Vec::new(),
            popup: None,
        });
    });
    with_shell(|sh| {
        if let Some(url) = first_url {
            let id = sh.session.new_tab(&url, tasks::now_secs());
            sh.create_view(id);
        }
        sh.apply_visibility();
    });
    tasks::on_ui(2_000, refresh_extensions);
    tasks::on_ui(6_000, refresh_extensions);

    window_create_top_level(Some(&mut ShellWindowDelegate::new()));
    tasks::on_ui(SUSPEND_CHECK_MS, suspend_tick);
    tasks::on_ui(SAVE_EVERY_MS, save_tick);
}

fn raise_priority() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::System::Threading::{
            ABOVE_NORMAL_PRIORITY_CLASS, GetCurrentProcess, SetPriorityClass,
        };
        SetPriorityClass(GetCurrentProcess(), ABOVE_NORMAL_PRIORITY_CLASS);
    }
    // On Linux raising priority needs CAP_SYS_NICE; Game mode relies on the
    // Chromium switches and freezing background tabs instead.
}

impl Shell {
    fn create_view(&mut self, id: TabId) {
        let url = self
            .session
            .tab(id)
            .map(|t| t.url.clone())
            .unwrap_or_default();
        let mut client = TabClient::new(id);
        let settings = BrowserSettings {
            background_color: 0xffffffff,
            ..Default::default()
        };
        let Some(view) = browser_view_create(
            Some(&mut client),
            Some(&CefString::from(url.as_str())),
            Some(&settings),
            None,
            None,
            Some(&mut TabViewDelegate::new()),
        ) else {
            log::error!("cannot create browser for tab {id}");
            return;
        };
        view.set_visible(0);
        self.content.add_child_view(Some(&mut View::from(&view)));
        self.views.insert(id, view);
    }

    fn browser(&self, id: TabId) -> Option<Browser> {
        self.views.get(&id)?.browser()
    }

    fn active(&self) -> Option<TabId> {
        self.session.current_workspace().active
    }

    /// Show the active (and split) tab, hide the rest, apply live policy.
    fn apply_visibility(&mut self) {
        let visible = self.session.visible_tabs();
        let policy = self.running.live_policy();
        let now = tasks::now_secs();
        for id in &visible {
            if self.session.tab(*id).is_some_and(|t| t.suspended) {
                self.session.tab_mut(*id).expect("exists").suspended = false;
                self.create_view(*id);
            }
            if let Some(t) = self.session.tab_mut(*id) {
                t.last_visible = now;
            }
        }
        for (id, view) in &self.views {
            let show = visible.contains(id);
            view.set_visible(show as i32);
            if let Some(host) = view.browser().and_then(|b| b.host()) {
                host.set_audio_muted((policy.mute_background && !show) as i32);
            }
        }
        for (i, id) in visible.iter().enumerate() {
            if let Some(v) = self.views.get(id) {
                self.content
                    .reorder_child_view(Some(&mut View::from(v)), i as i32);
            }
        }
        self.content.layout();
        if let (Some(w), Some(t)) = (
            &self.window,
            self.active().and_then(|a| self.session.tab(a)),
        ) {
            let title = if t.title.is_empty() {
                "Zaphire".to_string()
            } else {
                format!("{} — Zaphire", t.title)
            };
            w.set_title(Some(&CefString::from(title.as_str())));
        }
        schedule_push();
    }

    fn apply_root_layout(&self) {
        let Some(window) = &self.window else { return };
        let horizontal_root = self.settings.tab_layout == TabLayout::Vertical;
        let layout = window.set_to_box_layout(Some(&BoxLayoutSettings {
            horizontal: horizontal_root as i32,
            cross_axis_alignment: AxisAlignment::STRETCH,
            ..Default::default()
        }));
        if let Some(layout) = layout {
            layout.set_flex_for_view(Some(&mut View::from(&self.content)), 1);
            layout.set_flex_for_view(Some(&mut View::from(&self.ui)), 0);
        }
        self.ui.invalidate_layout();
        window.layout();
    }

    fn suspend(&mut self, id: TabId) {
        if let Some(view) = self.views.remove(&id) {
            self.content.remove_child_view(Some(&mut View::from(&view)));
            if let Some(host) = view.browser().and_then(|b| b.host()) {
                host.close_browser(1);
            }
        }
        if let Some(t) = self.session.tab_mut(id) {
            t.suspended = true;
            t.loading = false;
        }
    }

    fn close_tab(&mut self, id: TabId) {
        self.suspend(id);
        self.session.close_tab(id);
        if self.session.current_workspace().tabs.is_empty() {
            let id = self
                .session
                .new_tab(&self.settings.homepage.clone(), tasks::now_secs());
            self.create_view(id);
        }
        self.apply_visibility();
    }

    fn new_tab(&mut self, url: &str, focus: bool) -> TabId {
        if needs_chrome_window(url) {
            // Deferred: open_chrome_window borrows the shell itself.
            let url = url.to_string();
            tasks::on_ui(0, move || open_chrome_window(&url));
            return self.active().unwrap_or_default();
        }
        let previous = self.active();
        let id = self.session.new_tab(url, tasks::now_secs());
        self.create_view(id);
        if !focus && let Some(p) = previous {
            self.session.activate(p, tasks::now_secs());
        }
        self.apply_visibility();
        id
    }

    fn frame_color(&self) -> u32 {
        self.settings
            .theme
            .frame_color(SYSTEM_DARK.load(Ordering::Relaxed))
    }

    /// Paint the window and the gaps between panes in the theme's colour.
    fn apply_frame_color(&self) {
        let c = self.frame_color();
        self.content.set_background_color(c);
        if let Some(w) = &self.window {
            w.set_background_color(c);
        }
    }

    /// Persist history, bookmarks and (if enabled) the open tabs.
    fn save_all(&self) {
        let Ok(mut lib) = app::global().library.lock() else {
            return;
        };
        if let Err(e) = lib.save() {
            log::error!("cannot save history/bookmarks: {e}");
        }
        if self.settings.restore_session
            && let Err(e) = lib.save_session(&self.session)
        {
            log::error!("cannot save session: {e}");
        }
    }

    fn close_popup(&mut self) {
        if let Some(p) = self.popup.take() {
            // Clicking the same button again first moves focus (closing the
            // popup); remember it so that click does not reopen it.
            if let Ok(mut last) = LAST_POPUP_CLOSE.lock() {
                *last = Some((p.id.clone(), std::time::Instant::now()));
            }
            if let Some(host) = p.view.browser().and_then(|b| b.host()) {
                host.close_browser(1);
            }
            p.controller.destroy();
        }
    }

    fn open_popup(&mut self, id: &str, x: i32, y: i32) {
        self.close_popup();
        let Some(window) = self.window.clone() else {
            return;
        };
        let exts = app::global()
            .extensions
            .lock()
            .map(|e| e.clone())
            .unwrap_or_default();
        let Some(ext) = exts.iter().find(|e| e.id == id) else {
            return;
        };
        let Some(popup) = &ext.popup else {
            // No popup: show the extension's options in Chrome's manager.
            open_chrome_window(&format!("chrome://extensions/?id={id}"));
            return;
        };
        let url = format!("chrome-extension://{id}/{popup}");
        let Some(view) = browser_view_create(
            Some(&mut PopupClient::new()),
            Some(&CefString::from(url.as_str())),
            Some(&BrowserSettings {
                background_color: self.frame_color(),
                ..Default::default()
            }),
            None,
            None,
            Some(&mut PopupViewDelegate::new()),
        ) else {
            return log::error!("cannot create popup for {id}");
        };
        let Some(controller) =
            window.add_overlay_view(Some(&mut View::from(&view)), DockingMode::CUSTOM, 1)
        else {
            return;
        };
        let anchor = Point { x, y };
        self.popup = Some(Popup {
            id: id.to_string(),
            controller,
            view,
            anchor,
        });
        self.place_popup(POPUP_DEFAULT);
        if let Some(p) = &self.popup {
            p.controller.set_visible(1);
            p.view.request_focus();
        }
    }

    /// Put the popup under its button, kept inside the window.
    fn place_popup(&self, size: Size) {
        let (Some(p), Some(w)) = (&self.popup, &self.window) else {
            return;
        };
        let win = w.bounds();
        let width = size.width.clamp(80, 800);
        let height = size.height.clamp(40, 600);
        let x = p.anchor.x.clamp(4, (win.width - width - 4).max(4));
        let y = p.anchor.y.clamp(4, (win.height - height - 4).max(4));
        p.controller.set_bounds(Some(&Rect {
            x,
            y,
            width,
            height,
        }));
    }

    fn save_settings(&self) {
        if let Err(e) = self.settings.save(&Settings::default_path()) {
            log::error!("cannot save settings: {e}");
        }
    }

    fn push(&self) {
        let Some(frame) = self.ui.browser().and_then(|b| b.main_frame()) else {
            return;
        };

        let g = app::global();
        let (Ok(lib), Ok(exts)) = (g.library.lock(), g.extensions.lock()) else {
            return;
        };
        let bookmarked = self
            .active()
            .and_then(|a| self.session.tab(a))
            .is_some_and(|t| lib.is_bookmarked(&t.url));
        let event = Event::State {
            session: &self.session,
            settings: &self.settings,
            blocked: g.blocker.blocked_count(),
            restart_needed: self.running != self.settings.profile
                && self.running.needs_restart_to(self.settings.profile),
            platform: std::env::consts::OS,
            bookmarks: &lib.bookmarks,
            extensions: &exts,
            bookmarked,
        };
        if let Ok(mut last) = LAST_STATE.lock() {
            *last = serde_json::to_string(&event).unwrap_or_default();
        }
        let js = event.to_js();
        frame.execute_java_script(
            Some(&CefString::from(js.as_str())),
            Some(&CefString::from("zaphire://app/ipc")),
            0,
        );
    }

    fn emit_js(&self, js: &str) {
        if let Some(frame) = self.ui.browser().and_then(|b| b.main_frame()) {
            frame.execute_java_script(
                Some(&CefString::from(js)),
                Some(&CefString::from("zaphire://app/ipc")),
                0,
            );
        }
    }

    fn cycle(&mut self, step: isize) {
        let ws = self.session.current_workspace();
        let Some(active) = ws.active else { return };
        let pos = ws.tabs.iter().position(|t| *t == active).unwrap_or(0) as isize;
        let n = ws.tabs.len() as isize;
        let next = ws.tabs[((pos + step).rem_euclid(n)) as usize];
        self.session.activate(next, tasks::now_secs());
        self.apply_visibility();
    }
}

// ─────────────────────────── public entry points ───────────────────────────

pub fn open_tab(url: &str, focus: bool) {
    with_shell(|sh| {
        sh.new_tab(url, focus);
    });
}

/// Update a tab from a CEF callback. Deferred so it never re-enters the shell.
pub fn update_tab(id: TabId, f: impl FnOnce(&mut Tab) + Send + 'static) {
    tasks::on_ui(0, move || {
        with_shell(|sh| {
            if let Some(t) = sh.session.tab_mut(id) {
                let (was_loading, old_title) = (t.loading, t.title.clone());
                f(t);
                if let Ok(mut lib) = app::global().library.lock() {
                    if was_loading && !t.loading {
                        lib.record(&t.url, &t.title, unix_now());
                    } else if t.title != old_title && !t.loading {
                        lib.update_title(&t.url, &t.title);
                    }
                }
            }
            if sh.active() == Some(id)
                && let (Some(w), Some(t)) = (&sh.window, sh.session.tab(id))
                && !t.title.is_empty()
            {
                w.set_title(Some(&CefString::from(
                    format!("{} — Zaphire", t.title).as_str(),
                )));
            }
        });
        schedule_push();
    });
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Re-read installed extensions from the profile and tell the UI.
pub fn refresh_extensions() {
    let g = app::global();
    let list = zaphire_core::extensions::installed(&g.profile_dir);
    if let Ok(mut e) = g.extensions.lock() {
        if *e == list {
            return;
        }
        log::info!("{} extension(s) installed", list.len());
        *e = list;
    }
    schedule_push();
}

/// Resize the open popup to its content (from `on_auto_resize`).
pub fn resize_popup(size: Size) {
    tasks::on_ui(0, move || {
        with_shell(|sh| sh.place_popup(size));
    });
}

pub fn close_popup() {
    tasks::on_ui(0, || {
        with_shell(|sh| sh.close_popup());
    });
}

/// Coalesce state pushes to one per frame.
pub fn schedule_push() {
    if !PUSH_PENDING.swap(true, Ordering::AcqRel) {
        tasks::on_ui(16, || {
            PUSH_PENDING.store(false, Ordering::Release);
            with_shell(|sh| sh.push());
        });
    }
}

pub fn browser_created() {
    LIVE_BROWSERS.fetch_add(1, Ordering::AcqRel);
}

pub fn browser_closed() {
    let left = LIVE_BROWSERS.fetch_sub(1, Ordering::AcqRel) - 1;
    if left == 0 && CLOSING.load(Ordering::Acquire) {
        tasks::on_ui(0, || {
            SHELL.with(|s| s.borrow_mut().take());
            quit_message_loop();
        });
    }
}

pub fn set_drag_regions(regions: Vec<DraggableRegion>) {
    tasks::on_ui(0, move || {
        with_shell(|sh| {
            if let Some(w) = &sh.window {
                w.set_draggable_regions(Some(&regions));
            }
        });
    });
}

/// HTML5 fullscreen from a page (videos, games): hide all chrome.
pub fn set_fullscreen(id: TabId, on: bool) {
    tasks::on_ui(0, move || {
        with_shell(|sh| {
            sh.fullscreen = on.then_some(id);
            sh.ui.set_visible((!on) as i32);
            for (tab, view) in &sh.views {
                if on && *tab != id {
                    view.set_visible(0);
                }
            }
            if let Some(w) = &sh.window {
                w.set_fullscreen(on as i32);
            }
            if !on {
                sh.apply_visibility();
            }
            if let Some(w) = &sh.window {
                w.layout();
            }
        });
    });
}

pub fn shortcut(s: Shortcut) {
    with_shell(|sh| match s {
        Shortcut::NewTab => {
            let home = sh.settings.homepage.clone();
            sh.new_tab(&home, true);
            sh.emit_js("window.__zaphire&&window.__zaphire.receive({type:'focusAddress'})");
        }
        Shortcut::CloseTab => {
            if let Some(a) = sh.active() {
                sh.close_tab(a);
            }
        }
        Shortcut::FocusAddress => {
            sh.ui.request_focus();
            sh.emit_js("window.__zaphire&&window.__zaphire.receive({type:'focusAddress'})");
        }
        Shortcut::Reload => {
            if let Some(b) = sh.active().and_then(|a| sh.browser(a)) {
                b.reload();
            }
        }
        Shortcut::NextTab => sh.cycle(1),
        Shortcut::PrevTab => sh.cycle(-1),
        Shortcut::ToggleGame => {
            sh.settings.profile = if sh.settings.profile == Profile::Game {
                Profile::Balanced
            } else {
                Profile::Game
            };
            sh.save_settings();
            schedule_push();
        }
        Shortcut::ToggleSplit => {
            let ws = sh.session.current_workspace();
            let target = if ws.split.is_some() {
                None
            } else {
                ws.tabs.iter().copied().find(|t| Some(*t) != ws.active)
            };
            sh.session.set_split(target);
            sh.apply_visibility();
        }
        Shortcut::Bookmark => toggle_bookmark(sh),
        Shortcut::History => {
            sh.new_tab("zaphire://app/library", true);
        }
        Shortcut::DevTools => {
            if let Some(host) = sh
                .active()
                .and_then(|a| sh.browser(a))
                .and_then(|b| b.host())
            {
                host.show_dev_tools(None, None, None, None);
            }
        }
    });
}

fn toggle_bookmark(sh: &mut Shell) {
    let Some(t) = sh.active().and_then(|a| sh.session.tab(a)) else {
        return;
    };
    if t.url.starts_with("zaphire:") || t.url.starts_with("chrome") {
        return;
    }
    if let Ok(mut lib) = app::global().library.lock() {
        lib.toggle_bookmark(&t.url, &t.title, unix_now());
    }
    schedule_push();
}

/// Handle a command from the UI.
pub fn handle(cmd: Command) {
    let now = tasks::now_secs();
    with_shell(|sh| {
        match cmd {
            Command::Ready => tasks::on_ui(0, refresh_extensions),
            Command::NewTab { url } => {
                let url = url
                    .map(|u| resolve_input(&u, &sh.settings.search_engine))
                    .unwrap_or_else(|| sh.settings.homepage.clone());
                sh.new_tab(&url, true);
            }
            Command::CloseTab { id } => sh.close_tab(id),
            Command::ActivateTab { id } => {
                sh.session.activate(id, now);
                sh.apply_visibility();
            }
            Command::PinTab { id, pinned } => {
                if let Some(t) = sh.session.tab_mut(id) {
                    t.pinned = pinned;
                }
            }
            Command::Navigate { id, input } => {
                let url = resolve_input(&input, &sh.settings.search_engine);
                if needs_chrome_window(&url) {
                    tasks::on_ui(0, move || open_chrome_window(&url));
                } else if let Some(id) = id.or(sh.active()) {
                    if let Some(t) = sh.session.tab_mut(id) {
                        t.url = url.clone();
                    }
                    match sh.browser(id).and_then(|b| b.main_frame()) {
                        Some(frame) => frame.load_url(Some(&CefString::from(url.as_str()))),
                        None => sh.apply_visibility(),
                    }
                    if let Some(b) = sh.views.get(&id) {
                        b.request_focus();
                    }
                }
            }
            Command::Back | Command::Forward | Command::Reload | Command::Stop => {
                if let Some(b) = sh.active().and_then(|a| sh.browser(a)) {
                    match cmd {
                        Command::Back => b.go_back(),
                        Command::Forward => b.go_forward(),
                        Command::Reload => b.reload(),
                        _ => b.stop_load(),
                    }
                }
            }
            Command::SetSplit { id } => {
                sh.session.set_split(id);
                sh.apply_visibility();
            }
            Command::NewWorkspace { name, icon, color } => {
                let ws = sh.session.add_workspace(&name, &icon, &color);
                sh.session.switch_workspace(ws);
                let home = sh.settings.homepage.clone();
                sh.new_tab(&home, true);
            }
            Command::SwitchWorkspace { id } => {
                sh.session.switch_workspace(id);
                if sh.session.current_workspace().tabs.is_empty() {
                    let home = sh.settings.homepage.clone();
                    sh.new_tab(&home, true);
                }
                sh.apply_visibility();
            }
            Command::RemoveWorkspace { id } => {
                let tabs = sh
                    .session
                    .workspace(id)
                    .map(|w| w.tabs.clone())
                    .unwrap_or_default();
                if sh.session.workspaces.len() > 1 {
                    for t in tabs {
                        sh.suspend(t);
                    }
                    sh.session.remove_workspace(id);
                }
                sh.apply_visibility();
            }
            Command::MoveTab {
                id,
                workspace,
                index,
            } => {
                sh.session.move_tab(id, workspace, index);
                sh.apply_visibility();
            }
            Command::UpdateSettings { settings } => {
                let layout_changed = settings.tab_layout != sh.settings.tab_layout
                    || settings.sidebar_width != sh.settings.sidebar_width;
                app::global()
                    .adblock_on
                    .store(settings.adblock, Ordering::Relaxed);
                UI_WIDTH.store(
                    settings.sidebar_width.clamp(200, 480) as i32,
                    Ordering::Relaxed,
                );
                sh.settings = settings;
                sh.save_settings();
                sh.apply_frame_color();
                if layout_changed {
                    sh.apply_root_layout();
                }
            }
            Command::SetProfile { profile } => {
                sh.settings.profile = profile;
                sh.save_settings();
            }
            Command::OpenExtensions => open_chrome_window("chrome://extensions"),
            Command::DevTools => {
                if let Some(host) = sh.ui.browser().and_then(|b| b.host()) {
                    host.show_dev_tools(None, None, None, None);
                }
            }
            Command::Restart => {
                sh.save_all();
                restart(sh)
            }
            Command::ToggleBookmark => toggle_bookmark(sh),
            Command::RemoveBookmark { url } => {
                if let Ok(mut lib) = app::global().library.lock() {
                    lib.remove_bookmark(&url);
                }
            }
            Command::DeleteHistory { url } => {
                if let Ok(mut lib) = app::global().library.lock() {
                    lib.delete_url(&url);
                }
            }
            Command::ClearHistory => {
                if let Ok(mut lib) = app::global().library.lock() {
                    lib.clear_history();
                }
            }
            Command::ExtensionPopup { id, x, y } => {
                let just_closed = LAST_POPUP_CLOSE
                    .lock()
                    .ok()
                    .and_then(|l| l.clone())
                    .is_some_and(|(last, at)| {
                        last == id && at.elapsed() < std::time::Duration::from_millis(400)
                    });
                if sh.popup.as_ref().is_some_and(|p| p.id == id) {
                    sh.close_popup();
                } else if !just_closed {
                    sh.open_popup(&id, x, y);
                }
            }
            Command::SystemDark { dark } => {
                if SYSTEM_DARK.swap(dark, Ordering::Relaxed) != dark {
                    sh.apply_frame_color();
                }
            }
            Command::Minimize => {
                if let Some(w) = &sh.window {
                    w.minimize();
                }
            }
            Command::ToggleMaximize => {
                if let Some(w) = &sh.window {
                    if w.is_maximized() == 1 {
                        w.restore()
                    } else {
                        w.maximize()
                    }
                }
            }
            Command::CloseWindow => {
                if let Some(w) = &sh.window {
                    w.close();
                }
            }
        }
        schedule_push();
    });
}

/// Pages that only work in a Chrome-style browser.
fn needs_chrome_window(url: &str) -> bool {
    url.starts_with("chrome://extensions")
        || url.starts_with("https://chromewebstore.google.com")
        || url.starts_with("https://chrome.google.com/webstore")
}

/// Open `url` in a separate Chrome-style window.
///
/// CEF allows only one Chrome-style BrowserView per window, so Zaphire's main
/// window uses Alloy-style views (many per window, content scripts of
/// extensions still run in them). Chrome-only pages — `chrome://extensions`,
/// installing from the Chrome Web Store, extension action popups — open here,
/// with Chrome's toolbar so extension buttons are reachable.
pub fn open_chrome_window(url: &str) {
    let mut delegate = ChromeViewDelegate::new();
    let Some(view) = browser_view_create(
        None,
        Some(&CefString::from(url)),
        Some(&BrowserSettings {
            background_color: BG_COLOR,
            ..Default::default()
        }),
        None,
        None,
        Some(&mut delegate),
    ) else {
        return log::error!("cannot create chrome window for {url}");
    };
    let window = window_create_top_level(Some(&mut ChromeWindowDelegate::new(RefCell::new(Some(
        view,
    )))));
    if let Some(w) = window {
        with_shell(|sh| {
            sh.chrome_windows.retain(|w| w.is_closed() == 0);
            sh.chrome_windows.push(w);
        });
    }
}

fn restart(sh: &mut Shell) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return log::error!("restart: {e}"),
    };
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with("--zaphire-profile"))
        .collect();
    if let Err(e) = std::process::Command::new(exe).args(args).spawn() {
        return log::error!("restart: {e}");
    }
    if let Some(w) = &sh.window {
        w.close();
    }
}

fn save_tick() {
    with_shell(|sh| sh.save_all());
    // Chromium writes the profile's Preferences lazily, and extensions can be
    // added from the Chrome window at any time: re-read the list.
    refresh_extensions();
    tasks::on_ui(SAVE_EVERY_MS, save_tick);
}

fn suspend_tick() {
    with_shell(|sh| {
        if let Some(after) = sh.running.live_policy().suspend_after_secs {
            for id in sh.session.tabs_to_suspend(tasks::now_secs(), after) {
                log::info!("suspending tab {id}");
                sh.suspend(id);
            }
            schedule_push();
        }
    });
    tasks::on_ui(SUSPEND_CHECK_MS, suspend_tick);
}

// ─────────────────────────── view delegates ───────────────────────────

wrap_browser_view_delegate! {
    struct UiViewDelegate {}

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: UI_WIDTH.load(Ordering::Relaxed), height: UI_HEIGHT.load(Ordering::Relaxed) }
        }
    }

    impl BrowserViewDelegate {
        fn chrome_toolbar_type(&self, _browser_view: Option<&mut BrowserView>) -> ChromeToolbarType {
            ChromeToolbarType::NONE
        }
        fn browser_runtime_style(&self) -> RuntimeStyle {
            // CEF allows one Chrome-style BrowserView per window; the UI uses Alloy.
            RuntimeStyle::ALLOY
        }
    }
}

wrap_browser_view_delegate! {
    struct TabViewDelegate {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn chrome_toolbar_type(&self, _browser_view: Option<&mut BrowserView>) -> ChromeToolbarType {
            ChromeToolbarType::NONE
        }
        fn browser_runtime_style(&self) -> RuntimeStyle {
            tab_runtime_style()
        }
    }
}

wrap_browser_view_delegate! {
    struct ChromeViewDelegate {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn chrome_toolbar_type(&self, _browser_view: Option<&mut BrowserView>) -> ChromeToolbarType {
            ChromeToolbarType::NORMAL
        }
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }
    }
}

wrap_window_delegate! {
    struct ChromeWindowDelegate {
        view: RefCell<Option<BrowserView>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1100, height: 760 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let (Some(window), Some(view)) = (window, self.view.borrow().clone()) else { return };
            window.add_child_view(Some(&mut View::from(&view)));
            window.show();
        }
        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            self.view.borrow_mut().take();
            // Extensions may have been installed or removed there.
            tasks::on_ui(300, refresh_extensions);
        }
        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            self.view
                .borrow()
                .as_ref()
                .and_then(|v| v.browser())
                .and_then(|b| b.host())
                .map_or(1, |h| h.try_close_browser())
        }
        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }
    }
}

wrap_browser_view_delegate! {
    struct PopupViewDelegate {}

    impl ViewDelegate {
        fn on_blur(&self, _view: Option<&mut View>) {
            // Like Chrome: clicking anywhere else closes the popup.
            close_popup();
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

/// Experimental switch while the tab architecture is settled.
fn tab_runtime_style() -> RuntimeStyle {
    match std::env::var("ZAPHIRE_TAB_STYLE").as_deref() {
        Ok("chrome") => RuntimeStyle::CHROME,
        _ => RuntimeStyle::ALLOY,
    }
}

wrap_window_delegate! {
    struct ShellWindowDelegate {}

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1360, height: 860 }
        }
        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 640, height: 420 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let Some(window) = window else { return };
            window.set_title(Some(&CefString::from("Zaphire")));
            with_shell(|sh| {
                window.add_child_view(Some(&mut View::from(&sh.ui)));
                window.add_child_view(Some(&mut View::from(&sh.content)));
                sh.window = Some(window.clone());
                sh.apply_frame_color();
                sh.apply_root_layout();
            });
            window.show();
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            CLOSING.store(true, Ordering::Release);
            SHELL.with(|s| {
                if let Ok(mut s) = s.try_borrow_mut() {
                    s.take();
                }
            });
            if LIVE_BROWSERS.load(Ordering::Acquire) == 0 {
                quit_message_loop();
            }
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            CLOSING.store(true, Ordering::Release);
            with_shell(|sh| {
                sh.save_all();
                sh.close_popup();
                for w in sh.chrome_windows.drain(..) {
                    if w.is_closed() == 0 {
                        w.close();
                    }
                }
                let mut all = 1;
                for b in std::iter::once(&sh.ui).chain(sh.views.values()) {
                    if let Some(host) = b.browser().and_then(|b| b.host()) {
                        all &= host.try_close_browser();
                    }
                }
                all
            })
            .unwrap_or(1)
        }

        fn is_frameless(&self, _window: Option<&mut Window>) -> i32 {
            // The UI draws its own title bar; `ZAPHIRE_NATIVE_FRAME=1` brings
            // the system frame back (useful on unusual window managers).
            (std::env::var_os("ZAPHIRE_NATIVE_FRAME").is_none()) as i32
        }

        fn can_maximize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_minimize(&self, _window: Option<&mut Window>) -> i32 { 1 }
        fn can_resize(&self, _window: Option<&mut Window>) -> i32 { 1 }
    }
}
