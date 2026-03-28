# Phase 02: UI Styling Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 02-ui-styling-infrastructure
**Areas discussed:** Theme design tokens, Dark mode strategy, Component wrapping depth, Global CSS foundation

---

## Theme design tokens

### Brand colors

| Option | Description | Selected |
|--------|-------------|----------|
| Blue-indigo primary | Like Linear's indigo-blue. Neutral grays for backgrounds/text. Works well for developer tools. | ✓ |
| Purple-violet primary | More distinctive, less corporate. | |
| Teal-green primary | Fresh, modern, works for productivity apps. | |

**User's choice:** Blue-indigo primary

### Gray scale

| Option | Description | Selected |
|--------|-------------|----------|
| Neutral gray | Pure neutral grays (zinc/neutral). Clean, no warm or cool bias. | ✓ |
| Warm gray | Slightly warm grays. Softer feel. | |
| Cool blue-gray | More technical feel. | |

**User's choice:** Neutral gray (zinc)

### Typography

| Option | Description | Selected |
|--------|-------------|----------|
| System fonts | Zero-config, fastest load, native to each OS. Perfect for boilerplate. | ✓ |
| Inter + JetBrains Mono | Clean, widely used, Google Fonts CDN. | |
| Geist + Geist Mono | Vercel's font, very clean. | |

**User's choice:** System fonts

---

## Dark mode strategy

### Default mode

| Option | Description | Selected |
|--------|-------------|----------|
| Light default | Light on first load, dark mode opt-in. Safest for boilerplate. | |
| System preference | Respect OS preference via prefers-color-scheme. Modern approach. | ✓ |
| Dark default | Dark on first load. Unusual for boilerplates. | |

**User's choice:** System preference

### Toggle UX

| Option | Description | Selected |
|--------|-------------|----------|
| Toggle in header | Sun/moon icon in header. Standard pattern. | |
| Settings page only | Settings page only, not in header. Less cluttered header. | ✓ |
| No manual toggle | System preference only, no manual override. | |

**User's choice:** Settings page only

---

## Component wrapping depth

### Wrapping approach

| Option | Description | Selected |
|--------|-------------|----------|
| Full wrappers | Button.svelte, Input.svelte, etc. — apply theme styling, consistent props. Recommended for boilerplates. | ✓ |
| Selective wrapping | Only wrap where bitsUI API doesn't match needs. | |
| Direct bitsUI | Import bitsUI directly everywhere. Less code, styling scattered. | |

**User's choice:** Full wrappers

### Scope

| Option | Description | Selected |
|--------|-------------|----------|
| 4 core components | Button, Input, Dialog, Select — the four in success criteria. | |
| Extended set (8-10) | Also add: Card, Badge, Switch, Toast, Dropdown. More complete. | ✓ |
| Minimal (2) | Only Button and Input. | |

**User's choice:** Extended set (8-10)

---

## Global CSS foundation

### CSS reset

| Option | Description | Selected |
|--------|-------------|----------|
| Tailwind preflight only | TailwindCSS v4 includes preflight (modern-normalize) by default. | ✓ |
| Additional custom reset | Add a dedicated CSS reset on top of Tailwind. | |

**User's choice:** Tailwind preflight only

### Spacing

| Option | Description | Selected |
|--------|-------------|----------|
| 4px base | Standard 0.25rem increments. Tailwind default. | ✓ |
| 8px base | More generous spacing for a clean feel. | |
| Custom scale | Custom scale with specific gaps. | |

**User's choice:** 4px base (Tailwind default)

### Breakpoints

| Option | Description | Selected |
|--------|-------------|----------|
| Tailwind defaults | sm:640, md:768, lg:1024, xl:1280, 2xl:1536. | |
| Tauri-optimized | Custom breakpoints for Tauri window sizes: 375 mobile, 800 tablet, 1280 desktop. | ✓ |

**User's choice:** Tauri-optimized breakpoints

---

## Design Direction

**User selected:** Clean minimal (like Linear/Vercel) — muted palette, generous whitespace, subtle borders. No heavy shadows or gradients.

---

## Agent's Discretion

- Exact CSS custom property naming — agent decides
- File organization within $lib/components/ — agent decides
- Dark mode implementation method — agent picks best approach

---

## Deferred Ideas

None — discussion stayed within phase scope.
