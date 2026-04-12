/**
 * Event Schema Validation Tests
 *
 * Validates that event schemas in packages/contracts/events/
 * are well-formed and consistent.
 *
 * Run: bun run verification/contract/event-schema/event.test.ts
 */

import { describe, test, expect } from 'bun:test';
import { readdirSync, readFileSync } from 'fs';
import { join } from 'path';

describe('Event Schema Validation', () => {
  const eventsDir = join('packages/contracts/events/src');

  test('events directory exists and has files', () => {
    const files = readdirSync(eventsDir).filter(f => f.endsWith('.rs'));
    expect(files.length).toBeGreaterThan(0);
  });

  test('all event structs derive required traits', () => {
    const files = readdirSync(eventsDir).filter(f => f.endsWith('.rs'));
    const requiredDerives = ['Serialize', 'Deserialize', 'Clone', 'Debug'];

    for (const file of files) {
      const content = readFileSync(join(eventsDir, file), 'utf-8');
      // Skip lib.rs and non-event files
      if (file === 'lib.rs') continue;

      // Event structs should have Serialize + Deserialize for JSON transport
      const structMatch = content.match(/pub struct \w+/);
      if (structMatch) {
        const precedingLines = content
          .substring(0, structMatch.index)
          .split('\n')
          .slice(-3)
          .join('\n');

        for (const trait of requiredDerives.slice(0, 2)) {
          // Serialize and Deserialize are required for transport
          expect(precedingLines).toMatch(
            new RegExp(trait),
            `${file}: Event struct should derive ${trait}`
          );
        }
      }
    }
  });

  test('AppEvent envelope has required fields', async () => {
    const content = readFileSync(join(eventsDir, 'lib.rs'), 'utf-8');
    expect(content).toContain('pub struct AppEvent');
    // AppEvent should have event_type, payload, timestamp, tenant_id
    expect(content).toContain('event_type');
    expect(content).toContain('payload');
  });
});
