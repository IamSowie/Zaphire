import { AnimatePresence, motion } from "motion/react";
import { Columns2, Moon, Pin, PinOff, Plus, Volume2, X, FolderInput } from "lucide-react";
import { useEffect, useState } from "react";
import { send } from "../lib/ipc";
import type { Session, Tab } from "../lib/types";
import { cn, tabTitle, workspaceTabs } from "../lib/util";
import { Favicon } from "./controls";

type Menu = { tab: Tab; x: number; y: number } | null;

function TabMenu({ menu, session, onClose, row }: { menu: Menu; session: Session; onClose: () => void; row?: boolean }) {
  useEffect(() => {
    if (!menu) return;
    const close = () => onClose();
    window.addEventListener("pointerdown", close);
    window.addEventListener("blur", close);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("blur", close);
    };
  }, [menu, onClose]);
  if (!menu) return null;
  const { tab } = menu;
  const ws = session.workspaces.find((w) => w.id === session.current);
  const others = session.workspaces.filter((w) => w.id !== tab.workspace);
  const item = cn("flex items-center gap-2 rounded-md px-2 py-1.5 text-left hover:bg-panel-hover", !row && "w-full");
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.14, ease: [0.23, 1, 0.32, 1] }}
      // The UI view is only as tall as the top bar in horizontal mode, so the
      // menu becomes a one-line toolbar there instead of a dropdown.
      style={
        row
          ? { left: Math.max(4, Math.min(menu.x, window.innerWidth - 560)), top: 4 }
          : { left: Math.min(menu.x, window.innerWidth - 200), top: Math.min(menu.y, window.innerHeight - 170) }
      }
      className={cn(
        "z-glass fixed z-50 rounded-[var(--z-radius)] bg-menu p-1 text-[13px]",
        row ? "flex h-9 items-center gap-0.5 whitespace-nowrap" : "w-48",
      )}
      onPointerDown={(e) => e.stopPropagation()}
    >
      <button className={item} onClick={() => (send({ type: "pinTab", id: tab.id, pinned: !tab.pinned }), onClose())}>
        {tab.pinned ? <PinOff size={14} /> : <Pin size={14} />} {tab.pinned ? "Desfijar" : "Fijar pestaña"}
      </button>
      {ws?.active !== tab.id && (
        <button className={item} onClick={() => (send({ type: "setSplit", id: tab.id }), onClose())}>
          <Columns2 size={14} /> Vista dividida
        </button>
      )}
      {others.map((w) => (
        <button key={w.id} className={item} onClick={() => (send({ type: "moveTab", id: tab.id, workspace: w.id, index: 9999 }), onClose())}>
          <FolderInput size={14} /> {w.icon} {w.name}
        </button>
      ))}
      <button className={cn(item, "text-danger")} onClick={() => (send({ type: "closeTab", id: tab.id }), onClose())}>
        <X size={14} /> Cerrar
      </button>
    </motion.div>
  );
}

function useTabMenu() {
  const [menu, setMenu] = useState<Menu>(null);
  return { menu, open: (tab: Tab, x: number, y: number) => setMenu({ tab, x, y }), close: () => setMenu(null) };
}

function handlers(tab: Tab, open: (t: Tab, x: number, y: number) => void) {
  return {
    onClick: () => send({ type: "activateTab", id: tab.id }),
    onAuxClick: (e: React.MouseEvent) => e.button === 1 && send({ type: "closeTab", id: tab.id }),
    onContextMenu: (e: React.MouseEvent) => {
      e.preventDefault();
      open(tab, e.clientX, e.clientY);
    },
  };
}

function CloseButton({ tab, size = 13 }: { tab: Tab; size?: number }) {
  return (
    <button
      title="Cerrar (Ctrl+W)"
      onClick={(e) => {
        e.stopPropagation();
        send({ type: "closeTab", id: tab.id });
      }}
      className="grid size-5 shrink-0 place-items-center rounded-md text-faint opacity-0 group-hover:opacity-100 hover:bg-panel-hover hover:text-text"
    >
      <X size={size} />
    </button>
  );
}

