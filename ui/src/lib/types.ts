// Mirrors crates/zaphire-core and crates/zaphire-ipc. Keep in sync.

export type TabId = number;
export type WorkspaceId = number;
export type Profile = "saver" | "balanced" | "game";
export type TabLayout = "vertical" | "horizontal";

export interface Tab {
  id: TabId;
  workspace: WorkspaceId;
  url: string;
  title: string;
  favicon: string | null;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  pinned: boolean;
  suspended: boolean;
  audible: boolean;
}

export interface Workspace {
  id: WorkspaceId;
  name: string;
  icon: string;
  color: string;
  tabs: TabId[];
  active: TabId | null;
  split: TabId | null;
}

export interface Session {
  workspaces: Workspace[];
  tabs: Record<string, Tab>;
  current: WorkspaceId;
}

export interface Theme {
  accent: string;
  glass: number;
  radius: number;
  reduce_motion: boolean;
}

export interface Settings {
  tab_layout: TabLayout;
  profile: Profile;
  adblock: boolean;
  search_engine: string;
  homepage: string;
  sidebar_width: number;
  theme: Theme;
}

export interface State {
  session: Session;
  settings: Settings;
  blocked: number;
  restartNeeded: boolean;
  platform: string;
}

export type Event = ({ type: "state" } & State) | { type: "focusAddress" };

export type Command =
  | { type: "ready" }
  | { type: "newTab"; url: string | null }
  | { type: "closeTab"; id: TabId }
  | { type: "activateTab"; id: TabId }
  | { type: "pinTab"; id: TabId; pinned: boolean }
  | { type: "navigate"; id: TabId | null; input: string }
  | { type: "back" }
  | { type: "forward" }
  | { type: "reload" }
  | { type: "stop" }
  | { type: "setSplit"; id: TabId | null }
  | { type: "newWorkspace"; name: string; icon: string; color: string }
  | { type: "switchWorkspace"; id: WorkspaceId }
  | { type: "removeWorkspace"; id: WorkspaceId }
  | { type: "moveTab"; id: TabId; workspace: WorkspaceId; index: number }
  | { type: "updateSettings"; settings: Settings }
  | { type: "setProfile"; profile: Profile }
  | { type: "openExtensions" }
  | { type: "devTools" }
  | { type: "restart" }
  | { type: "minimize" }
  | { type: "toggleMaximize" }
  | { type: "closeWindow" };
