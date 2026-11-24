//! History, bookmarks and the saved session, stored as JSON next to the
//! settings (`history.json`, `bookmarks.json`, `session.json`).

use crate::session::Session;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

/// Oldest visits are dropped past this many entries.
pub const HISTORY_LIMIT: usize = 5_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visit {
    pub url: String,
    pub title: String,
    /// Unix seconds.
    pub time: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub url: String,
    pub title: String,
    /// Unix seconds.
    pub added: u64,
}

#[derive(Debug, Default)]
pub struct Library {
    dir: PathBuf,
    history: VecDeque<Visit>,
    pub bookmarks: Vec<Bookmark>,
    dirty: bool,
}

/// Pages that are not worth remembering.
fn recordable(url: &str) -> bool {
    !(url.is_empty()
        || url.starts_with("zaphire:")
        || url.starts_with("chrome")
        || url.starts_with("about:")
        || url.starts_with("data:")
        || url.starts_with("devtools:"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec(value).map_err(std::io::Error::other)?,
    )?;
    std::fs::rename(tmp, path)
}

impl Library {
    pub fn load(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            history: read_json(&dir.join("history.json")).unwrap_or_default(),
            bookmarks: read_json(&dir.join("bookmarks.json")).unwrap_or_default(),
            dirty: false,
        }
    }

    /// Record a finished page load. Reloads and title updates of the most
    /// recent visit to the same URL update it instead of adding a row.
    pub fn record(&mut self, url: &str, title: &str, now: u64) {
        if !recordable(url) {
            return;
        }
        if let Some(last) = self.history.back_mut().filter(|v| v.url == url) {
            if !title.is_empty() {
                last.title = title.to_string();
            }
            last.time = now;
        } else {
            self.history.push_back(Visit {
                url: url.into(),
                title: title.into(),
                time: now,
            });
            while self.history.len() > HISTORY_LIMIT {
                self.history.pop_front();
            }
        }
        self.dirty = true;
    }

    /// A page's title arrived after it finished loading: fix the latest
    /// visit to that URL.
    pub fn update_title(&mut self, url: &str, title: &str) {
        if let Some(v) = self.history.iter_mut().rev().find(|v| v.url == url)
            && v.title != title
        {
            v.title = title.to_string();
            self.dirty = true;
        }
    }

    /// Newest first; `query` matches URL or title, case-insensitively.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Visit> {
        let q = query.trim().to_lowercase();
        self.history
            .iter()
            .rev()
            .filter(|v| {
                q.is_empty()
                    || v.url.to_lowercase().contains(&q)
                    || v.title.to_lowercase().contains(&q)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn delete_url(&mut self, url: &str) {
        self.history.retain(|v| v.url != url);
        self.dirty = true;
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.dirty = true;
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|b| b.url == url)
    }

    /// Add or remove a bookmark; returns whether it is now bookmarked.
    pub fn toggle_bookmark(&mut self, url: &str, title: &str, now: u64) -> bool {
        self.dirty = true;
        if self.is_bookmarked(url) {
            self.bookmarks.retain(|b| b.url != url);
            false
        } else {
            self.bookmarks.push(Bookmark {
                url: url.into(),
                title: title.into(),
                added: now,
            });
            true
        }
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        self.bookmarks.retain(|b| b.url != url);
        self.dirty = true;
    }

    /// Write history and bookmarks if they changed.
    pub fn save(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        write_json(&self.dir.join("history.json"), &self.history)?;
        write_json(&self.dir.join("bookmarks.json"), &self.bookmarks)?;
        self.dirty = false;
        Ok(())
    }

    pub fn save_session(&self, session: &Session) -> std::io::Result<()> {
        write_json(&self.dir.join("session.json"), session)
    }

    /// The previous session with every tab unloaded, so restoring costs
    /// nothing until a tab is shown. `None` if missing, broken or empty.
    pub fn load_session(&self) -> Option<Session> {
        let mut s: Session = read_json(&self.dir.join("session.json"))?;
        if s.tabs.is_empty() || s.workspace(s.current).is_none() {
            return None;
        }
        for t in s.tabs.values_mut() {
            t.suspended = true;
            t.loading = false;
            t.audible = false;
        }
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("zaphire-lib-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        d
    }

    #[test]
    fn history_records_merges_and_searches() {
        let mut l = Library::load(&tmp("h"));
        l.record("https://a.com/", "", 1);
        l.record("https://a.com/", "A site", 2);
        l.record("zaphire://app/newtab", "New", 3);
        l.record("https://b.org/rust", "", 4);
        l.update_title("https://b.org/rust", "Rust");
        let all = l.search("", 10);
        assert_eq!(all.len(), 2, "merged reload, skipped internal page");
        assert_eq!(all[0].url, "https://b.org/rust", "newest first");
        assert_eq!(all[1].title, "A site");
        assert_eq!(l.search("RUST", 10).len(), 1);
        l.delete_url("https://a.com/");
        assert_eq!(l.search("", 10).len(), 1);
    }

    #[test]
    fn history_is_capped() {
        let mut l = Library::load(&tmp("cap"));
        for i in 0..HISTORY_LIMIT + 10 {
            l.record(&format!("https://x.com/{i}"), "", i as u64);
        }
        assert_eq!(l.search("", usize::MAX).len(), HISTORY_LIMIT);
        assert_eq!(
            l.search("", 1)[0].url,
            format!("https://x.com/{}", HISTORY_LIMIT + 9)
        );
    }

    #[test]
    fn bookmarks_and_persistence() {
        let dir = tmp("p");
        let mut l = Library::load(&dir);
        assert!(l.toggle_bookmark("https://a.com/", "A", 1));
        assert!(l.is_bookmarked("https://a.com/"));
        l.record("https://a.com/", "A", 1);
        l.save().unwrap();

        let mut s = Session::default();
        let t = s.new_tab("https://a.com/", 0);
        l.save_session(&s).unwrap();

        let l2 = Library::load(&dir);
        assert_eq!(l2.bookmarks.len(), 1);
        assert_eq!(l2.search("", 5).len(), 1);
        let restored = l2.load_session().unwrap();
        assert!(
            restored.tab(t).unwrap().suspended,
            "restored tabs load lazily"
        );
        // New ids continue after the restored ones.
        let mut restored = restored;
        assert!(restored.new_tab("https://b.com/", 0) > t);
        assert!(!Library::load(&dir).toggle_bookmark("https://a.com/", "A", 2));
        std::fs::remove_dir_all(dir).ok();
    }
}
