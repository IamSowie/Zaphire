// A fake core used when the UI runs in a normal browser (`npm run dev`,
// screenshots, tests). It mimics the Rust session rules closely enough to
// click around; the real logic lives in crates/zaphire-core.
import type { Command, Event, Settings, State, Tab } from "./types";

export function createMock(emit: (e: Event) => void) {
  let next = 10;
  const tab = (id: number, url: string, title: string, extra: Partial<Tab> = {}): Tab => ({
    id,
    workspace: 1,
    url,
    title,
    favicon: null,
    loading: false,
    canGoBack: true,
    canGoForward: false,
    pinned: false,
    suspended: false,
    audible: false,
    ...extra,
  });
  const settings: Settings = {
    tab_layout: "vertical",
    profile: "balanced",
    adblock: true,
    search_engine: "https://duckduckgo.com/?q={}",
    homepage: "zaphire://app/newtab",
    sidebar_width: 280,
    theme: {
      mode: "dark",
      background: "gradient",
      background_color: null,
      accent: "#3b82f6",
      glass: 0.55,
      radius: 12,
      scale: 1,
      density: "comfortable",
      reduce_motion: false,
    },
    restore_session: true,
    show_bookmarks: true,
    show_extensions: true,
  };
  const s: State = {
    session: {
      current: 1,
      workspaces: [
        { id: 1, name: "Personal", icon: "✦", color: "#3b82f6", tabs: [2, 3, 4, 5], active: 3, split: null },
        { id: 6, name: "Trabajo", icon: "💼", color: "#22c55e", tabs: [7], active: 7, split: null },
      ],
      tabs: {
        2: tab(2, "https://github.com/", "GitHub", { pinned: true }),
        3: tab(3, "https://www.rust-lang.org/", "Rust Programming Language"),
        4: tab(4, "https://krunker.io/", "Krunker", { audible: true }),
        5: tab(5, "https://news.ycombinator.com/", "Hacker News", { suspended: true }),
        7: { ...tab(7, "https://docs.rs/cef", "cef - Rust"), workspace: 6 },
      },
    },
    settings,
    blocked: 1284,
    restartNeeded: false,
    platform: "linux",
    bookmarks: [
      { url: "https://github.com/", title: "GitHub", added: 0 },
      { url: "https://doc.rust-lang.org/book/", title: "The Rust Book", added: 0 },
      { url: "https://www.youtube.com/", title: "YouTube", added: 0 },
    ],
    extensions: [
      { id: "cjpalhdlnbpafiamejdnhcphjbkeiagm", name: "uBlock Origin", popup: "popup.html", hasIcon: false },
      { id: "nngceckbapebfimnlniiiahkandclblb", name: "Bitwarden", popup: "popup/index.html", hasIcon: false },
      { id: "eimadpbcbfnmbkopoojfekhnkhdbieeh", name: "Dark Reader", popup: "ui/popup/index.html", hasIcon: false },
    ],
    bookmarked: false,
  };
  const history: { url: string; title: string; time: number }[] = [
    { url: "https://www.rust-lang.org/", title: "Rust Programming Language", time: Date.now() / 1000 - 60 },
    { url: "https://github.com/", title: "GitHub", time: Date.now() / 1000 - 3600 },
    { url: "https://news.ycombinator.com/", title: "Hacker News", time: Date.now() / 1000 - 90000 },
  ];
  (globalThis as { __zaphireMockHistory?: typeof history }).__zaphireMockHistory = history;
  const ws = () => s.session.workspaces.find((w) => w.id === s.session.current)!;
  const push = () => emit({ type: "state", ...structuredClone(s) });

  return (cmd: Command) => {
    const w = ws();
    switch (cmd.type) {
      case "newTab": {
        const id = next++;
        s.session.tabs[id] = { ...tab(id, cmd.url ?? "zaphire://app/newtab", "Nueva pestaña"), workspace: w.id, canGoBack: false };
        const at = w.active ? w.tabs.indexOf(w.active) + 1 : w.tabs.length;
        w.tabs.splice(at, 0, id);
        w.active = id;
        break;
      }
      case "closeTab": {
        const i = w.tabs.indexOf(cmd.id);
        if (i < 0) break;
        w.tabs.splice(i, 1);
        delete s.session.tabs[cmd.id];
        if (w.split === cmd.id) w.split = null;
        if (w.active === cmd.id) w.active = w.tabs[i] ?? w.tabs[i - 1] ?? null;
        break;
      }
      case "activateTab":
        if (w.split === cmd.id) w.split = w.active;
        w.active = cmd.id;
        s.session.tabs[cmd.id].suspended = false;
        break;
      case "pinTab":
        s.session.tabs[cmd.id].pinned = cmd.pinned;
        break;
      case "navigate": {
        const id = cmd.id ?? w.active;
        if (id) Object.assign(s.session.tabs[id], { url: cmd.input, title: cmd.input });
        break;
      }
      case "setSplit":
        w.split = cmd.id === w.active ? null : cmd.id;
        break;
      case "switchWorkspace":
        s.session.current = cmd.id;
        break;
      case "newWorkspace": {
        const id = next++;
        s.session.workspaces.push({ id, name: cmd.name, icon: cmd.icon, color: cmd.color, tabs: [], active: null, split: null });
        s.session.current = id;
        break;
      }
      case "toggleBookmark": {
        const t = w.active ? s.session.tabs[w.active] : null;
        if (!t) break;
        const i = s.bookmarks.findIndex((b) => b.url === t.url);
        if (i >= 0) s.bookmarks.splice(i, 1);
        else s.bookmarks.push({ url: t.url, title: t.title, added: 0 });
        break;
      }
      case "removeBookmark":
        s.bookmarks = s.bookmarks.filter((b) => b.url !== cmd.url);
        break;
      case "updateSettings":
        s.settings = cmd.settings;
        break;
      case "setProfile":
        s.restartNeeded = cmd.profile !== "balanced";
        s.settings.profile = cmd.profile;
        break;
      default:
        break;
    }
    const active = ws().active;
    s.bookmarked = active != null && s.bookmarks.some((b) => b.url === s.session.tabs[active]?.url);
    setTimeout(push, 0);
  };
}
