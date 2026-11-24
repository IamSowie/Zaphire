import { motion } from "motion/react";
import { ArrowLeft, ArrowRight, Gauge, Leaf, Gamepad2, Minus, RotateCw, Square, X, Lock, Search, Globe, Columns2, Star } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { onFocusAddress, send } from "../lib/ipc";
import type { Profile, Tab } from "../lib/types";
import { cn, displayUrl } from "../lib/util";

export function IconButton({
  children,
  onClick,
  title,
  disabled,
  active,
  className,
}: {
  children: ReactNode;
  onClick?: () => void;
  title: string;
  disabled?: boolean;
  active?: boolean;
  className?: string;
}) {
  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "no-drag z-press grid size-8 place-items-center rounded-[calc(var(--z-radius)-4px)] text-dim",
        "hover:bg-panel-hover hover:text-text disabled:pointer-events-none disabled:opacity-35",
        active && "bg-accent-soft text-text",
        className,
      )}
    >
      {children}
    </button>
  );
}

export function WindowControls({ className }: { className?: string }) {
  return (
    <div className={cn("no-drag flex items-center gap-0.5", className)}>
      <IconButton title="Minimizar" onClick={() => send({ type: "minimize" })}>
        <Minus size={15} />
      </IconButton>
      <IconButton title="Maximizar" onClick={() => send({ type: "toggleMaximize" })}>
        <Square size={12} />
      </IconButton>
      <IconButton title="Cerrar" onClick={() => send({ type: "closeWindow" })} className="hover:bg-danger/80 hover:text-white">
        <X size={15} />
      </IconButton>
    </div>
  );
}

export function NavButtons({ tab, split, onSplit }: { tab: Tab | null; split: boolean; onSplit: () => void }) {
  return (
    <div className="flex items-center gap-0.5">
      <IconButton title="Atrás (Alt+←)" disabled={!tab?.canGoBack} onClick={() => send({ type: "back" })}>
        <ArrowLeft size={16} />
      </IconButton>
      <IconButton title="Adelante (Alt+→)" disabled={!tab?.canGoForward} onClick={() => send({ type: "forward" })}>
        <ArrowRight size={16} />
      </IconButton>
      <IconButton
        title={tab?.loading ? "Detener" : "Recargar (Ctrl+R)"}
        onClick={() => send({ type: tab?.loading ? "stop" : "reload" })}
      >
        {tab?.loading ? <X size={16} /> : <RotateCw size={15} />}
      </IconButton>
      <IconButton title="Vista dividida (Ctrl+Shift+S)" active={split} onClick={onSplit}>
        <Columns2 size={15} />
      </IconButton>
    </div>
  );
}

export function AddressBar({ tab, className, bookmarked }: { tab: Tab | null; className?: string; bookmarked?: boolean }) {
  const [focused, setFocused] = useState(false);
  const [value, setValue] = useState("");
  const input = useRef<HTMLInputElement>(null);
  const url = tab?.url ?? "";

  useEffect(() => {
    if (!focused) setValue(url.startsWith("zaphire://app/newtab") ? "" : url);
  }, [url, focused]);

  useEffect(
    () =>
      onFocusAddress(() => {
        input.current?.focus();
        input.current?.select();
      }),
    [],
  );

  const secure = url.startsWith("https://");
  const internal = url.startsWith("zaphire://") || url.startsWith("chrome://");

  return (
    <form
      className={cn(
        "no-drag group flex h-9 items-center gap-2 rounded-[var(--z-radius)] px-3 transition-colors duration-150",
        focused ? "bg-panel-active ring-1 ring-accent/60" : "bg-panel hover:bg-panel-hover",
        className,
      )}
      onSubmit={(e) => {
        e.preventDefault();
        if (!value.trim()) return;
        send({ type: "navigate", id: null, input: value });
        input.current?.blur();
      }}
    >
      <span className={cn("shrink-0", secure ? "text-success/80" : "text-faint")}>
        {focused ? <Search size={14} /> : secure ? <Lock size={13} /> : internal ? <Globe size={13} /> : <Globe size={13} />}
      </span>
      <input
        ref={input}
        value={focused ? value : displayUrl(url) || ""}
        placeholder="Busca o escribe una dirección"
        spellCheck={false}
        className="min-w-0 flex-1 bg-transparent text-[13px] text-text outline-none placeholder:text-faint"
        onFocus={(e) => {
          setFocused(true);
          setValue(url.startsWith("zaphire://app/newtab") ? "" : url);
          requestAnimationFrame(() => e.target.select());
        }}
        onBlur={() => setFocused(false)}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            setValue(url);
            input.current?.blur();
          }
        }}
      />
      {!focused && tab && !internal && (
        <button
          type="button"
          title={bookmarked ? "Quitar de marcadores (Ctrl+D)" : "Añadir a marcadores (Ctrl+D)"}
          onClick={() => send({ type: "toggleBookmark" })}
          className={cn("z-press shrink-0 rounded-md p-0.5", bookmarked ? "text-warning" : "text-faint opacity-0 group-hover:opacity-100 hover:text-text")}
        >
          <Star size={14} fill={bookmarked ? "currentColor" : "none"} />
        </button>
      )}
    </form>
  );
}

