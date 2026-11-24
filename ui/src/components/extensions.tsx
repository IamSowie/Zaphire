import { Puzzle } from "lucide-react";
import { useState } from "react";
import { send } from "../lib/ipc";
import type { ExtensionInfo } from "../lib/types";
import { cn, uiZoom } from "../lib/util";

function ExtIcon({ ext }: { ext: ExtensionInfo }) {
  const [broken, setBroken] = useState(false);
  if (ext.hasIcon && !broken) {
    return (
      <img
        src={`zaphire://app/api/ext-icon?id=${encodeURIComponent(ext.id)}`}
        width={16}
        height={16}
        alt=""
        onError={() => setBroken(true)}
      />
    );
  }
  return <span className="grid size-4 place-items-center rounded bg-accent-soft text-[10px] font-bold">{ext.name[0]?.toUpperCase()}</span>;
}

/**
 * Toolbar buttons of installed extensions. Clicking opens the extension's
 * popup as a floating view, anchored to the button. `side` says where the
 * popup goes: below the button (top bar) or to its right (sidebar).
 */
export function ExtensionBar({
  extensions,
  side,
  className,
}: {
  extensions: ExtensionInfo[];
  side: "below" | "right";
  className?: string;
}) {
  const open = (ext: ExtensionInfo, el: HTMLElement) => {
    const r = el.getBoundingClientRect();
    const z = uiZoom();
    // Sidebar: open beside it, over the page, not on top of the tab list.
    const x = side === "below" ? r.left : window.innerWidth + 6;
    const y = side === "below" ? r.bottom + 6 : r.top;
    send({ type: "extensionPopup", id: ext.id, x: Math.round(x * z), y: Math.round(y * z) });
  };
  return (
    <div className={cn("no-drag flex min-w-0 items-center gap-0.5 overflow-x-auto", className)}>
      {extensions.map((ext) => (
        <button
          key={ext.id}
          title={ext.popup ? ext.name : `${ext.name} (sin ventana emergente: abre sus opciones)`}
          onClick={(e) => open(ext, e.currentTarget)}
          className="z-press grid size-8 shrink-0 place-items-center rounded-[calc(var(--z-radius)-4px)] hover:bg-panel-hover"
        >
          <ExtIcon ext={ext} />
        </button>
      ))}
      <button
        title="Gestionar extensiones"
        onClick={() => send({ type: "openExtensions" })}
        className="z-press grid size-8 shrink-0 place-items-center rounded-[calc(var(--z-radius)-4px)] text-dim hover:bg-panel-hover hover:text-text"
      >
        <Puzzle size={15} />
      </button>
    </div>
  );
}
