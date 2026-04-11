# services/indexer

> Protocol event indexer — pulls from various sources, normalizes to business DTOs, writes to Turso.

## Status
- [ ] Phase 1: Stub — no implementation yet
- [ ] Phase 2+: Implement per-protocol sources (Nostr, Farcaster, EVM, TON, Solana)

## Dependencies
- `packages/web3` (protocol SDKs)
- `packages/core` (base types)
- `packages/adapters/turso` (storage)

## Architecture
- `sources/` — Per-protocol data pullers
- `transformers/` — Raw events → business DTOs
- `sinks/` — Write to Turso / cache / trigger domain events
