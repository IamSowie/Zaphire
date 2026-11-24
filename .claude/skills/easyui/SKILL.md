---
name: easyui
description: Use when Zaphire's React UI (ui/) needs an animated, production-style component — spotlight search / command palette, glass navbar, sidebar with hover pill, animated tabs, toggles, switches, dropdowns, toasts, tooltips, floating dock, metric HUD. Covers installing EasyUI components through the shadcn CLI and which of its 138 components suit a browser.
---

# EasyUI in Zaphire

EasyUI (https://easyui.site) — 138 open-source animated components built with
**React + Tailwind CSS + Framer Motion**, copy-paste ownership through the
shadcn CLI. Catalog for agents: `https://easyui.site/llms.txt`; shadcn
registry index: `https://easyui.site/registry.json`.

## Install

Source files live in the GitHub repo `Surajmaurya1/easyui`:

```bash
cd ui
npx shadcn@latest add Surajmaurya1/easyui/<component-name>
# e.g.
npx shadcn@latest add Surajmaurya1/easyui/spotlight-search
```

Most items depend on `framer-motion` and `lucide-react` and bring
`lib/utils.ts`, `lib/motion-tokens.ts`. In Zaphire, rewrite
`import { motion } from "framer-motion"` to `motion/react` (same API, one
copy in the bundle).

## Components that fit a browser

| Zaphire surface | EasyUI component |
| --- | --- |
| Ctrl+K palette / omnibox overlay | `spotlight-search`, `command-menu`, `expandable-search`, `ios-search-bar` |
| Vertical tab rail | `sidebar` (hover pill follows the cursor) |
| Horizontal tab strip / segmented controls | `animated-tabs`, `pill-navigation` |
| Top bar in horizontal mode | `glass-navbar` |
| Settings toggles | `liquid-toggle`, `stretch-switch`, `draw-checkbox` |
| Menus | `origin-dropdown`, `spring-select`, `split-button`, `gooey-menu` |
| Notifications | `velocity-toast`, `undo-toast` ("tab closed — undo"), `notification-stack` |
| Hints | `directional-tooltip`, `peek-card` (tab hover preview) |
| Game / perf overlay | `metric-hud`, `animated-number`, `live-line-chart` |
| Quick launch | `small-floating-dock`, `floating-action-dock` |
| Modals | `settle-modal`, `morphing-dialog`, `sheet` |
| Loading | `skeleton`, `loader` |

## Zaphire rules

- Shell surfaces (sidebar, top bar) must not run continuous animations;
  hover/press transitions only. Canvas backgrounds (`dot-shader`,
  `glyph-matrix`, `speed-warp`, `shooting-stars`) are new-tab-only and pause
  when hidden.
- Swap hard-coded colors for Zaphire tokens in `ui/src/index.css`.
- Respect `reduce_motion`.
