#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
//! Zaphire entry point.
//!
//! Chromium runs several processes (browser, renderer, GPU, utility). They
//! all start from this same executable; only the browser process gets past
//! `execute_process` and builds the window.

mod app;
mod clients;
mod scheme;
mod shell;
mod tasks;

use cef::*;

fn main() {
    // Initialize the CEF API version before any other call.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = args::Args::new();
    let Some(cmd_line) = args.as_cmd_line() else {
        eprintln!("zaphire: cannot parse command line");
        std::process::exit(1);
    };
    let is_browser_process = cmd_line.has_switch(Some(&CefString::from("type"))) != 1;

    let mut app = app::ZaphireApp::new(app::AppState::load());

    // Sub-processes run here and return their exit code.
    let code = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    if !is_browser_process {
        std::process::exit(code.max(0));
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Zaphire {} starting", env!("CARGO_PKG_VERSION"));

    let cache = zaphire_core::Settings::config_dir().join("profile");
    let settings = Settings {
        no_sandbox: 1,
        root_cache_path: CefString::from(cache.to_string_lossy().as_ref()),
        cache_path: CefString::from(cache.join("Default").to_string_lossy().as_ref()),
        background_color: 0xff0b1020,
        persist_session_cookies: 1,
        ..Default::default()
    };

    if initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut(),
    ) != 1
    {
        eprintln!("zaphire: CEF failed to initialize");
        std::process::exit(1);
    }
    run_message_loop();
    shutdown();
}
