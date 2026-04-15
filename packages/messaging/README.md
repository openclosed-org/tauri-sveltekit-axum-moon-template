# Event Bus (Deprecated Service Placeholder)

> This directory is a transitional artifact from the previous architecture.
> Event bus behavior should live in shared runtime/packages and relay workers,
> not as a long-term business service.

## Current Status

1. Deprecated as a business service
2. Kept temporarily to avoid destructive removal during harness rebuild
3. Target ownership is split between:
   - `packages/messaging/`
   - `workers/outbox-relay/`
4. `model.yaml` exists only so the deprecated state is explicit in the harness

## Rule

Do not use this directory as the template for new services.
Do not model new business capabilities after `event-bus`.
