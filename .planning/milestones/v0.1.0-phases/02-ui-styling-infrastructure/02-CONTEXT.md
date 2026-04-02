# Phase 02: UI Styling Infrastructure - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Configure the frontend design system foundation: TailwindCSS v4 theming, bitsUI component integration with wrappers, dark mode infrastructure, and the component library directory structure. This is INFRASTRUCTURE — no pages or features. Output: a styled, themeable component library importable from `$lib/components`.

</domain>

<decisions>
## Implementation Decisions

### Theme Design Tokens
- **D-01:** Primary color: blue-indigo (like Linear's indigo-blue). Developer-tools aesthetic.
- **D-02:** Gray tone: neutral gray (Tailwind zinc). No warm or cool bias.
- **D-03:** Typography: system fonts (native to each OS). Zero-config, fastest load. No custom font imports.

### Dark Mode Strategy
- **D-04:** Default mode: system preference (`prefers-color-scheme`). No flash of wrong theme — must handle SSR/hydration correctly.
- **D-05:** Toggle placement: settings page only. Not in header — keeps header clean per Linear/Vercel aesthetic.

### Component Wrapping Depth
- **D-06:** Full wrapper components around bitsUI primitives. Apply theme styling, consistent props API.
- **D-07:** Extended initial set (8-10): Button, Input, Dialog, Select, Card, Badge, Switch, Toast, Dropdown. All in `$lib/components/`.

### Global CSS Foundation
- **D-08:** CSS reset: TailwindCSS v4 preflight only (built-in modern-normalize). No additional reset.
- **D-09:** Spacing: 4px base (Tailwind default, 0.25rem increments).
- **D-10:** Breakpoints: Tauri-optimized — sm:375 (mobile), md:800 (tablet/windowed), lg:1280 (desktop). Custom, not Tailwind defaults.

### Agent's Discretion
- Exact CSS custom property naming (e.g., `--color-primary-500` vs `--primary`) — agent decides based on TailwindCSS v4 @theme conventions.
- File organization within `$lib/components/` (flat vs per-component folders) — agent decides based on component complexity.
- Dark mode implementation method (class-based vs data-attribute vs CSS `light-dark()`) — agent picks best approach for SvelteKit + Tailwind v4.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 2 — Phase goal, success criteria, dependencies
- `.planning/REQUIREMENTS.md` §UI-03, §UI-04 — Acceptance criteria for bitsUI and TailwindCSS theme

### Stack & Architecture
- `.planning/research/STACK.md` §Frontend Supporting Libraries — Library versions, rationale
- `.planning/PROJECT.md` §Tech stack — Frontend stack overview, UI requirements

### TailwindCSS v4
- `.agents/skills/tailwindcss/SKILL.md` — TailwindCSS v4 configuration patterns, @theme usage, dark mode setup

### Prior Phase (dependencies)
- `.planning/phases/01-package-foundation/01-01-SUMMARY.md` — Frontend deps installed (tailwindcss 4.2.2, bits-ui 2.16.4)
- `.planning/phases/01-package-foundation/01-01-PLAN.md` — package.json structure reference

### Svelte 5 (framework context)
- Svelte 5 runes ($state, $derived, $effect) — all components must use runes syntax
- `$lib/components` convention — standard SvelteKit import path

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `apps/desktop-ui/package.json` — tailwindcss 4.2.2, @tailwindcss/vite, bits-ui 2.16.4, @lucide/svelte 1.7.0 already installed
- `apps/desktop-ui/svelte.config.js` — static adapter configured, vitePreprocess active
- `apps/desktop-ui/vite.config.ts` — sveltekit() plugin, needs @tailwindcss/vite added

### Established Patterns
- SvelteKit static adapter with SPA fallback — no SSR, all client-side rendering
- `+layout.ts` exports `ssr = false` and `prerender = false` — SPA mode confirmed
- Biome for linting/formatting — no ESLint/Prettier

### Integration Points
- `apps/desktop-ui/vite.config.ts` — needs @tailwindcss/vite plugin
- `apps/desktop-ui/src/app.html` — may need CSS import or theme script
- `apps/desktop-ui/src/routes/+layout.svelte` — does NOT exist yet, needs creation for global layout + CSS import
- `$lib/components/` — directory does NOT exist yet, must be created

### Gaps
- No `+layout.svelte` exists — must create one to import global CSS and wrap pages
- No `$lib/` directory — must create `$lib/components/`, `$lib/utils/`, etc.
- No CSS files at all — starting from zero

</code_context>

<specifics>
## Specific Ideas

- Clean minimal aesthetic (like Linear/Vercel) — muted palette, generous whitespace, subtle borders, no heavy shadows or gradients
- "A blank canvas" — no existing patterns to maintain, freedom to establish conventions
- TailwindCSS v4 CSS-first config (@theme blocks, not tailwind.config.js) — researcher should investigate exact @theme syntax for colors/fonts

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-ui-styling-infrastructure*
*Context gathered: 2026-03-27*
