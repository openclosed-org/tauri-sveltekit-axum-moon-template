# indexing-service

`indexing-service` is a planned service boundary kept only for platform semantics.

Current status:

1. It is not a workspace member.
2. It is not managed by `release-plz`.
3. It does not provide runtime behavior yet.
4. This directory exists to keep future indexing/search ownership explicit in the harness.

What stays here:

1. `model.yaml` for service semantics.
2. Minimal notes that explain why the boundary still exists.

If indexing/search becomes an active implementation later, recreate the Rust crate shell and add it to the workspace and release flow deliberately.
