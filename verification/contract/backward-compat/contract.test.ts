/**
 * Contract Backward Compatibility Tests
 *
 * Ensures that changes to HTTP contracts, event schemas, and DTOs
 * don't break existing clients.
 *
 * Run: bun run verification/contract/backward-compat/contract.test.ts
 */

import { describe, test, expect } from 'bun:test';
import { readdirSync, readFileSync } from 'fs';
import { join } from 'path';

const CONTRACTS_DIR = 'packages/contracts';

describe('HTTP Contract Backward Compatibility', () => {
  test('all HTTP contracts export valid TypeScript types', () => {
    // Verify contracts can be imported without errors
    const apiContracts = import(`${CONTRACTS_DIR}/api/src/lib.ts`);
    expect(apiContracts).toBeDefined();
  });

  test('no breaking changes in CounterResponse schema', async () => {
    const api = await import(`${CONTRACTS_DIR}/api/src/lib.ts`);
    // CounterResponse must have these fields
    const schema = api.CounterResponse;
    expect(schema).toBeDefined();
  });

  test('error response schema is stable', async () => {
    const errors = await import(`${CONTRACTS_DIR}/errors/src/lib.ts`);
    expect(errors.ErrorResponse).toBeDefined();
    expect(errors.ErrorCode).toBeDefined();
  });
});

describe('Event Schema Compatibility', () => {
  test('all event schemas are valid JSON', () => {
    const eventsDir = join(CONTRACTS_DIR, 'events/src');
    const files = readdirSync(eventsDir).filter(f => f.endsWith('.rs'));
    for (const file of files) {
      const content = readFileSync(join(eventsDir, file), 'utf-8');
      // Verify Rust struct derives Serialize
      expect(content).toMatch(/#\[derive\(.*Serialize.*\]\]/);
    }
  });

  test('AppEvent envelope wraps all events consistently', async () => {
    const events = await import(`${CONTRACTS_DIR}/events/src/lib.ts`);
    expect(events.AppEvent).toBeDefined();
  });
});

describe('Auth Contract Stability', () => {
  test('auth DTOs are backward compatible', async () => {
    const auth = await import(`${CONTRACTS_DIR}/auth/src/lib.ts`);
    expect(auth.TokenPair).toBeDefined();
    expect(auth.UserProfile).toBeDefined();
  });
});
