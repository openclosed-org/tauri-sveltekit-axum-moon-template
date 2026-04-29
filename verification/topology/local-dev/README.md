# Topology: Local Dev

> Local development topology verification placeholder.

## Status

1. This directory exists so `local-dev` remains a first-class topology with an explicit verification hook.
2. The topology intentionally keeps several workers disabled by default.
3. This is a placeholder, not a full E2E verification suite.

## Minimum Expectations

1. Core local infrastructure can start via `repo-tools infra local up`.
2. `web-bff` can run as the default synchronous backend entrypoint.
3. Optional workers can still be started independently when a task requires them.

## Follow-up

Add runnable topology verification once the local-dev path is formalized beyond the current bootstrap and ad hoc process model.
