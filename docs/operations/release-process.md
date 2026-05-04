# Release Process

This repository releases as one upstream template product. Tags and GitHub Releases are the public version contract; ordinary PRs and `main` pushes do not automatically update versions or publish tags.

## Consensus

1. Regular PRs accumulate unreleased changes on `main`.
2. A release is a maintainer decision, not a side effect of every merge.
3. `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md` are updated in an intentional release-prep change.
4. `release-plz` runs only when a maintainer manually triggers the release workflow.
5. SemVer checks must express the selected release line. Do not restore obsolete APIs only to satisfy a `minor` gate when the intended release is breaking.
6. For the pre-`1.0.0` template line, a material public contract reset such as `0.3.x -> 0.4.0` is checked with the `major` compatibility expectation because Rust public API removals are allowed only in that mode.

## Normal PR Flow

Ordinary PRs should keep the codebase healthy without forcing a version bump.

Expected evidence depends on changed paths, but common checks are:

```bash
just drift-check
just verify-contracts strict
just check-backend-primary
```

The governance SemVer job runs on pull requests as a breaking-change detector. If a PR intentionally removes or renames public Rust API, mark the PR explicitly with a breaking signal such as a conventional commit `!` marker or a `BREAKING CHANGE:` footer. That signal documents release impact; it does not publish a version.

`main` push workflows should not fail merely because accumulated unreleased changes will require a future breaking release. The blocking release SemVer gate belongs to the release cut.

## Preparing A Release

When maintainers decide to publish accumulated changes, create a release-prep PR.

1. Choose the next repository version, for example `0.4.0`.
2. Choose the release compatibility expectation: `patch`, `minor`, or `major`.
3. Update the root release anchor in `Cargo.toml`.
4. Update the matching `axum-harness` entry in `Cargo.lock`.
5. Add a new `CHANGELOG.md` section with changed behavior, fixes, breaking changes, and migration notes.
6. Run the release SemVer check against the previous tag.

Example for the current `v0.4.0` contract reset:

```bash
just semver-check 'v0.3.1' 'major' 'v[0-9]*.[0-9]*.[0-9]*'
just drift-check
just verify-contracts strict
```

If the release is not breaking, use `patch` or `minor` instead of `major`. Do not use `major` as a way to hide accidental API deletion; use it only when the release notes and migration guidance intentionally describe a breaking release.

## Publishing A Release

After the release-prep PR is merged to `main`, a maintainer publishes by manually running the `Release Automation` workflow from GitHub Actions.

Required input:

1. `release_type`: choose `patch`, `minor`, or `major` to match the prepared release.

Optional inputs:

1. `baseline_tag`: explicit existing tag to compare against, for example `v0.3.1`.
2. `release_tag_template`: defaults to `v{{ version }}`.
3. `release_tag_glob`: defaults to `v[0-9]*.[0-9]*.[0-9]*`.

For the current `v0.4.0` release, run:

```bash
gh workflow run release-plz.yml \
  --ref main \
  -f release_type=major \
  -f baseline_tag=v0.3.1
```

The workflow runs the repository SemVer check with the selected expectation and then invokes `release-plz` to create the GitHub Release/tag from the prepared root release anchor.

## Agent Guidance

When an agent sees release or SemVer failures:

1. First identify whether the workflow is a normal PR/main validation or a maintainer-triggered release cut.
2. Do not restore deleted DTOs, generated bindings, or obsolete compatibility layers just to satisfy a mismatched `minor` check.
3. If the change is intentionally breaking, ensure the release-prep PR and `CHANGELOG.md` say so and run `just semver-check <baseline> major <tag_glob>`.
4. If the change is not intended to be breaking, repair the causal API deletion at the owning boundary.
5. If the user asks how to update the version/tag, tell them to prepare a release PR first, then manually trigger `Release Automation` with the appropriate `release_type`.
