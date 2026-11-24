import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

import pkg from "./package.json" with { type: "json" };

export default defineConfig({
  plugins: [react(), tailwindcss()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  // Served from zaphire://app/ – absolute paths resolve against that origin.
  base: "/",
  build: {
    target: "chrome120",
    cssMinify: true,
    // Shell must stay tiny; the new-tab page is split into its own chunk.
    chunkSizeWarningLimit: 300,
  },
  test: { environment: "jsdom" },
});