/** Vertical tab list for the sidebar: pinned grid, then the rest. */
export function VerticalTabs({ session }: { session: Session }) {
  const ws = session.workspaces.find((w) => w.id === session.current)!;
  const tabs = workspaceTabs(session);
  const pinned = tabs.filter((t) => t.pinned);
  const rest = tabs.filter((t) => !t.pinned);
  const { menu, open, close } = useTabMenu();

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      {pinned.length > 0 && (
        <div className="grid grid-cols-4 gap-1.5 px-2">
          {pinned.map((t) => (
            <button
              key={t.id}
              title={tabTitle(t)}
              {...handlers(t, open)}
              className={cn(
                "z-press grid h-10 place-items-center rounded-[calc(var(--z-radius)-2px)] bg-panel hover:bg-panel-hover",
                ws.active === t.id && "bg-panel-active ring-1 ring-accent/50",
              )}
            >
              <Favicon tab={t} size={18} />
            </button>
          ))}
        </div>
      )}

      <button
        onClick={() => send({ type: "newTab", url: null })}
        className="z-press mx-2 flex items-center gap-2 rounded-[calc(var(--z-radius)-2px)] px-2.5 py-2 text-dim hover:bg-panel-hover hover:text-text"
      >
        <Plus size={15} /> Nueva pestaña
        <kbd className="ml-auto text-[11px] text-faint">Ctrl T</kbd>
      </button>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        <AnimatePresence initial={false}>
          {rest.map((t) => {
            const isActive = ws.active === t.id;
            const isSplit = ws.split === t.id;
            return (
              <motion.div
                key={t.id}
                layout
                initial={{ opacity: 0, x: -8 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, height: 0, marginBottom: 0 }}
                transition={{ duration: 0.18, ease: [0.23, 1, 0.32, 1] }}
                {...handlers(t, open)}
                className={cn(
                  "group relative mb-0.5 flex h-9 cursor-default items-center gap-2.5 rounded-[calc(var(--z-radius)-2px)] px-2.5",
                  isActive ? "bg-panel-active text-text" : "text-dim hover:bg-panel-hover hover:text-text",
                  t.suspended && "opacity-55",
                )}
              >
                {isActive && (
                  <motion.span layoutId="active-tab-bar" className="absolute top-2 bottom-2 left-0 w-[3px] rounded-full bg-accent" />
                )}
                <Favicon tab={t} />
                <span className="min-w-0 flex-1 truncate">{tabTitle(t)}</span>
                {isSplit && <Columns2 size={13} className="text-accent" />}
                {t.audible && <Volume2 size={13} className="text-dim" />}
                {t.suspended && <Moon size={12} className="text-faint" aria-label="Suspendida" />}
                <CloseButton tab={t} />
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>
      <TabMenu menu={menu} session={session} onClose={close} />
    </div>
  );
}

/** Horizontal tab strip for the top bar. */
export function HorizontalTabs({ session }: { session: Session }) {
  const ws = session.workspaces.find((w) => w.id === session.current)!;
  const tabs = workspaceTabs(session);
  const { menu, open, close } = useTabMenu();
  return (
    <div className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto pr-2">
      {tabs.map((t) => {
        const isActive = ws.active === t.id;
        return (
          <motion.div
            key={t.id}
            layout
            transition={{ duration: 0.18, ease: [0.23, 1, 0.32, 1] }}
            {...handlers(t, open)}
            className={cn(
              "no-drag group relative flex h-8 shrink-0 cursor-default items-center gap-2 rounded-[calc(var(--z-radius)-2px)] px-2.5",
              t.pinned ? "w-10 justify-center" : "w-[clamp(110px,14vw,220px)]",
              isActive ? "bg-panel-active text-text" : "text-dim hover:bg-panel-hover hover:text-text",
              t.suspended && "opacity-55",
            )}
            title={tabTitle(t)}
          >
            {isActive && (
              <motion.span layoutId="active-tab-line" className="absolute right-3 bottom-0 left-3 h-[2px] rounded-full bg-accent" />
            )}
            <Favicon tab={t} />
            {!t.pinned && <span className="min-w-0 flex-1 truncate">{tabTitle(t)}</span>}
            {ws.split === t.id && <Columns2 size={12} className="text-accent" />}
            {!t.pinned && <CloseButton tab={t} size={12} />}
          </motion.div>
        );
      })}
      <button
        title="Nueva pestaña (Ctrl+T)"
        onClick={() => send({ type: "newTab", url: null })}
        className="no-drag z-press grid size-8 shrink-0 place-items-center rounded-[calc(var(--z-radius)-2px)] text-dim hover:bg-panel-hover hover:text-text"
      >
        <Plus size={15} />
      </button>
      <TabMenu menu={menu} session={session} onClose={close} row />
    </div>
  );
}
