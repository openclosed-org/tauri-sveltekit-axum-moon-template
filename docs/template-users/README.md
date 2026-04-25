# Template User Guide

> Audience: teams using this repository via GitHub "Use this template".
> Keep this directory small. If guidance fits in the root `README.md`, it should usually stay there instead.

## What To Read First

If you are adopting this repository as a starting point, the shortest useful path is:

1. `README.md`
2. `docs/operations/local-dev.md`
3. `docs/operations/secret-management.md`
4. `docs/operations/counter-service-reference-chain.md`
5. `docs/template-users/template-init.md`

## What `template-init` Is For

`just template-init` is a conservative dry-run helper.

Its job is to preview upstream-maintainer artifacts a derived project may want to review or remove, such as:

1. upstream governance docs
2. archived historical notes
3. open-source maintenance templates and release automation files

It keeps the agent protocol and machine-readable project guidance intact. It is not meant to aggressively prune the main backend code tree.

## Keep Or Remove In Your Derived Project

After adoption, keep only the docs and project files that still describe your real workflows.

If a file only exists to support the upstream template as an open-source project, it is usually a candidate to remove from your derived repo.

The current implementation is intentionally conservative and `dry-run` only.
