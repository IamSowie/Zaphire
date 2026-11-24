import { useSyncExternalStore } from "react";
import type { Command, Event, State } from "./types";
import { createMock } from "./mock";

/** Running inside Zaphire (served from zaphire://app) or in a plain browser. */
export const inZaphire = location.protocol === "zaphire:";
/** The UI view (sidebar/top bar) vs. a page loaded in a tab (new tab, settings). */
export const isShellView = location.pathname === "/" || location.pathname === "/index.html";

let state: State | null = null;
const listeners = new Set<() => void>();
const focusListeners = new Set<() => void>();

function receive(e: Event) {
  if (e.type === "state") {
    const { type: _t, ...rest } = e;
    state = rest;
    listeners.forEach((l) => l());
  } else if (e.type === "focusAddress") {
    focusListeners.forEach((l) => l());
  }
}

declare global {
  interface Window {
    __zaphire?: { receive: (e: Event) => void };
  }
}
window.__zaphire = { receive };

const mock = inZaphire ? null : createMock(receive);

export function send(cmd: Command): void {
  if (mock) {
    mock(cmd);
    return;
  }
  void fetch("zaphire://app/api", { method: "POST", body: JSON.stringify(cmd) }).catch((err) =>
    console.error("zaphire ipc", err),
  );
}

/** Pages inside tabs do not receive pushes; they poll the latest snapshot. */
async function pollState() {
  try {
    const r = await fetch("zaphire://app/api/state");
    if (r.ok) receive({ type: "state", ...(await r.json()) });
  } catch {
    /* browser not ready yet */
  }
}

let started = false;
export function start() {
  if (started) return;
  started = true;
  if (inZaphire && !isShellView) {
    void pollState();
    setInterval(() => !document.hidden && void pollState(), 1000);
  }
  send({ type: "ready" });
}

export function useZaphire(): State | null {
  return useSyncExternalStore(
    (cb) => {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    () => state,
  );
}

export function onFocusAddress(cb: () => void): () => void {
  focusListeners.add(cb);
  return () => focusListeners.delete(cb);
}
