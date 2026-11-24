---
name: kobra-ui
description: Use when adding a functional control to Zaphire's React UI (ui/) — command menu, toast, tabs, sidebar, dialog, sheet, input, switch, select, slider, tooltip — and you want a quiet, accessible shadcn-style component. Covers the Kobra registry, which components are free, and Kobra's house-style rules that Zaphire follows.
---

# Kobra in Zaphire

Kobra (https://kobra.systems) is a React component library for **shadcn**,
installed through the shadcn CLI. 91 components; **11 are free**, the rest
need a paid `KOBRA_TOKEN`. Agent docs: `https://kobra.systems/llms.txt`
(also `/install.md`, `/design.md`).

## Install

Add the registry to `ui/components.json`:

```json
"registries": {
  "@kobra": {
    "url": "https://kobra.systems/r/{name}.json",
    "headers": { "Authorization": "Bearer ${KOBRA_TOKEN}" }
  }
}
```

```bash
cd ui
npx shadcn@latest add @kobra/command-menu
```

Never commit a token. Without one, use only the free components.

## Free components (no account)

`@kobra/command-menu` (nested command palette, kmenu) · `@kobra/toast`
(stacked notifications) · `@kobra/navigation-menu` · `@kobra/morphing-icons`
· `@kobra/carousel` · `@kobra/input-otp` · `@kobra/video` · `@kobra/sound`
· `@kobra/conversation` · `@kobra/ai-editor` · `@kobra/halftone-dots`

For Zaphire the useful free ones are **command-menu** (Ctrl+K palette:
open tab, switch workspace, toggle Game mode), **toast** (download done,
"restart to apply profile") and **morphing-icons** (reload ⇄ stop).

Paid but relevant if the project buys a license: `sidebar`, `tabs`,
`dialog`, `sheet`, `switch`, `slider`, `select`, `combobox`, `tooltip`,
`resizable` (split view handle), `kbd`, `skeleton`, `empty`.

## Kobra house style (Zaphire follows it for controls)

- Restyle through variants/props, not `className` overrides.
- Theme tokens only (`bg-muted`, `text-foreground`, Zaphire `--z-*`), never
  raw palette colors.
- Two font weights (400 / 700). Body 14px, secondary 13px.
- One radius family; nested radius = parent radius − padding.
- One hairline border (~10% opacity), depth from shadow.
- Motion: entrances 150–250 ms `cubic-bezier(0.23,1,0.32,1)`, exits shorter
  `cubic-bezier(0.4,0,0.6,1)`; animate transform/opacity/filter only;
  honor `motion-reduce`.
- Pressed controls scale to ~0.99; focus is one ring
  (`focus-visible:ring-3 ring-ring/50`).
- Changing numbers (blocked-ads counter, RAM) use `tabular-nums`.
- Empty and loading states are designed (skeletons, helpful empty states).
