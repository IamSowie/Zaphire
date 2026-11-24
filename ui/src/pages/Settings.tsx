import { motion } from "motion/react";
import { Columns3, PanelLeft, PanelTop, Puzzle, ShieldCheck, Gamepad2, Palette, Moon, Sun, Monitor, History, RotateCcw } from "lucide-react";
import { useEffect, type ReactNode } from "react";
import { ProfileSwitch, Segmented } from "../components/controls";
import { send, useZaphire } from "../lib/ipc";
import type { Settings as S } from "../lib/types";
import { cn, compact, resolvedMode } from "../lib/util";

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

/** Base colours for the browser frame, per theme. */
const BACKGROUNDS: Record<"dark" | "light", { name: string; color: string }[]> = {
  dark: [
    { name: "Zafiro", color: "#0b1020" },
    { name: "Grafito", color: "#16181d" },
    { name: "Negro", color: "#000000" },
    { name: "Bosque", color: "#0c1a14" },
    { name: "Ciruela", color: "#1a1020" },
  ],
  light: [
    { name: "Niebla", color: "#eef1f7" },
    { name: "Blanco", color: "#ffffff" },
    { name: "Arena", color: "#f5f0e6" },
    { name: "Menta", color: "#eaf5ef" },
    { name: "Lavanda", color: "#f1edf8" },
  ],
};

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
                <option key={e.url} value={e.url} className="bg-menu">
                  {e.name}
                </option>
              ))}
            </select>
          </Row>
        </Section>

        <Section icon={<Palette size={17} />} title="Apariencia">
          <Row label="Tema">
            <Segmented
              label="Tema"
              value={s.settings.theme.mode}
              onChange={(mode) => theme({ mode, background_color: null })}
              options={[
                { id: "dark", label: "Oscuro", icon: <Moon size={13} /> },
                { id: "light", label: "Claro", icon: <Sun size={13} /> },
                { id: "system", label: "Sistema", icon: <Monitor size={13} /> },
              ]}
            />
          </Row>
          <Row label="Fondo" hint="Degradado suave con el color de acento, o un color plano">
            <Segmented
              label="Fondo"
              value={s.settings.theme.background}
              onChange={(background) => theme({ background })}
              options={[
                { id: "gradient", label: "Degradado" },
                { id: "solid", label: "Plano" },
              ]}
            />
          </Row>
          <Row label="Color de fondo" hint={s.settings.theme.background_color ?? "Automático según el tema"}>
            <div className="flex items-center gap-2">
              <button
                title="Automático"
                onClick={() => theme({ background_color: null })}
                className={cn(
                  "z-press grid size-7 place-items-center rounded-full border border-line text-dim",
                  !s.settings.theme.background_color && "ring-2 ring-accent",
                )}
              >
                <RotateCcw size={12} />
              </button>
              {BACKGROUNDS[resolvedMode(s.settings.theme)].map((b) => (
                <button
                  key={b.color}
                  title={b.name}
                  onClick={() => theme({ background_color: b.color })}
                  style={{ background: b.color }}
                  className={cn(
                    "z-press size-7 rounded-full border border-line",
                    s.settings.theme.background_color === b.color && "ring-2 ring-accent",
                  )}
                />
              ))}
              <label title="Color personalizado" className="z-press relative size-7 cursor-pointer overflow-hidden rounded-full border border-line bg-[conic-gradient(red,yellow,lime,cyan,blue,magenta,red)]">
                <input
                  type="color"
                  value={s.settings.theme.background_color ?? "#0b1020"}
                  onChange={(e) => theme({ background_color: e.target.value })}
                  className="absolute inset-0 cursor-pointer opacity-0"
                />
              </label>
            </div>
          </Row>
          <Row label="Color de acento">
            <div className="flex gap-2">
              {ACCENTS.map((c) => (
                <button
                  key={c}
                  title={c}
                  onClick={() => theme({ accent: c })}
                  style={{ background: c }}
                  className={cn("z-press size-7 rounded-full", s.settings.theme.accent === c && "ring-2 ring-text ring-offset-2 ring-offset-[var(--z-bg)]")}
                />
              ))}
              <label title="Color personalizado" className="z-press relative size-7 cursor-pointer overflow-hidden rounded-full border border-line bg-[conic-gradient(red,yellow,lime,cyan,blue,magenta,red)]">
                <input type="color" value={s.settings.theme.accent} onChange={(e) => theme({ accent: e.target.value })} className="absolute inset-0 cursor-pointer opacity-0" />
              </label>
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
          <Row label="Tamaño de la interfaz" hint={`${Math.round(s.settings.theme.scale * 100)}%`}>
            <input
              type="range"
              min={0.85}
              max={1.25}
              step={0.05}
              value={s.settings.theme.scale}
              onChange={(e) => theme({ scale: Number(e.target.value) })}
              className="w-48 accent-[var(--z-accent)]"
            />
          </Row>
          <Row label="Densidad">
            <Segmented
              label="Densidad"
              value={s.settings.theme.density}
              onChange={(density) => theme({ density })}
              options={[
                { id: "comfortable", label: "Cómoda" },
                { id: "compact", label: "Compacta" },
              ]}
            />
          </Row>
          <Row label="Reducir animaciones">
            <Toggle label="Reducir animaciones" on={s.settings.theme.reduce_motion} onChange={(v) => theme({ reduce_motion: v })} />
          </Row>
        </Section>

        <Section icon={<History size={17} />} title="Inicio y biblioteca">
          <Row label="Restaurar pestañas al abrir" hint="Vuelven sin cargar: solo se cargan al abrirlas, así no gastan memoria">
            <Toggle label="Restaurar pestañas" on={s.settings.restore_session} onChange={(v) => set({ restore_session: v })} />
          </Row>
          <Row label="Marcadores en la barra lateral">
            <Toggle label="Marcadores en la barra" on={s.settings.show_bookmarks} onChange={(v) => set({ show_bookmarks: v })} />
          </Row>
          <Row label="Historial y marcadores" hint="Ctrl+H">
            <button onClick={() => send({ type: "newTab", url: "zaphire://app/library" })} className="z-press rounded-[var(--z-radius)] bg-panel px-3 py-2 text-[13px] hover:bg-panel-hover">
              Abrir
            </button>
          </Row>
        </Section>

        <Section icon={<Puzzle size={17} />} title="Extensiones">
          <Row label="Botones de extensiones en la barra" hint="Al pulsarlos se abre la ventana emergente de la extensión">
            <Toggle label="Botones de extensiones" on={s.settings.show_extensions} onChange={(v) => set({ show_extensions: v })} />
          </Row>
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
