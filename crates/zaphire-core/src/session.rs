//! Tabs, workspaces and split view.
//!
//! The session is the single source of truth for what the UI shows. The CEF
//! layer only keeps a map from [`TabId`] to the native browser.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type TabId = u32;
pub type WorkspaceId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: TabId,
    pub workspace: WorkspaceId,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub pinned: bool,
    /// Unloaded to save memory; reloaded from `url` when activated.
    pub suspended: bool,
    pub audible: bool,
    /// Seconds (monotonic, caller-defined clock) when the tab was last visible.
    #[serde(skip)]
    pub last_visible: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    /// Emoji or short glyph shown in the switcher.
    pub icon: String,
    pub color: String,
    /// Tab order inside the workspace.
    pub tabs: Vec<TabId>,
    pub active: Option<TabId>,
    /// Second pane of the split view, shown to the right of `active`.
    pub split: Option<TabId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub workspaces: Vec<Workspace>,
    pub tabs: BTreeMap<TabId, Tab>,
    pub current: WorkspaceId,
    next_id: u32,
}

impl Default for Session {
    fn default() -> Self {
        let mut s = Self {
            workspaces: vec![],
            tabs: BTreeMap::new(),
            current: 0,
            next_id: 1,
        };
        let ws = s.add_workspace("Personal", "✦", "#3b82f6");
        s.current = ws;
        s
    }
}