const PROFILES: { id: Profile; label: string; icon: ReactNode; hint: string }[] = [
  { id: "saver", label: "Ahorro", icon: <Leaf size={13} />, hint: "Mínimo consumo: suspende pestañas a los 5 min y limita procesos" },
  { id: "balanced", label: "Normal", icon: <Gauge size={13} />, hint: "Equilibrado: suspende pestañas tras 30 min" },
  { id: "game", label: "Juego", icon: <Gamepad2 size={13} />, hint: "Máximo rendimiento: sin límite de FPS, GPU forzada, resto de pestañas congeladas (Ctrl+Shift+G)" },
];

/** Segmented Saver / Balanced / Game switch with a sliding pill. */
export function ProfileSwitch({ value, compact }: { value: Profile; compact?: boolean }) {
  return (
    <div role="radiogroup" aria-label="Perfil de rendimiento" className="no-drag relative flex rounded-[var(--z-radius)] bg-panel p-0.5">
      {PROFILES.map((p) => (
        <button
          key={p.id}
          role="radio"
          aria-checked={value === p.id}
          title={p.hint}
          onClick={() => send({ type: "setProfile", profile: p.id })}
          className={cn(
            "z-press relative flex flex-1 items-center justify-center gap-1.5 rounded-[calc(var(--z-radius)-3px)] px-2 py-1.5 text-[12px]",
            value === p.id ? "text-text" : "text-dim hover:text-text",
          )}
        >
          {value === p.id && (
            <motion.span
              layoutId="profile-pill"
              transition={{ type: "spring", stiffness: 500, damping: 38 }}
              className={cn(
                "absolute inset-0 rounded-[calc(var(--z-radius)-3px)]",
                p.id === "game" ? "bg-gradient-to-r from-fuchsia-500/35 to-accent/40" : p.id === "saver" ? "bg-success/20" : "bg-panel-active",
              )}
            />
          )}
          <span className="relative">{p.icon}</span>
          {!compact && <span className="relative">{p.label}</span>}
        </button>
      ))}
    </div>
  );
}

export function Favicon({ tab, size = 16 }: { tab: Tab; size?: number }) {
  const [broken, setBroken] = useState(false);
  if (tab.loading) {
    return (
      <span
        style={{ width: size, height: size }}
        className="inline-block shrink-0 animate-spin rounded-full border-2 border-accent/30 border-t-accent"
      />
    );
  }
  if (tab.favicon && !broken) {
    return <img src={tab.favicon} width={size} height={size} className="shrink-0 rounded-[3px]" onError={() => setBroken(true)} alt="" />;
  }
  const letter = (tab.title || displayUrl(tab.url) || "Z").trim()[0]?.toUpperCase() ?? "Z";
  return (
    <span
      style={{ width: size, height: size, fontSize: size * 0.62 }}
      className="grid shrink-0 place-items-center rounded-[4px] bg-accent-soft font-bold text-text"
    >
      {letter}
    </span>
  );
}

/** Generic segmented control (theme, background, density…). */
export function Segmented<T extends string>({
  value,
  options,
  onChange,
  label,
}: {
  value: T;
  options: { id: T; label: string; icon?: ReactNode }[];
  onChange: (v: T) => void;
  label: string;
}) {
  return (
    <div role="radiogroup" aria-label={label} className="no-drag flex rounded-[var(--z-radius)] bg-panel p-0.5">
      {options.map((o) => (
        <button
          key={o.id}
          role="radio"
          aria-checked={value === o.id}
          onClick={() => onChange(o.id)}
          className={cn(
            "z-press flex items-center gap-1.5 rounded-[calc(var(--z-radius)-3px)] px-3 py-1.5 text-[12px]",
            value === o.id ? "bg-panel-active text-text shadow-sm" : "text-dim hover:text-text",
          )}
        >
          {o.icon}
          {o.label}
        </button>
      ))}
    </div>
  );
}
