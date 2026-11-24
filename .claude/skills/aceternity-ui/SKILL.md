---
name: aceternity-ui
description: Use when building or restyling Zaphire's React UI (ui/) and an animated effect, background, card, dock, sidebar, modal, tooltip or text effect would help — especially the new-tab page and showcase surfaces. Covers how to install Aceternity UI components with the shadcn CLI, which ones fit a browser, and the performance rules Zaphire puts on them.
---

# Aceternity UI in Zaphire

Aceternity UI (https://ui.aceternity.com) is a collection of 114+ animated
components built with **React + Tailwind CSS v4 + Motion (framer-motion)**,
distributed as copy-paste source through the shadcn registry.

## Install

```bash
cd ui
npx shadcn@latest add @aceternity/<component-name>
```

The registry item JSON lives at `https://ui.aceternity.com/registry/<name>.json`.
Machine-readable catalog: `https://ui.aceternity.com/llms.txt`,
`https://ui.aceternity.com/llms-full.txt`, `https://ui.aceternity.com/api/components`.

Components land in `ui/src/components/ui/`. They are our code once added:
edit them freely, keep the file header noting the source.

## Good fits for a browser

| Where in Zaphire | Component (slug) | Why |
| --- | --- | --- |
| New-tab page background | `aurora-background`, `background-gradient-animation`, `spotlight-new`, `shooting-stars-and-stars-background` | Atmosphere behind the search box |
| New-tab search | `placeholders-and-vanish-input`, `gooey-input` | Rotating hints, nice submit |
| Workspace / quick-launch | `floating-dock` | macOS-style dock of sites or workspaces |
| Glass surfaces | `liquid-glass`, `glowing-effect`, `card-spotlight` | Glass panels with refraction / glow |
| Settings dialogs | `animated-modal` | Compound modal with motion |
| Hints | `animated-tooltip` | Tooltips on icon buttons |
| Tabs in settings | `tabs` (Animated Tabs) | Sliding background indicator |
| Loading | `loader`, `multi-step-loader` | First-run / import screens |
| Collapsible sidebar pattern | `sidebar` | Hover-expand reference for the tab rail |

## Zaphire rules (performance first)

1. **Chrome UI stays cheap.** The sidebar / top bar is always on screen:
   no canvas, WebGL, particle or infinite animations there. Heavy effects
   (`aurora`, `shooting-stars`, `sparkles`, `3d-*`, shaders) are only allowed
   on the new-tab page, and must pause when `document.hidden`.
2. Respect `settings.theme.reduce_motion` and `prefers-reduced-motion`:
   render the static fallback.
3. Use Zaphire's CSS tokens (`--z-bg`, `--z-panel`, `--z-accent`, …, see
   `ui/src/index.css`) instead of the hard-coded colors in the snippets.
4. Import `motion` from `motion/react` (already a dependency); do not add
   `framer-motion` a second time.
5. Lazy-load new-tab-only components with `React.lazy` so the shell bundle
   stays small.