impl Session {
    fn next(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    fn workspace_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    pub fn current_workspace(&self) -> &Workspace {
        self.workspace(self.current)
            .expect("current workspace exists")
    }

    pub fn add_workspace(&mut self, name: &str, icon: &str, color: &str) -> WorkspaceId {
        let id = self.next();
        self.workspaces.push(Workspace {
            id,
            name: name.into(),
            icon: icon.into(),
            color: color.into(),
            tabs: vec![],
            active: None,
            split: None,
        });
        id
    }

    /// Remove a workspace and its tabs. The last workspace cannot be removed.
    /// Returns the ids of the closed tabs.
    pub fn remove_workspace(&mut self, id: WorkspaceId) -> Vec<TabId> {
        if self.workspaces.len() <= 1 {
            return vec![];
        }
        let Some(pos) = self.workspaces.iter().position(|w| w.id == id) else {
            return vec![];
        };
        let ws = self.workspaces.remove(pos);
        for t in &ws.tabs {
            self.tabs.remove(t);
        }
        if self.current == id {
            self.current = self.workspaces[pos.saturating_sub(1)].id;
        }
        ws.tabs
    }

    pub fn switch_workspace(&mut self, id: WorkspaceId) -> bool {
        if self.workspace(id).is_some() {
            self.current = id;
            true
        } else {
            false
        }
    }

    /// Open a tab in the current workspace, right after the active one, and
    /// make it active.
    pub fn new_tab(&mut self, url: &str, now: u64) -> TabId {
        let id = self.next();
        let ws_id = self.current;
        self.tabs.insert(
            id,
            Tab {
                id,
                workspace: ws_id,
                url: url.into(),
                title: String::new(),
                favicon: None,
                loading: true,
                can_go_back: false,
                can_go_forward: false,
                pinned: false,
                suspended: false,
                audible: false,
                last_visible: now,
            },
        );
        let ws = self.workspace_mut(ws_id).expect("current workspace exists");
        let at = ws
            .active
            .and_then(|a| ws.tabs.iter().position(|t| *t == a))
            .map_or(ws.tabs.len(), |p| p + 1);
        ws.tabs.insert(at, id);
        ws.active = Some(id);
        id
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.get(&id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.get_mut(&id)
    }

    /// Close a tab. If it was active, the neighbour to the right (else left)
    /// becomes active. Returns `true` if the tab existed.
    pub fn close_tab(&mut self, id: TabId) -> bool {
        let Some(tab) = self.tabs.remove(&id) else {
            return false;
        };
        let ws = self
            .workspace_mut(tab.workspace)
            .expect("tab's workspace exists");
        let pos = ws
            .tabs
            .iter()
            .position(|t| *t == id)
            .expect("tab listed in workspace");
        ws.tabs.remove(pos);
        if ws.split == Some(id) {
            ws.split = None;
        }
        if ws.active == Some(id) {
            ws.active = ws
                .tabs
                .get(pos)
                .or_else(|| ws.tabs.get(pos.wrapping_sub(1)))
                .copied();
            if ws.active.is_some() && ws.active == ws.split {
                ws.split = None;
            }
        }
        true
    }

    /// Make a tab active, switching workspace if needed.
    pub fn activate(&mut self, id: TabId, now: u64) -> bool {
        let Some(tab) = self.tabs.get_mut(&id) else {
            return false;
        };
        tab.last_visible = now;
        let ws_id = tab.workspace;
        self.current = ws_id;
        let ws = self.workspace_mut(ws_id).expect("tab's workspace exists");
        if ws.split == Some(id) {
            ws.split = ws.active;
        }
        ws.active = Some(id);
        true
    }

    /// Show `id` next to the active tab, or close the split with `None`.
    pub fn set_split(&mut self, id: Option<TabId>) -> bool {
        let current = self.current;
        if let Some(t) = id {
            match self.tabs.get(&t) {
                Some(tab) if tab.workspace == current => {}
                _ => return false,
            }
        }
        let ws = self
            .workspace_mut(current)
            .expect("current workspace exists");
        if id.is_some() && id == ws.active {
            return false;
        }
        ws.split = id;
        true
    }

    pub fn move_tab(&mut self, id: TabId, to_ws: WorkspaceId, index: usize) -> bool {
        if self.workspace(to_ws).is_none() || !self.tabs.contains_key(&id) {
            return false;
        }
        let from_ws = self.tabs[&id].workspace;
        // Detach.
        let ws = self.workspace_mut(from_ws).expect("exists");
        let pos = ws.tabs.iter().position(|t| *t == id).expect("listed");
        ws.tabs.remove(pos);
        if from_ws != to_ws {
            if ws.split == Some(id) {
                ws.split = None;
            }
            if ws.active == Some(id) {
                ws.active = ws
                    .tabs
                    .get(pos)
                    .or_else(|| ws.tabs.get(pos.wrapping_sub(1)))
                    .copied();
            }
        }
        // Attach.
        let ws = self.workspace_mut(to_ws).expect("exists");
        let index = index.min(ws.tabs.len());
        ws.tabs.insert(index, id);
        if ws.active.is_none() {
            ws.active = Some(id);
        }
        self.tabs.get_mut(&id).expect("exists").workspace = to_ws;
        true
    }

    /// Tabs on screen right now: the active tab and the split partner.
    pub fn visible_tabs(&self) -> Vec<TabId> {
        let ws = self.current_workspace();
        ws.active.into_iter().chain(ws.split).collect()
    }

    /// Background tabs that have not been visible for `after_secs`. Pinned,
    /// audible and already-suspended tabs are skipped.
    pub fn tabs_to_suspend(&self, now: u64, after_secs: u64) -> Vec<TabId> {
        let visible = self.visible_tabs();
        self.tabs
            .values()
            .filter(|t| {
                !visible.contains(&t.id)
                    && !t.pinned
                    && !t.audible
                    && !t.suspended
                    && now.saturating_sub(t.last_visible) >= after_secs
            })
            .map(|t| t.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_close_activates_neighbour() {
        let mut s = Session::default();
        let a = s.new_tab("https://a", 0);
        let b = s.new_tab("https://b", 0);
        let c = s.new_tab("https://c", 0);
        assert_eq!(s.current_workspace().tabs, vec![a, b, c]);
        s.activate(b, 1);
        let d = s.new_tab("https://d", 1);
        assert_eq!(
            s.current_workspace().tabs,
            vec![a, b, d, c],
            "opens after active"
        );
        assert!(s.close_tab(d));
        assert_eq!(s.current_workspace().active, Some(c), "right neighbour");
        s.close_tab(c);
        assert_eq!(s.current_workspace().active, Some(b), "left when last");
        s.close_tab(b);
        s.close_tab(a);
        assert_eq!(s.current_workspace().active, None);
        assert!(!s.close_tab(a));
    }

    #[test]
    fn split_view_rules() {
        let mut s = Session::default();
        let a = s.new_tab("https://a", 0);
        let b = s.new_tab("https://b", 0);
        assert!(!s.set_split(Some(b)), "cannot split with itself");
        assert!(s.set_split(Some(a)));
        assert_eq!(s.visible_tabs(), vec![b, a]);
        // Activating the split partner swaps panes.
        s.activate(a, 1);
        assert_eq!(s.visible_tabs(), vec![a, b]);
        s.close_tab(b);
        assert_eq!(s.current_workspace().split, None);
    }

    #[test]
    fn workspaces_and_moving() {
        let mut s = Session::default();
        let first = s.current;
        let a = s.new_tab("https://a", 0);
        let work = s.add_workspace("Work", "💼", "#22c55e");
        assert!(s.move_tab(a, work, 0));
        assert_eq!(s.current_workspace().active, None);
        assert_eq!(s.workspace(work).unwrap().active, Some(a));
        s.activate(a, 0);
        assert_eq!(s.current, work);
        assert_eq!(s.remove_workspace(work), vec![a]);
        assert_eq!(s.current, first);
        assert!(s.remove_workspace(first).is_empty(), "keeps the last one");
    }

    #[test]
    fn suspension_candidates() {
        let mut s = Session::default();
        let a = s.new_tab("https://a", 0);
        let b = s.new_tab("https://b", 0);
        let c = s.new_tab("https://c", 0);
        s.tab_mut(b).unwrap().pinned = true;
        // c is active and visible.
        assert_eq!(s.tabs_to_suspend(100, 60), vec![a]);
        assert!(s.tabs_to_suspend(30, 60).is_empty());
        s.tab_mut(a).unwrap().suspended = true;
        assert!(s.tabs_to_suspend(100, 60).is_empty());
        let _ = c;
    }

    #[test]
    fn serializes_camel_case() {
        let mut s = Session::default();
        s.new_tab("https://a", 0);
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("canGoBack"));
        assert!(!json.contains("last_visible") && !json.contains("lastVisible"));
    }
}
