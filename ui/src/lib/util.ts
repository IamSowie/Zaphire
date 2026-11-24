import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { Session, Settings, Tab } from "./types";

export const cn = (...v: ClassValue[]) => twMerge(clsx(v));

/** Host shown in the address bar when it is not focused. */
export function displayUrl(url: string): string {
  if (url.startsWith("zaphire://app/newtab")) return "";
  try {
    const u = new URL(url);
    if (u.protocol === "https:" || u.protocol === "http:") return u.host.replace(/^www\./, "") + (u.pathname === "/" ? "" : u.pathname);
    return url;
  } catch {
    return url;
  }
}

export function tabTitle(t: Tab): string {
  if (t.title) return t.title;
  if (t.url.startsWith("zaphire://app/newtab")) return "Nueva pestaña";
  return displayUrl(t.url) || "Cargando…";
}

export function activeTab(s: Session): Tab | null {
  const ws = s.workspaces.find((w) => w.id === s.current);
  return ws?.active != null ? (s.tabs[ws.active] ?? null) : null;
}

export function workspaceTabs(s: Session): Tab[] {
  const ws = s.workspaces.find((w) => w.id === s.current);
  return (ws?.tabs ?? []).map((id) => s.tabs[id]).filter(Boolean);
}

/** Apply theme settings to CSS variables. */
export function applyTheme(settings: Settings) {
  const r = document.documentElement.style;
  r.setProperty("--z-accent", settings.theme.accent);
  r.setProperty("--z-glass", String(settings.theme.glass));
  r.setProperty("--z-radius", `${settings.theme.radius}px`);
  document.documentElement.classList.toggle("reduce-motion", settings.theme.reduce_motion);
}

export const compact = new Intl.NumberFormat("es", { notation: "compact", maximumFractionDigits: 1 });
