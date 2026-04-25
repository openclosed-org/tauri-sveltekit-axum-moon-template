# Services

`services/` contains business capability and state-boundary libraries.

Keep this file minimal. Current source-of-truth lives in the service directories themselves plus `agent/codemap.yml`.

## Current Rule Of Thumb

1. Every service directory must carry a `model.yaml` that explains its current semantic status.
2. `counter-service` is the only default copy target for new business services.
3. `tenant-service` may be consulted as a secondary semantics reference when a feature truly needs multi-entity, workflow, or compensation semantics.
4. Stub services are placeholders, not reference modules.
