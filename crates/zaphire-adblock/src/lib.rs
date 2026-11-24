//! Network-level ad and tracker blocking.
//!
//! Filter lists use Adblock Plus syntax (EasyList, EasyPrivacy, uBlock lists).
//! Zaphire ships a small built-in list so blocking works offline on first
//! start, and loads every `*.txt` in `<config>/filters/` on top of it.

use adblock::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Built-in rules: the most common ad and tracking networks.
pub const BUILTIN_RULES: &str = include_str!("builtin.txt");

/// Lists Zaphire offers to download into `<config>/filters/`.
pub const RECOMMENDED_LISTS: [(&str, &str); 2] = [
    ("easylist.txt", "https://easylist.to/easylist/easylist.txt"),
    (
        "easyprivacy.txt",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
];

pub struct Blocker {
    engine: Engine,
    blocked: AtomicU64,
}

impl Blocker {
    pub fn from_rules<'a>(lists: impl IntoIterator<Item = &'a str>) -> Self {
        let mut set = FilterSet::new(false);
        for list in lists {
            set.add_filter_list(list.to_string(), ParseOptions::default());
        }
        Self {
            engine: Engine::new_with_filter_set(set),
            blocked: AtomicU64::new(0),
        }
    }

    /// Built-in rules plus every `.txt` file in `dir` (missing dir is fine).
    pub fn load(dir: &Path) -> Self {
        let mut lists = vec![BUILTIN_RULES.to_string()];
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "txt") {
                    match std::fs::read_to_string(&p) {
                        Ok(s) => lists.push(s),
                        Err(err) => log::warn!("adblock: cannot read {}: {err}", p.display()),
                    }
                }
            }
        }
        log::info!("adblock: loaded {} list(s)", lists.len());
        Self::from_rules(lists.iter().map(String::as_str))
    }

    /// Should a request be blocked?
    /// `kind` is an adblock request type: `script`, `image`, `stylesheet`,
    /// `xmlhttprequest`, `document`, `subdocument`, `media`, `font`, `other`…
    pub fn should_block(&self, url: &str, source_url: &str, kind: &str) -> bool {
        // Never block top-level navigations or our own pages.
        if kind == "document" || url.starts_with("zaphire:") || url.starts_with("chrome") {
            return false;
        }
        let Ok(req) = Request::new(url, source_url, kind, "GET") else {
            return false;
        };
        let res = self.engine.check_network_request(&req);
        let block = res.filter.is_some() && (res.exception.is_none() || res.important);
        if block {
            self.blocked.fetch_add(1, Ordering::Relaxed);
        }
        block
    }

    /// Total requests blocked since start.
    pub fn blocked_count(&self) -> u64 {
        self.blocked.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_blocks_known_ad_hosts() {
        let b = Blocker::from_rules([BUILTIN_RULES]);
        assert!(b.should_block(
            "https://securepubads.g.doubleclick.net/tag/js/gpt.js",
            "https://news.example.com/",
            "script"
        ));
        assert!(b.should_block(
            "https://www.google-analytics.com/analytics.js",
            "https://example.com/",
            "script"
        ));
        assert!(!b.should_block(
            "https://example.com/app.js",
            "https://example.com/",
            "script"
        ));
        assert_eq!(b.blocked_count(), 2);
    }

    #[test]
    fn never_blocks_documents_and_respects_exceptions() {
        let b = Blocker::from_rules(["||ads.example^", "@@||ads.example/allowed.js"]);
        assert!(!b.should_block("https://ads.example/", "", "document"));
        assert!(b.should_block("https://ads.example/x.js", "https://site.test/", "script"));
        assert!(!b.should_block(
            "https://ads.example/allowed.js",
            "https://site.test/",
            "script"
        ));
    }

    #[test]
    fn engine_is_thread_safe() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Blocker>();
    }
}
