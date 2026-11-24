---
name: rareui
description: Use when Zaphire's React UI (ui/) needs a component from RareUI (rareui.com). Explains how to read its catalog first, since the site's contents could not be captured when this skill was written.
---

# RareUI in Zaphire

**Status: incomplete.** When this skill was written, rareui.com answered
every automated request with a Vercel bot-protection page (HTTP 429
"Security Checkpoint"). Its catalog, tech stack and install command could not
be read, so this file holds no component list. Do not invent one.

## Before using it

1. Open `https://rareui.com/llms.txt` or `https://rareui.com/` in a normal
   browser (or ask the user to paste the component page / install command).
2. If it publishes a shadcn registry, install the way the other libraries
   do: `npx shadcn@latest add <registry-url-or-name>` from `ui/`.
3. Update this skill with the real install command, a component list and the
   ones that fit Zaphire, following the layout of `.claude/skills/easyui`.

## Zaphire rules (same as every UI library here)

- Use the tokens from `ui/src/index.css`, not hard-coded colors.
- No always-on animation in the sidebar or top bar; heavy effects only on the
  new-tab page, paused when hidden.
- Honor `reduce_motion` / `prefers-reduced-motion`.
