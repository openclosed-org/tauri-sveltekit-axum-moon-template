/**
 * E2E Full Gate Runner
 * 
 * Orchestrates the complete E2E test pipeline:
 * 1. Runtime preflight check (API ready + types + ports)
 * 2. Web Playwright E2E tests
 * 3. Tauri Desktop E2E tests
 * 
 * Runs web and desktop tests in parallel after preflight passes
 */

import { runSync, runInherit } from '../lib/spawn.js';
import process from 'node:process';

interface TestLane {
  name: string;
  cwd: string;
  args: string[];
}

interface TestResult {
  name: string;
  exitCode: number;
}

const isWindows = process.platform === 'win32';

async function runPreflight(): Promise<boolean> {
  console.log('=== Runtime Preflight ===');
  const result = runSync(process.execPath, ['scripts/e2e/runtime-preflight.ts'], {
    cwd: process.cwd(),
  });

  if (!result.success) {
    console.log('\n=== E2E Full Gate Summary ===');
    console.log(`[FAIL] Runtime preflight (exit ${result.exitCode})`);
    return false;
  }

  console.log('[PASS] Runtime preflight');
  return true;
}

async function runLane(lane: TestLane): Promise<TestResult> {
  console.log(`\n=== ${lane.name} ===`);
  const exitCode = await runInherit(isWindows ? 'bun.cmd' : 'bun', lane.args, {
    cwd: lane.cwd,
  });

  return { name: lane.name, exitCode };
}

async function main(): Promise<number> {
  // Step 1: Preflight
  const preflightOk = await runPreflight();
  if (!preflightOk) {
    return 1;
  }

  // Step 2: Run test lanes in parallel
  const lanes: TestLane[] = [
    {
      name: 'Web Playwright E2E matrix',
      cwd: 'apps/client/web/app',
      args: ['run', 'test:e2e'],
    },
    {
      name: 'Tauri Playwright Desktop E2E (CI)',
      cwd: 'e2e-desktop-playwright',
      args: ['run', 'test:ci'],
    },
  ];

  console.log('\n=== Starting Parallel E2E Tests ===');
  console.log(`Running ${lanes.length} lanes in parallel`);

  const results = await Promise.all(lanes.map(runLane));

  // Step 3: Summary
  console.log('\n=== E2E Full Gate Summary ===');
  let hasFailure = false;

  for (const result of results) {
    const ok = result.exitCode === 0;
    if (!ok) hasFailure = true;
    console.log(`${ok ? '[PASS]' : '[FAIL]'} ${result.name} (exit ${result.exitCode})`);
  }

  return hasFailure ? 1 : 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
