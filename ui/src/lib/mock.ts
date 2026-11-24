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
    theme: { accent: "#3b82f6", glass: 0.55, radius: 12, reduce_motion: false },
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
  };
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
    setTimeout(push, 0);
  };
}
