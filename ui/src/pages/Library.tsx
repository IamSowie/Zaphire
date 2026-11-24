import { motion } from "motion/react";
import { History, Search, Star, Trash2, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Segmented } from "../components/controls";
import { inZaphire, send, useZaphire } from "../lib/ipc";
import type { Visit } from "../lib/types";
import { cn, displayUrl } from "../lib/util";

async function fetchHistory(q: string): Promise<Visit[]> {
  if (!inZaphire) {
    const all = (globalThis as { __zaphireMockHistory?: Visit[] }).__zaphireMockHistory ?? [];
    const ql = q.toLowerCase();
    return all.filter((v) => !ql || v.url.toLowerCase().includes(ql) || v.title.toLowerCase().includes(ql));
  }
  const r = await fetch(`zaphire://app/api/history?q=${encodeURIComponent(q)}`);
  return r.ok ? r.json() : [];
}

function dayLabel(ts: number): string {
  const d = new Date(ts * 1000);
  const today = new Date();
  const yesterday = new Date(Date.now() - 86_400_000);
  if (d.toDateString() === today.toDateString()) return "Hoy";
  if (d.toDateString() === yesterday.toDateString()) return "Ayer";
  return d.toLocaleDateString("es", { weekday: "long", day: "numeric", month: "long" });
}

function Row({ url, title, meta, onRemove }: { url: string; title: string; meta?: string; onRemove: () => void }) {
  return (
    <div className="group flex items-center gap-3 rounded-[var(--z-radius)] px-3 py-2 hover:bg-panel-hover">
      {meta && <span className="w-12 shrink-0 text-[12px] text-faint tabular-nums">{meta}</span>}
      <button
        className="flex min-w-0 flex-1 flex-col text-left"
        onClick={() => send({ type: "navigate", id: null, input: url })}
        onAuxClick={(e) => e.button === 1 && send({ type: "newTab", url })}
      >
        <span className="truncate text-[14px]">{title || displayUrl(url)}</span>
        <span className="truncate text-[12px] text-faint">{displayUrl(url)}</span>
      </button>
      <button
        title="Quitar"
        onClick={onRemove}
        className="grid size-7 place-items-center rounded-md text-faint opacity-0 group-hover:opacity-100 hover:bg-panel-active hover:text-text"
      >
        <X size={14} />
      </button>
    </div>
  );
}

export default function Library() {
  const s = useZaphire();
  const [tab, setTab] = useState<"history" | "bookmarks">(location.hash === "#bookmarks" ? "bookmarks" : "history");
  const [q, setQ] = useState("");
  const [visits, setVisits] = useState<Visit[]>([]);
  const [reload, setReload] = useState(0);

  useEffect(() => {
    document.title = "Historial y marcadores";
  }, []);
  useEffect(() => {
    let live = true;
    const t = setTimeout(() => fetchHistory(q).then((v) => live && setVisits(v)), 120);
    return () => {
      live = false;
      clearTimeout(t);
    };
  }, [q, reload]);

  const groups = useMemo(() => {
    const out: { day: string; items: Visit[] }[] = [];
    for (const v of visits) {
      const day = dayLabel(v.time);
      if (out.at(-1)?.day !== day) out.push({ day, items: [] });
      out.at(-1)!.items.push(v);
    }
    return out;
  }, [visits]);

  const bookmarks = (s?.bookmarks ?? []).filter(
    (b) => !q || b.url.toLowerCase().includes(q.toLowerCase()) || b.title.toLowerCase().includes(q.toLowerCase()),
  );

  return (
    <main className="z-backdrop h-full overflow-y-auto select-text">
      <div className="mx-auto flex max-w-3xl flex-col gap-5 px-6 py-10">
        <header className="flex flex-wrap items-center justify-between gap-4">
          <h1 className="text-2xl font-bold">{tab === "history" ? "Historial" : "Marcadores"}</h1>
          <Segmented
            label="Sección"
            value={tab}
            onChange={setTab}
            options={[
              { id: "history", label: "Historial", icon: <History size={13} /> },
              { id: "bookmarks", label: "Marcadores", icon: <Star size={13} /> },
            ]}
          />
        </header>

        <div className="z-glass flex h-11 items-center gap-3 rounded-[var(--z-radius)] px-4 focus-within:ring-2 focus-within:ring-accent/50">
          <Search size={16} className="text-dim" />
          <input
            autoFocus
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder={tab === "history" ? "Buscar en el historial" : "Buscar en marcadores"}
            className="flex-1 bg-transparent text-[14px] outline-none placeholder:text-faint"
          />
        </div>

        {tab === "history" ? (
          <>
            {visits.length > 0 && (
              <button
                onClick={() => {
                  if (confirm("¿Borrar todo el historial?")) {
                    send({ type: "clearHistory" });
                    setTimeout(() => setReload((r) => r + 1), 200);
                  }
                }}
                className="z-press flex items-center gap-2 self-end rounded-[var(--z-radius)] px-3 py-1.5 text-[13px] text-danger hover:bg-danger/10"
              >
                <Trash2 size={14} /> Borrar historial
              </button>
            )}
            {groups.length === 0 && <p className="py-16 text-center text-dim">{q ? "Sin resultados" : "Aún no hay historial"}</p>}
            {groups.map((g) => (
              <motion.section key={g.day} initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} className="z-glass rounded-[calc(var(--z-radius)+4px)] p-2">
                <h2 className="px-3 pt-2 pb-1 text-[12px] font-bold text-dim first-letter:uppercase">{g.day}</h2>
                {g.items.map((v) => (
                  <Row
                    key={`${v.url}-${v.time}`}
                    url={v.url}
                    title={v.title}
                    meta={new Date(v.time * 1000).toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })}
                    onRemove={() => {
                      send({ type: "deleteHistory", url: v.url });
                      setVisits((vs) => vs.filter((x) => x.url !== v.url));
                    }}
                  />
                ))}
              </motion.section>
            ))}
          </>
        ) : (
          <section className={cn("z-glass rounded-[calc(var(--z-radius)+4px)] p-2", bookmarks.length === 0 && "py-16")}>
            {bookmarks.length === 0 && (
              <p className="text-center text-dim">{q ? "Sin resultados" : "Pulsa la estrella de la barra de direcciones o Ctrl+D para guardar una página"}</p>
            )}
            {bookmarks.map((b) => (
              <Row key={b.url} url={b.url} title={b.title} onRemove={() => send({ type: "removeBookmark", url: b.url })} />
            ))}
          </section>
        )}
      </div>
    </main>
  );
}
