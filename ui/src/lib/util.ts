import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { Session, Settings, Tab, Theme } from "./types";

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

const darkQuery = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;

export function systemPrefersDark(): boolean {
  return darkQuery?.matches ?? true;
}

export function onSystemThemeChange(cb: (dark: boolean) => void): () => void {
  const h = (e: MediaQueryListEvent) => cb(e.matches);
  darkQuery?.addEventListener("change", h);
  return () => darkQuery?.removeEventListener("change", h);
}

export function resolvedMode(t: Theme): "dark" | "light" {
  return t.mode === "system" ? (systemPrefersDark() ? "dark" : "light") : t.mode;
}

/** Apply theme settings to the document (attributes + CSS variables). */
export function applyTheme(settings: Settings) {
  const t = settings.theme;
  const root = document.documentElement;
  root.dataset.theme = resolvedMode(t);
  root.dataset.bg = t.background;
  root.dataset.density = t.density;
  const r = root.style;
  r.setProperty("--z-accent", t.accent);
  r.setProperty("--z-glass", String(t.glass));
  if (t.density === "compact") r.removeProperty("--z-radius");
  else r.setProperty("--z-radius", `${t.radius}px`);
  if (t.background_color && /^#[0-9a-f]{6}$/i.test(t.background_color)) r.setProperty("--z-bg", t.background_color);
  else r.removeProperty("--z-bg");
  r.setProperty("zoom", String(Math.min(1.25, Math.max(0.85, t.scale || 1))));
  root.classList.toggle("reduce-motion", t.reduce_motion);
}

/** Current UI zoom, to convert CSS pixels to window pixels. */
export function uiZoom(): number {
  return Number(document.documentElement.style.getPropertyValue("zoom")) || 1;
}

export const compact = new Intl.NumberFormat("es", { notation: "compact", maximumFractionDigits: 1 });
