import { motion } from "motion/react";
import { Search } from "lucide-react";
import { useEffect, useState } from "react";
import { send, useZaphire } from "../lib/ipc";
import { compact } from "../lib/util";

const QUICK = [
  { name: "YouTube", url: "https://www.youtube.com", color: "#ef4444" },
  { name: "GitHub", url: "https://github.com", color: "#a3a3a3" },
  { name: "Wikipedia", url: "https://es.wikipedia.org", color: "#e5e5e5" },
  { name: "Reddit", url: "https://www.reddit.com", color: "#f97316" },
  { name: "Twitch", url: "https://www.twitch.tv", color: "#a855f7" },
  { name: "Chrome Web Store", url: "https://chromewebstore.google.com", color: "#22c55e" },
];

const HINTS = ["Busca en la web…", "Escribe una dirección…", "Prueba “juegos webgl”…", "Ctrl+Shift+G activa el Modo Juego"];

function Clock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const t = setInterval(() => setNow(new Date()), 15_000);
    return () => clearInterval(t);
  }, []);
  return (
    <div className="text-center">
      <div className="text-6xl font-bold tracking-tight tabular-nums">
        {now.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })}
      </div>
      <div className="mt-1 text-[14px] text-dim first-letter:uppercase">{now.toLocaleDateString("es", { weekday: "long", day: "numeric", month: "long" })}</div>
    </div>
  );
}

export default function NewTab() {
  const s = useZaphire();
  const [q, setQ] = useState("");
  const [hint, setHint] = useState(0);
  useEffect(() => {
    document.title = "Nueva pestaña";
    const t = setInterval(() => !document.hidden && setHint((h) => (h + 1) % HINTS.length), 3500);
    return () => clearInterval(t);
  }, []);

  return (
    <main className="z-backdrop relative flex h-full flex-col items-center justify-center gap-10 overflow-hidden px-6 select-text">
      {/* Soft aurora: CSS only, and only on this page, never in the shell. */}
      <div aria-hidden className="pointer-events-none absolute inset-0 opacity-60 [mask-image:radial-gradient(60%_50%_at_50%_40%,black,transparent)]">
        <div className="absolute -inset-[20%] animate-[spin_60s_linear_infinite] bg-[conic-gradient(from_90deg,transparent,var(--z-accent),#8b5cf6,transparent_60%)] blur-3xl" />
      </div>

      <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3, ease: [0.23, 1, 0.32, 1] }} className="relative">
        <Clock />
      </motion.div>

      <motion.form
        initial={{ opacity: 0, y: 8, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.3, delay: 0.05, ease: [0.23, 1, 0.32, 1] }}
        className="z-glass relative flex h-14 w-full max-w-xl items-center gap-3 rounded-2xl px-5 focus-within:ring-2 focus-within:ring-accent/60"
        onSubmit={(e) => {
          e.preventDefault();
          if (q.trim()) send({ type: "navigate", id: null, input: q });
        }}
      >
        <Search size={18} className="text-dim" />
        <input
          autoFocus
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder={HINTS[hint]}
          className="flex-1 bg-transparent text-[16px] outline-none placeholder:text-faint"
        />
      </motion.form>

      <div className="relative grid w-full max-w-xl grid-cols-3 gap-3 sm:grid-cols-6">
        {QUICK.map((site, i) => (
          <motion.button
            key={site.url}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.25, delay: 0.08 + i * 0.03 }}
            onClick={() => send({ type: "navigate", id: null, input: site.url })}
            className="z-press group flex flex-col items-center gap-2 rounded-xl p-2 hover:bg-panel-hover"
          >
            <span className="z-glass grid size-12 place-items-center rounded-xl text-lg font-bold" style={{ color: site.color }}>
              {site.name[0]}
            </span>
            <span className="max-w-full truncate text-[12px] text-dim group-hover:text-text">{site.name}</span>
          </motion.button>
        ))}
      </div>

      {s && s.settings.adblock && (
        <p className="relative text-[12px] text-faint tabular-nums">
          Zaphire ha bloqueado {compact.format(s.blocked)} anuncios y rastreadores en esta sesión
        </p>
      )}
    </main>
  );
}
