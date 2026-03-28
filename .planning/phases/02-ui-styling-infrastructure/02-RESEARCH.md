# Phase 02 Research: UI Styling Infrastructure

**Researched:** 2026-03-28
**Confidence:** HIGH

## Research Questions

1. How to configure TailwindCSS v4 with @theme blocks and custom tokens?
2. How to integrate bits-ui v2.16.4 components with wrapper components?
3. How to implement dark mode in SvelteKit SPA with Tailwind v4?
4. What component wrapper pattern to use for theme consistency?

## Findings

### TailwindCSS v4 CSS-First Configuration

**Source:** TailwindCSS v4 docs, project tailwindcss skill

Tailwind v4 uses `@import "tailwindcss"` and `@theme` blocks in CSS. No `tailwind.config.js` needed.

**Key syntax:**
```css
@import "tailwindcss";

@theme {
  --color-primary-50: oklch(97% 0.02 260);
  --color-primary-500: oklch(55% 0.2 260);
  --color-primary-900: oklch(25% 0.15 260);
  --font-family-sans: system-ui, -apple-system, sans-serif;
  --breakpoint-sm: 375px;
  --breakpoint-md: 800px;
  --breakpoint-lg: 1280px;
}
```

**Integration with Vite:** `@tailwindcss/vite` plugin already in devDependencies (v^4.0.0). Must be added to `vite.config.ts`:
```ts
import tailwindcss from "@tailwindcss/vite";
export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
});
```

**Dark mode via CSS `light-dark()` function or class-based:**
- System preference: Use `prefers-color-scheme: dark` media query
- Manual toggle: Use class-based approach with `dark:` prefix
- For SPA mode (ssr=false): No hydration mismatch concern — class-based is simpler

**Decision:** Use class-based dark mode on `<html>` element. When no class set, CSS `prefers-color-scheme` handles system preference. Toggle adds/removes `dark` class.

### bits-ui v2.16.4 Component API

**Source:** bits-ui.com docs, npm

bits-ui is **headless** — ships no styling. Components use `Component.SubPart` naming:

```svelte
<script>
  import { Button, Dialog, Input, Select } from "bits-ui";
</script>

<Button.Root class="px-4 py-2 bg-primary-500 text-white rounded">Click</Button.Root>

<Dialog.Root>
  <Dialog.Trigger>Open</Dialog.Trigger>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50" />
    <Dialog.Content class="fixed top-1/2 left-1/2 -translate-1/2 bg-white rounded-lg p-6">
      <Dialog.Title>Title</Dialog.Title>
      <Dialog.Description>Description</Dialog.Description>
      <Dialog.Close>Close</Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
```

**Styling approaches:**
1. Class props (recommended) — Pass Tailwind classes directly
2. Data attributes — Target `[data-button-root]` etc. in CSS
3. Child snippet — Render custom element via `{@render children()}`

**For wrapper components:** Use class props pattern. Each wrapper accepts a `class` prop and merges with theme defaults using `cn()` utility (clsx + tailwind-merge).

### Dark Mode in SvelteKit SPA

**Source:** TailwindCSS v4 dark mode docs, SvelteKit SPA patterns

Since `ssr = false` in this project, there's no hydration mismatch risk. Approach:

1. CSS: Define light/dark tokens using CSS custom properties
2. JS: Read `localStorage.getItem('theme')` on mount, apply to `<html class="dark">`
3. Toggle: Set/remove `dark` class on `<html>`, persist to localStorage
4. System default: If no localStorage, match `prefers-color-scheme`

**Tailwind v4 dark mode config:**
```css
@import "tailwindcss";
/* Tailwind v4 dark mode works via class strategy by default */
/* Use dark: prefix in utilities */
```

### Component Wrapper Pattern

**Recommended:** Flat structure with `cn()` utility for class merging.

```
$lib/
├── components/
│   ├── ui/
│   │   ├── Button.svelte
│   │   ├── Input.svelte
│   │   ├── Dialog.svelte
│   │   ├── Select.svelte
│   │   ├── Card.svelte
│   │   ├── Badge.svelte
│   │   ├── Switch.svelte
│   │   ├── Toast.svelte
│   │   └── Dropdown.svelte
│   └── index.ts          (re-exports all UI components)
├── utils/
│   └── cn.ts             (clsx + tailwind-merge)
└── stores/
    └── theme.ts           (dark mode state)
```

Each wrapper:
- Imports bits-ui primitive
- Applies theme styling via Tailwind classes
- Exposes consistent props API (variant, size, class)
- Merges user classes via `cn()`

### cn() Utility

Need `clsx` + `tailwind-merge` for class merging:

```ts
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

Dependencies: `clsx`, `tailwind-merge` (need to add to package.json)

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Dark mode mechanism | Class-based on `<html>` | SPA mode = no hydration concern; class toggle simpler than data-attr |
| CSS custom property naming | `--color-primary-{scale}` | Follows Tailwind v4 @theme convention |
| Component wrapper depth | Full wrappers | Per D-06: apply theme, consistent props API |
| Class merging utility | clsx + tailwind-merge | Standard pattern, avoids class conflicts |
| File organization | Flat `$lib/components/ui/` | Per D-07: simple structure, no per-component folders |

## Validation Architecture

1. **CSS compilation:** `vite build` must succeed with `@tailwindcss/vite` plugin
2. **Component rendering:** Each wrapper component must render without errors
3. **Dark mode toggle:** Switching theme must update all themed components
4. **Import paths:** `$lib/components` must export all wrappers

## Pitfalls Found

1. **@tailwindcss/vite must come BEFORE sveltekit()** in vite plugins array — CSS processing order matters
2. **bits-ui Dialog needs Portal** — must wrap overlay/content in `<Dialog.Portal>`
3. **tailwind-merge may not recognize custom colors** — need to configure extend in merge config or use default
4. **No `tailwind.config.js`** in v4 — all config in CSS @theme block

## RESEARCH COMPLETE
