# Phase 03: Application Pages - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Three core pages (Login, Counter, Admin dashboard) are functional and responsive on mobile (375px) and desktop (1280px) viewports. SPA routing works between all pages without full page reloads. Authentication logic and backend integration are separate phases — this phase delivers the UI shell and navigation structure.

</domain>

<decisions>
## Implementation Decisions

### Routing & Layout Structure
- **D-01:** SvelteKit route groups: `(auth)` for login, `(app)` for counter + admin
- **D-02:** `(auth)/login` — standalone page, no sidebar, no tab bar
- **D-03:** `(app)/+layout.svelte` — shared layout wrapping counter and admin pages
- **D-04:** URL paths: `/login`, `/counter`, `/admin`

### Navigation Pattern
- **D-05:** Mobile (<800px): bottom tab bar with 3 tabs (Counter, Admin, Settings)
- **D-06:** Desktop (>=800px): left collapsible sidebar
- **D-07:** md (800px): sidebar defaults to icon-only collapsed (~64px)
- **D-08:** lg (1280px): sidebar defaults to icon+label expanded (~240px)
- **D-09:** User can manually toggle sidebar expand/collapse
- **D-10:** Sidebar content: app logo+name (top), nav links with icons (middle), settings/theme toggle (bottom)
- **D-11:** Login page has no navigation chrome (standalone `(auth)` layout)

### Login Page
- **D-12:** App logo + name at top, "欢迎使用" subtitle
- **D-13:** Google sign-in Button component (primary variant, centered)
- **D-14:** Email input field below (disabled placeholder state — hints at future support)
- **D-15:** Mobile: full-screen layout (no padding constraints, fills viewport)
- **D-16:** Desktop: centered card layout (~400px max-width, vertically centered)

### Counter Page
- **D-17:** Large centered number display (showcases Svelte 5 `$state` rune)
- **D-18:** Increment (+) and decrement (-) buttons flanking the number
- **D-19:** Reset button below the counter
- **D-20:** Counter can go negative (no minimum bound)
- **D-21:** Always centered layout, larger touch targets on mobile (h-12+ buttons)

### Admin Dashboard Page
- **D-22:** Placeholder content: chart placeholder (gray blocks simulating bar/line charts) + statistics cards
- **D-23:** Mobile: list layout (cards stacked vertically, full width)
- **D-24:** Desktop: 2-3 column card grid
- **D-25:** Sidebar navigation item highlighted when on admin page

### Agent's Discretion
- Exact number of statistics cards and chart types in admin placeholder
- Tab bar icon choices (Lucide icons from @lucide/svelte)
- Sidebar transition animation (slide, fade, or instant)
- Route transition animations between pages

### Folded Todos
None.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 3 — Phase goal, success criteria, dependencies
- `.planning/REQUIREMENTS.md` §UI-01, §UI-02 — Acceptance criteria for three pages and responsive layout

### Prior Phase (dependencies)
- `.planning/phases/02-ui-styling-infrastructure/02-CONTEXT.md` — Theme tokens, component wrappers, dark mode decisions, breakpoint definitions
- `.planning/phases/02-ui-styling-infrastructure/02-03-PLAN.md` — Root layout + component barrel export structure

### SvelteKit Routing
- SvelteKit route groups `(group)` — directory naming convention for shared layouts without URL segments
- SvelteKit `+layout.svelte` — nested layout composition pattern

### Existing Code
- `apps/desktop-ui/src/routes/+layout.svelte` — Root layout (imports app.css, renders children)
- `apps/desktop-ui/src/routes/+layout.ts` — SPA mode (ssr=false, prerender=false)
- `apps/desktop-ui/src/lib/components/index.ts` — Component barrel export (10 components available)

### Design System (from Phase 2)
- `apps/desktop-ui/src/app.css` — TailwindCSS v4 @theme tokens, light/dark CSS variables
- Breakpoints: sm:375px, md:800px, lg:1280px
- Theme store: `apps/desktop-ui/src/lib/stores/theme.ts` — getTheme/setTheme/toggleTheme

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Button** (`$lib/components/ui/Button.svelte`): primary/secondary/ghost/destructive variants, sm/md/lg sizes, icon support. Use for all interactive elements.
- **Input** (`$lib/components/ui/Input.svelte`): For login email field and any text inputs.
- **Card** (`$lib/components/ui/Card.svelte`): For admin dashboard stat cards and chart containers.
- **Badge** (`$lib/components/ui/Badge.svelte`): For status indicators on admin cards.
- **Switch** (`$lib/components/ui/Switch.svelte`): For dark mode toggle in sidebar settings.
- **LottiePlayer/IconAnimate**: For animated icons or illustrations if needed.
- **cn() utility** (`$lib/utils/cn.ts`): Tailwind class merging — use for all conditional styling.
- **theme store** (`$lib/stores/theme.ts`): getTheme/setTheme/toggleTheme — integrate with sidebar settings.

### Established Patterns
- Svelte 5 runes ($state, $derived, $effect, $props, $snippet) — all new code must use runes
- bits-ui primitives wrapped with theme styling — don't use raw bits-ui directly
- TailwindCSS v4 utility classes with custom theme tokens (primary-*, gray-*)
- SPA mode: no SSR, all client-side rendering
- Biome for formatting (not ESLint/Prettier)

### Integration Points
- `apps/desktop-ui/src/routes/+layout.svelte` — root layout already exists, (auth) and (app) groups will create nested +layout.svelte files
- `apps/desktop-ui/src/lib/components/index.ts` — barrel export, import components from `$lib/components`
- `apps/desktop-ui/src/app.css` — global styles, may need sidebar/tab-bar CSS additions
- Sidebar needs access to `$page.url.pathname` for active nav highlighting (SvelteKit `$page` store)

</code_context>

<specifics>
## Specific Ideas

- Mobile-first cross-platform feel: bottom tab bar like Linear Mobile / Notion Mobile
- Desktop sidebar like Linear desktop: clean, collapsible, icon-first
- Login page: "like Notion's login" — clean centered card, prominent CTA button
- Counter: large monospace number (showcases `$state` rune for boilerplate users)
- Admin: "like Linear's dashboard" — card grid with subtle shadows, chart placeholders as gray gradient blocks
- Linear/Vercel aesthetic throughout: muted palette, generous whitespace, subtle borders, no heavy shadows

</specifics>

<deferred>
## Deferred Ideas

- Google OAuth actual implementation — Phase 6
- Admin dashboard real data — future phase
- Counter persistence (localStorage / backend sync) — future phase
- Page transition animations — can add later, not blocking
- Settings page with full preferences — expand when needed

</deferred>

---

*Phase: 03-application-pages*
*Context gathered: 2026-03-28*
