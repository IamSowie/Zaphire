import { motion } from "motion/react";
import { Plus } from "lucide-react";
import { useState } from "react";
import { send } from "../lib/ipc";
import type { Session } from "../lib/types";
import { cn } from "../lib/util";

const PRESETS = [
  { icon: "💼", color: "#22c55e", name: "Trabajo" },
  { icon: "🎮", color: "#d946ef", name: "Juegos" },
  { icon: "📚", color: "#f59e0b", name: "Estudio" },
  { icon: "🎵", color: "#06b6d4", name: "Ocio" },
];

/** Workspace switcher: each workspace keeps its own tabs. */
export function Workspaces({ session, className, popupSide = "top" }: { session: Session; className?: string; popupSide?: "top" | "left" }) {
  const [adding, setAdding] = useState(false);
  return (
    <div className={cn("no-drag flex items-center gap-1", className)}>
      {session.workspaces.map((w) => {
        const active = w.id === session.current;
        return (
          <button
            key={w.id}
            title={`${w.name} · ${w.tabs.length} pestañas`}
            onClick={() => send({ type: "switchWorkspace", id: w.id })}
            onContextMenu={(e) => {
              e.preventDefault();
              if (session.workspaces.length > 1 && confirm(`¿Cerrar el espacio "${w.name}" y sus pestañas?`))
                send({ type: "removeWorkspace", id: w.id });
            }}
            className={cn("z-press relative grid size-8 place-items-center rounded-full text-[14px]", !active && "opacity-60 hover:opacity-100")}
          >
            {active && (
              <motion.span
                layoutId="ws-ring"
                className="absolute inset-0 rounded-full"
                style={{ boxShadow: `0 0 0 2px ${w.color}`, background: `${w.color}22` }}
                transition={{ type: "spring", stiffness: 500, damping: 36 }}
              />
            )}
            <span className="relative">{w.icon}</span>
          </button>
        );
      })}
      <div className="relative">
        <button
          title="Nuevo espacio de trabajo"
          onClick={() => setAdding((v) => !v)}
          className="z-press grid size-8 place-items-center rounded-full text-dim hover:bg-panel-hover hover:text-text"
        >
          <Plus size={14} />
        </button>
        {adding && (
          <div className={cn("z-glass absolute z-50 flex gap-1 rounded-[var(--z-radius)] bg-menu p-1", popupSide === "top" ? "bottom-10 left-0" : "top-0 right-10")}>
            {PRESETS.map((p) => (
              <button
                key={p.name}
                title={p.name}
                onClick={() => {
                  send({ type: "newWorkspace", ...p });
                  setAdding(false);
                }}
                className="z-press grid size-8 place-items-center rounded-full hover:bg-panel-hover"
              >
                {p.icon}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
