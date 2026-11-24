import { History, Puzzle, Settings2, ShieldCheck, ShieldOff, RefreshCw } from "lucide-react";
import { send } from "../lib/ipc";
import type { State } from "../lib/types";
import { activeTab, cn, compact } from "../lib/util";
import { AddressBar, IconButton, NavButtons, ProfileSwitch, WindowControls } from "./controls";
import { HorizontalTabs, VerticalTabs } from "./tabs";
import { Workspaces } from "./workspaces";
import { ExtensionBar } from "./extensions";
import { SidebarBookmarks } from "./bookmarks";

function toggleSplit(s: State) {
  const ws = s.session.workspaces.find((w) => w.id === s.session.current)!;
  const other = ws.tabs.find((t) => t !== ws.active);
  send({ type: "setSplit", id: ws.split ? null : (other ?? null) });
}

function openSettings() {
  send({ type: "newTab", url: "zaphire://app/settings" });
}

function openLibrary() {
  send({ type: "newTab", url: "zaphire://app/library" });
}

function Brand() {
  return (
    <div className="flex items-center gap-2 px-1">
      <img src="/zaphire.svg" width={18} height={18} alt="" />
      <span className="text-[13px] font-bold tracking-wide">Zaphire</span>
    </div>
  );
}

function ShieldBadge({ s }: { s: State }) {
  return (
    <button
      title={s.settings.adblock ? `Bloqueador activo · ${s.blocked} peticiones bloqueadas` : "Bloqueador desactivado"}
      onClick={() => send({ type: "updateSettings", settings: { ...s.settings, adblock: !s.settings.adblock } })}
      className="no-drag z-press flex h-8 items-center gap-1.5 rounded-[calc(var(--z-radius)-4px)] px-2 text-dim hover:bg-panel-hover hover:text-text"
    >
      {s.settings.adblock ? <ShieldCheck size={15} className="text-success" /> : <ShieldOff size={15} />}
      <span className="text-[12px] tabular-nums">{s.settings.adblock ? compact.format(s.blocked) : "off"}</span>
    </button>
  );
}

function RestartHint({ s }: { s: State }) {
  if (!s.restartNeeded) return null;
  return (
    <button
      onClick={() => send({ type: "restart" })}
      title="El perfil cambia opciones de Chromium que solo se aplican al arrancar"
      className="no-drag z-press flex items-center gap-1.5 rounded-[calc(var(--z-radius)-4px)] bg-warning/15 px-2 py-1 text-[12px] text-warning hover:bg-warning/25"
    >
      <RefreshCw size={12} /> Reiniciar para aplicar
    </button>
  );
}

/** Sidebar layout (vertical tabs). */
export function SidebarShell({ s }: { s: State }) {
  const tab = activeTab(s.session);
  const ws = s.session.workspaces.find((w) => w.id === s.session.current)!;
  return (
    <aside className="z-backdrop flex h-full flex-col gap-2 border-r border-line pt-1.5">
      <div className="drag flex h-9 items-center justify-between pr-1.5 pl-3">
        <Brand />
        <WindowControls />
      </div>
      <div className="flex items-center justify-between px-2">
        <NavButtons tab={tab} split={ws.split != null} onSplit={() => toggleSplit(s)} />
        <ShieldBadge s={s} />
      </div>
      <AddressBar tab={tab} className="mx-2" bookmarked={s.bookmarked} />
      {s.settings.show_extensions && s.extensions.length > 0 && (
        <ExtensionBar extensions={s.extensions} side="right" className="px-2" />
      )}
      <div className="mx-3 my-1 h-px bg-line" />
      {s.settings.show_bookmarks && <SidebarBookmarks bookmarks={s.bookmarks} />}
      <VerticalTabs session={s.session} />
      <footer className="flex flex-col gap-2 border-t border-line p-2">
        <RestartHint s={s} />
        <ProfileSwitch value={s.settings.profile} />
        <div className="flex items-center justify-between">
          <Workspaces session={s.session} />
          <div className="flex items-center">
            <IconButton title="Historial y marcadores (Ctrl+H)" onClick={openLibrary}>
              <History size={15} />
            </IconButton>
            <IconButton title="Extensiones" onClick={() => send({ type: "openExtensions" })}>
              <Puzzle size={15} />
            </IconButton>
            <IconButton title="Ajustes" onClick={openSettings}>
              <Settings2 size={15} />
            </IconButton>
          </div>
        </div>
      </footer>
    </aside>
  );
}

/** Top-bar layout (horizontal tabs): tab strip row + toolbar row. */
export function TopBarShell({ s }: { s: State }) {
  const tab = activeTab(s.session);
  const ws = s.session.workspaces.find((w) => w.id === s.session.current)!;
  return (
    <header className="z-backdrop flex h-full flex-col border-b border-line">
      <div className="drag flex h-11 items-center gap-2 pr-1.5 pl-3">
        <Brand />
        <HorizontalTabs session={s.session} />
        <WindowControls />
      </div>
      <div className="flex h-11 items-center gap-2 px-2 pb-1">
        <NavButtons tab={tab} split={ws.split != null} onSplit={() => toggleSplit(s)} />
        <AddressBar tab={tab} className="flex-1" bookmarked={s.bookmarked} />
        <RestartHint s={s} />
        {s.settings.show_extensions && s.extensions.length > 0 && (
          <ExtensionBar extensions={s.extensions} side="below" className="max-w-64" />
        )}
        <ShieldBadge s={s} />
        <div className="w-44">
          <ProfileSwitch value={s.settings.profile} compact />
        </div>
        <Workspaces session={s.session} popupSide="left" />
        <div className={cn("flex items-center")}>
          <IconButton title="Historial y marcadores (Ctrl+H)" onClick={openLibrary}>
            <History size={15} />
          </IconButton>
          {!(s.settings.show_extensions && s.extensions.length > 0) && (
            <IconButton title="Extensiones" onClick={() => send({ type: "openExtensions" })}>
              <Puzzle size={15} />
            </IconButton>
          )}
          <IconButton title="Ajustes" onClick={openSettings}>
            <Settings2 size={15} />
          </IconButton>
        </div>
      </div>
    </header>
  );
}
