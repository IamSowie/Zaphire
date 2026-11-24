---
name: pinui
description: Use when Zaphire's React UI (ui/) wants a distinctive, playful component — theme switch, liquid tabs, chips, selection list, session list, search card, magnify, thermal dial, joystick — from Pin UI, the Pinterest-inspired React library. Covers its shadcn registry and which components suit a browser.
---

# Pin UI in Zaphire

Pin UI (https://www.pinui.xyz — the apex `pinui.xyz` redirects there) is a
free, open-source React component library that turns Pinterest-inspired
designs into copy-paste components. It also ships "blocx" page sections.

## Install

Registry items are at `https://www.pinui.xyz/r/<name>.json`:

```bash
cd ui
npx shadcn@latest add https://www.pinui.xyz/r/theme-switch.json
```

Files land under `components/pinui/`. Some components ship their own CSS file
and an "engine" module (e.g. `ThemeSwitch.tsx` + `theme-switch-engine`).

## Components

add-member · balance-card · calendar-card · cart-card · chat-room · chips ·
copy-paste-pad · desk-computer · dynamic-drop-zone · file-box · joystick ·
lamp-switch · liquid-tabs · liquid-voice · magnify · otp-input ·
presence-cards · progress-image · prompt-composer · reader-card · recorder ·
search-card · selection-list · session-list · sling-slider · swap-card ·
task-card · taxi · theme-switch · thermal-dial · weather

## Fits for Zaphire

| Surface | Component |
| --- | --- |
| Settings → appearance | `theme-switch`, `lamp-switch` |
| Settings sections / workspace picker | `liquid-tabs`, `chips`, `selection-list` |
| Sessions & history ("restore last session") | `session-list` |
| Downloads | `file-box`, `dynamic-drop-zone` |
| New-tab widgets | `search-card`, `weather`, `calendar-card` |
| Performance profile dial (Saver ↔ Game) | `thermal-dial`, `sling-slider` |
| Reader mode | `reader-card` |

## Zaphire rules

Same as the other UI skills: tokens from `ui/src/index.css`, no always-on
animation in the shell, honor `reduce_motion`, lazy-load new-tab widgets.
