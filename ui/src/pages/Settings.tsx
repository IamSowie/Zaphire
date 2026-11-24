import { motion } from "motion/react";
import { Columns3, PanelLeft, PanelTop, Puzzle, ShieldCheck, Gamepad2, Palette } from "lucide-react";
import { useEffect, type ReactNode } from "react";
import { ProfileSwitch } from "../components/controls";
import { send, useZaphire } from "../lib/ipc";
import type { Settings as S } from "../lib/types";
import { cn, compact } from "../lib/util";

function Section({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) {
  return (
    <section className="z-glass rounded-[calc(var(--z-radius)+4px)] p-5">
      <h2 className="mb-4 flex items-center gap-2 text-[15px] font-bold">
        <span className="text-accent">{icon}</span>
        {title}
      </h2>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6">
      <div>
        <div className="text-[14px]">{label}</div>
        {hint && <div className="text-[12px] text-dim">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ on, onChange, label }: { on: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button
      role="switch"
      aria-checked={on}
      aria-label={label}
      onClick={() => onChange(!on)}
      className={cn("relative h-6 w-11 rounded-full transition-colors duration-200", on ? "bg-accent" : "bg-panel-active")}
    >
      <motion.span
        layout
        transition={{ type: "spring", stiffness: 600, damping: 34 }}
        className={cn("absolute top-0.5 size-5 rounded-full bg-white shadow", on ? "right-0.5" : "left-0.5")}
      />
    </button>
  );
}

const ACCENTS = ["#3b82f6", "#8b5cf6", "#06b6d4", "#22c55e", "#f59e0b", "#ef4444", "#ec4899"];
const ENGINES = [
  { name: "DuckDuckGo", url: "https://duckduckgo.com/?q={}" },
  { name: "Google", url: "https://www.google.com/search?q={}" },
  { name: "Bing", url: "https://www.bing.com/search?q={}" },
  { name: "Brave Search", url: "https://search.brave.com/search?q={}" },
  { name: "Startpage", url: "https://www.startpage.com/do/search?q={}" },
];

export default function Settings() {
  const s = useZaphire();
  useEffect(() => {
    document.title = "Ajustes";
  }, []);
  if (!s) return <main className="z-backdrop grid h-full place-items-center text-dim">Cargando ajustes…</main>;
  const set = (patch: Partial<S>) => send({ type: "updateSettings", settings: { ...s.settings, ...patch } });
  const theme = (patch: Partial<S["theme"]>) => set({ theme: { ...s.settings.theme, ...patch } });

  return (
    <main className="z-backdrop h-full overflow-y-auto select-text">
      <div className="mx-auto flex max-w-3xl flex-col gap-5 px-6 py-10">
        <header className="mb-2 flex items-center gap-3">
          <img src="/zaphire.svg" width={36} height={36} alt="" />
          <div>
            <h1 className="text-2xl font-bold">Ajustes</h1>
            <p className="text-[13px] text-dim">Los cambios se guardan al instante.</p>
          </div>
        </header>

        <Section icon={<Columns3 size={17} />} title="Diseño">
          <Row label="Pestañas" hint="Verticales en una barra lateral u horizontales arriba">
            <div className="flex gap-2">
              {(
                [
                  ["vertical", "Verticales", <PanelLeft size={15} key="v" />],
                  ["horizontal", "Horizontales", <PanelTop size={15} key="h" />],
                ] as const
              ).map(([id, label, icon]) => (
                <button
                  key={id}
                  onClick={() => set({ tab_layout: id })}
                  className={cn(
                    "z-press flex items-center gap-2 rounded-[var(--z-radius)] px-3 py-2 text-[13px]",
                    s.settings.tab_layout === id ? "bg-accent-soft text-text ring-1 ring-accent/60" : "bg-panel text-dim hover:bg-panel-hover",
                  )}
                >
                  {icon} {label}
                </button>
              ))}
            </div>
          </Row>
          {s.settings.tab_layout === "vertical" && (
            <Row label="Ancho de la barra lateral" hint={`${s.settings.sidebar_width}px`}>
              <input
                type="range"
                min={220}
                max={420}
                step={10}
                value={s.settings.sidebar_width}
                onChange={(e) => set({ sidebar_width: Number(e.target.value) })}
                className="w-48 accent-[var(--z-accent)]"
              />
            </Row>
          )}
        </Section>

        <Section icon={<Gamepad2 size={17} />} title="Rendimiento">
          <Row label="Perfil" hint="Ahorro: mínimo consumo · Normal: equilibrado · Juego: máximo rendimiento para juegos web">
            <div className="w-72">
              <ProfileSwitch value={s.settings.profile} />
            </div>
          </Row>
          {s.restartNeeded && (
            <Row label="Reinicio necesario" hint="El perfil cambia opciones de Chromium que solo se aplican al arrancar">
              <button onClick={() => send({ type: "restart" })} className="z-press rounded-[var(--z-radius)] bg-warning/20 px-3 py-2 text-[13px] text-warning">
                Reiniciar ahora
              </button>
            </Row>
          )}
        </Section>

        <Section icon={<ShieldCheck size={17} />} title="Privacidad">
          <Row label="Bloquear anuncios y rastreadores" hint={`${compact.format(s.blocked)} peticiones bloqueadas en esta sesión`}>
            <Toggle label="Bloqueador" on={s.settings.adblock} onChange={(v) => set({ adblock: v })} />
          </Row>
          <Row label="Buscador">
            <select
              value={s.settings.search_engine}
              onChange={(e) => set({ search_engine: e.target.value })}
              className="rounded-[var(--z-radius)] border border-line bg-panel px-3 py-2 text-[13px] outline-none"
            >
              {ENGINES.map((e) => (
                <option key={e.url} value={e.url} className="bg-[#141c33]">
                  {e.name}
                </option>
              ))}
            </select>
          </Row>
        </Section>

        <Section icon={<Palette size={17} />} title="Apariencia">
          <Row label="Color de acento">
            <div className="flex gap-2">
              {ACCENTS.map((c) => (
                <button
                  key={c}
                  title={c}
                  onClick={() => theme({ accent: c })}
                  style={{ background: c }}
                  className={cn("z-press size-7 rounded-full", s.settings.theme.accent === c && "ring-2 ring-white ring-offset-2 ring-offset-[#0b1020]")}
                />
              ))}
            </div>
          </Row>
          <Row label="Transparencia del cristal" hint={`${Math.round(s.settings.theme.glass * 100)}%`}>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={s.settings.theme.glass}
              onChange={(e) => theme({ glass: Number(e.target.value) })}
              className="w-48 accent-[var(--z-accent)]"
            />
          </Row>
          <Row label="Esquinas" hint={`${s.settings.theme.radius}px`}>
            <input
              type="range"
              min={0}
              max={20}
              value={s.settings.theme.radius}
              onChange={(e) => theme({ radius: Number(e.target.value) })}
              className="w-48 accent-[var(--z-accent)]"
            />
          </Row>
          <Row label="Reducir animaciones">
            <Toggle label="Reducir animaciones" on={s.settings.theme.reduce_motion} onChange={(v) => theme({ reduce_motion: v })} />
          </Row>
        </Section>

        <Section icon={<Puzzle size={17} />} title="Extensiones">
          <Row label="Extensiones de Chrome" hint="Instálalas desde la Chrome Web Store y gestiónalas en chrome://extensions">
            <div className="flex gap-2">
              <button onClick={() => send({ type: "newTab", url: "https://chromewebstore.google.com" })} className="z-press rounded-[var(--z-radius)] bg-panel px-3 py-2 text-[13px] hover:bg-panel-hover">
                Web Store
              </button>
              <button onClick={() => send({ type: "openExtensions" })} className="z-press rounded-[var(--z-radius)] bg-accent-soft px-3 py-2 text-[13px] hover:bg-accent/30">
                Gestionar
              </button>
            </div>
          </Row>
        </Section>

        <p className="text-center text-[12px] text-faint">Zaphire 0.1 · {s.platform}</p>
      </div>
    </main>
  );
}
