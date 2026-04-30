# Language Docs

`docs/language/**` defines shared vocabulary for agent-assisted work in this repository.

These files compress project-specific concepts so humans and agents can use the same names without re-explaining the architecture every session.

## Scope

Language docs define terms, avoided aliases, and relationships. They do not prove current runtime behavior.

Use them for:

1. issue titles, plans, and agent briefs
2. test names and acceptance criteria
3. architecture reviews and refactor discussions
4. distinguishing current state, declared intent, target state, and executable evidence

Do not use them as evidence that code, validators, tests, gates, or deployment paths exist.

## Files

1. `harness-language.md` defines repository-wide harness vocabulary.
2. `backend-reference-language.md` defines the default backend reference chain vocabulary.
3. `platform-language.md` defines platform model and delivery vocabulary.
4. `agent-language.md` defines agent, skill, and documentation workflow vocabulary.
5. `architecture-deepening.md` defines module-depth and seam-review vocabulary.

## Promotion Rule

New or disputed terms start in `docs/_local/language-candidates.md` when a workflow needs scratch vocabulary tracking. Use `docs/agents/language-candidates-template.md` as the template.

Promote a term here only when it is stable enough to reduce future ambiguity. Do not promote one-off plan language, temporary file names, or target-state claims.
