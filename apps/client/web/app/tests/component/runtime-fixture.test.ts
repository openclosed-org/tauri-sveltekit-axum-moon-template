import { describe, expect, it } from 'vitest';
import { ensureApiReady } from '../fixtures/runtime';

describe('web runtime readiness fixture contract', () => {
	it('exports ensureApiReady function for e2e bootstrap', () => {
		expect(typeof ensureApiReady).toBe('function');
	});
});
