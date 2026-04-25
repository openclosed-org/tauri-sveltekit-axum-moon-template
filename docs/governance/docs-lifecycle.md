# Documentation Lifecycle

Keep tracked docs small and durable.

## Keep Tracked

Tracked `docs/**` should be reserved for:

1. current operator and developer guidance
2. durable architecture decisions
3. minimal template-adoption guidance
4. a small amount of stable governance guidance
5. historical notes worth preserving as project history

## Move To Archive

Use `docs/archive/**` when material still has historical value, but is no longer current-state guidance.

Examples:

1. launch notes
2. old execution plans kept only for project history

## Keep In `docs/_local/`

Use `docs/_local/` for material that should not become shared documentation debt:

1. one-off refactor backlogs
2. temporary execution checklists
3. scratch notes and personal planning
4. exploratory drafts that have not become ADRs or stable guidance

`docs/_local/` is the gitignored maintainer workspace for active planning material that stays outside the shared docs flow.
If you need lightweight local revision tracking, use frontmatter metadata or explicit revision notes inside the document.

## Promotion Rule

Move something out of `_local` only if it becomes one of these:

1. reusable onboarding guidance
2. stable operator workflow
3. durable contributor rule
4. architecture decision with long-term value
5. template initialization or migration guidance
