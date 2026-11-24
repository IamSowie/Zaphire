import { ChevronRight, Star } from "lucide-react";
import { useState } from "react";
import { send } from "../lib/ipc";
import type { Bookmark } from "../lib/types";
import { cn, displayUrl } from "../lib/util";

function Fav({ url }: { url: string }) {
  let host = "";
  try {
    host = new URL(url).host.replace(/^www\./, "");
  } catch {
    /* not a URL */
  }
  return <span className="grid size-4 shrink-0 place-items-center rounded-[4px] bg-accent-soft text-[10px] font-bold">{(host[0] ?? "★").toUpperCase()}</span>;
}

/** Collapsible bookmarks list for the sidebar. */
export function SidebarBookmarks({ bookmarks }: { bookmarks: Bookmark[] }) {
  const [open, setOpen] = useState(true);
  if (bookmarks.length === 0) return null;
  return (
    <div className="px-2">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center gap-1.5 px-2 py-1 text-[11px] font-bold tracking-wide text-faint uppercase hover:text-dim"
      >
        <ChevronRight size={12} className={cn("transition-transform", open && "rotate-90")} />
        <Star size={11} /> Marcadores
      </button>
      {open && (
        <div className="max-h-48 overflow-y-auto">
          {bookmarks.map((b) => (
            <button
              key={b.url}
              title={`${b.title}\n${b.url}`}
              onClick={() => send({ type: "navigate", id: null, input: b.url })}
              onAuxClick={(e) => e.button === 1 && send({ type: "newTab", url: b.url })}
              className="flex h-8 w-full items-center gap-2.5 rounded-[calc(var(--z-radius)-2px)] px-2.5 text-left text-dim hover:bg-panel-hover hover:text-text"
            >
              <Fav url={b.url} />
              <span className="min-w-0 flex-1 truncate">{b.title || displayUrl(b.url)}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
