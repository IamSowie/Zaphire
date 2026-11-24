import { StrictMode, Suspense, lazy, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { SidebarShell, TopBarShell } from "./components/Shell";
import { start, useZaphire } from "./lib/ipc";
import { applyTheme } from "./lib/util";
import "./index.css";

// Pages that load inside tabs are split out so the shell bundle stays small.
const NewTab = lazy(() => import("./pages/NewTab"));
const Settings = lazy(() => import("./pages/Settings"));

function ShellView() {
  const s = useZaphire();
  if (!s) return <div className="z-backdrop h-full" />;
  return s.settings.tab_layout === "vertical" ? <SidebarShell s={s} /> : <TopBarShell s={s} />;
}

function App() {
  const s = useZaphire();
  useEffect(() => {
    if (s) applyTheme(s.settings);
  }, [s]);
  const path = location.pathname.replace(/\/$/, "") || "/";
  const page = path === "/newtab" ? <NewTab /> : path === "/settings" ? <Settings /> : <ShellView />;
  return <Suspense fallback={<div className="z-backdrop h-full" />}>{page}</Suspense>;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
start();
