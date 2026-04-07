import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

const workspaceRoot = process.cwd();
const scriptPath = path.join(workspaceRoot, 'scripts', 'e2e', 'runtime-preflight.mjs');

function runPreflight(env = {}) {
	return spawnSync(process.execPath, [scriptPath], {
		cwd: workspaceRoot,
		env: { ...process.env, ...env },
		encoding: 'utf8'
	});
}

test('preflight fails with clear message when API is not ready', () => {
	const result = runPreflight({
		E2E_PREFLIGHT_TEST_MODE: '1',
		E2E_PREFLIGHT_READYZ_STATUS: 'down'
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr + result.stdout, /readyz/i);
});

test('preflight fails when required svelte types are missing', () => {
	const result = runPreflight({
		E2E_PREFLIGHT_TEST_MODE: '1',
		E2E_PREFLIGHT_READYZ_STATUS: 'ok',
		E2E_PREFLIGHT_TYPES_STATUS: 'missing'
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr + result.stdout, /\.svelte-kit\/types/i);
});

test('preflight fails when required ports are occupied', () => {
	const result = runPreflight({
		E2E_PREFLIGHT_TEST_MODE: '1',
		E2E_PREFLIGHT_READYZ_STATUS: 'ok',
		E2E_PREFLIGHT_TYPES_STATUS: 'ok',
		E2E_PREFLIGHT_PORTS_STATUS: 'busy:5173'
	});

	assert.notEqual(result.status, 0);
	assert.match(result.stderr + result.stdout, /5173/);
});

test('preflight passes when runtime, ports, and types are healthy', () => {
	const result = runPreflight({
		E2E_PREFLIGHT_TEST_MODE: '1',
		E2E_PREFLIGHT_READYZ_STATUS: 'ok',
		E2E_PREFLIGHT_TYPES_STATUS: 'ok',
		E2E_PREFLIGHT_PORTS_STATUS: 'free'
	});

	assert.equal(result.status, 0, result.stderr || result.stdout);
	assert.match(result.stdout, /preflight passed/i);
});
