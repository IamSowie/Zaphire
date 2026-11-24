//! `zaphire://app/…` — serves the React UI and receives UI commands.
//!
//! * `GET  zaphire://app/<path>` → file from the UI bundle (SPA fallback to
//!   `index.html`).
//! * `POST zaphire://app/api`    → JSON [`zaphire_ipc::Command`], only from
//!   the UI view or Zaphire's own pages.

use crate::{app, shell, tasks};
use cef::wrapper::stream_resource_handler::StreamResourceHandler;
use cef::*;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::Ordering;
use zaphire_ipc::Command;

/// Folder holding the built UI (`ui/dist`).
pub fn ui_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("ZAPHIRE_UI_DIR") {
            return PathBuf::from(d);
        }
        if let Some(d) = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.join("ui")))
            && d.join("index.html").exists()
        {
            return d;
        }
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui/dist")
    })
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html",
        "js" | "mjs" => "text/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn respond(status: i32, mime: &str, mut body: Vec<u8>) -> Option<ResourceHandler> {
    // CreateForData copies the bytes, so `body` may be dropped afterwards.
    let stream = stream_reader_create_for_data(body.as_mut_ptr(), body.len())?;
    let text = if status == 200 { "OK" } else { "Error" };
    Some(StreamResourceHandler::new(
        status,
        text.into(),
        mime.into(),
        None,
        Some(stream),
    ))
}

fn serve_file(path: &str) -> Option<ResourceHandler> {
    let rel = path.trim_start_matches('/');
    // No escaping the UI folder.
    if rel.split('/').any(|seg| seg == "..") {
        return respond(403, "text/plain", b"forbidden".to_vec());
    }
    let dir = ui_dir();
    let file = if rel.is_empty() || !rel.contains('.') {
        dir.join("index.html")
    } else {
        dir.join(rel)
    };
    match std::fs::read(&file) {
        Ok(bytes) => respond(200, mime_for(&file.to_string_lossy()), bytes),
        Err(_) => respond(404, "text/plain", format!("not found: {rel}").into_bytes()),
    }
}

fn post_body(request: &mut Request) -> Vec<u8> {
    let mut out = vec![];
    let Some(data) = request.post_data() else {
        return out;
    };
    // CEF reads the vector length as the number of elements to return.
    let mut elements = vec![None; data.element_count()];
    data.elements(Some(&mut elements));
    for el in elements.into_iter().flatten() {
        let n = el.bytes_count();
        let start = out.len();
        out.resize(start + n, 0);
        el.bytes(n, out[start..].as_mut_ptr());
    }
    out
}

wrap_scheme_handler_factory! {
    pub struct ZaphireSchemeFactory;

    impl SchemeHandlerFactory {
        fn create(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _scheme_name: Option<&CefString>,
            request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {
            let request = request?;
            let url = CefString::from(&request.url()).to_string();
            let path = url
                .strip_prefix("zaphire://app")
                .unwrap_or("/")
                .split(['?', '#'])
                .next()
                .unwrap_or("/")
                .to_string();

            if !path.starts_with("/api") {
                return serve_file(&path);
            }

            // Who is calling? Only the UI view or a zaphire:// page.
            let from_ui = browser
                .map(|b| b.identifier() == app::global().ui_browser_id.load(Ordering::Relaxed))
                .unwrap_or(false);
            let from_own_page = frame
                .map(|f| CefString::from(&f.url()).to_string().starts_with("zaphire://app/"))
                .unwrap_or(false);
            if !(from_ui || from_own_page) {
                log::warn!("rejected api call from {url}");
                return respond(403, "application/json", br#"{"error":"forbidden"}"#.to_vec());
            }
            // Pages inside tabs (settings, new tab) poll the latest state.
            if path == "/api/state" {
                let json = shell::LAST_STATE.lock().map(|s| s.clone()).unwrap_or_default();
                return respond(200, "application/json", json.into_bytes());
            }
            let is_post = CefString::from(&request.method()).to_string() == "POST";
            if !(is_post && path == "/api") {
                log::warn!("rejected api call from {url}");
                return respond(403, "application/json", br#"{"error":"forbidden"}"#.to_vec());
            }

            match Command::parse(&post_body(request)) {
                Ok(cmd) => {
                    tasks::on_ui(0, move || shell::handle(cmd));
                    respond(200, "application/json", b"{}".to_vec())
                }
                Err(e) => respond(400, "application/json", serde_json::json!({ "error": e.to_string() }).to_string().into_bytes()),
            }
        }
    }
}
