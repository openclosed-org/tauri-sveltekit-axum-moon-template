/**
 * Frontend Test Runner
 * 
 * Unified interface for all frontend quality checks
 * Stage: Testing
 */

import { runInherit } from '../lib/spawn.js';
import process from 'node:process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '../..');
const webDir = path.join(workspaceRoot, 'apps', 'client', 'web', 'app');

const GREEN = '\x1b[0;32m';
const RED = '\x1b[0;31m';
const YELLOW = '\x1b[1;33m';
const BLUE = '\x1b[0;34m';
const NC = '\x1b[0m';

const log = (...args: string[]) => console.log(`${BLUE}[frontend]${NC}`, ...args);
const ok = (...args: string[]) => console.log(`${GREEN}[✓]${NC}`, ...args);
const fail = (...args: string[]) => console.log(`${RED}[✗]${NC}`, ...args);

async function runBunScript(script: string, args: string[] = [], cwd: string = webDir): Promise<number> {
  return runInherit('bun', ['run', '--cwd', cwd, script, ...args], { cwd });
}

async function runCheck(): Promise<number> {
  log('Running svelte-check...');
  const exitCode = await runBunScript('check');
  if (exitCode === 0) ok('Type check passed');
  else fail('Type check failed');
  return exitCode;
}

async function runLint(): Promise<number> {
  log('Running biome lint...');
  const exitCode = await runBunScript('lint');
  if (exitCode === 0) ok('Lint passed');
  else fail('Lint failed');
  return exitCode;
}

async function runUnit(): Promise<number> {
  log('Running vitest unit tests...');
  const exitCode = await runBunScript('test:unit');
  if (exitCode === 0) ok('Unit tests passed');
  else fail('Unit tests failed');
  return exitCode;
}

async function runE2E(project?: string): Promise<number> {
  log('Running Playwright E2E tests...');
  const args: string[] = project ? ['test:e2e', '--project', project] : ['test:e2e'];
  const exitCode = await runBunScript(args[0], args.slice(1));
  if (exitCode === 0) ok('E2E tests passed');
  else fail('E2E tests failed');
  return exitCode;
}

async function runAll(): Promise<number> {
  let failures = 0;

  if (await runCheck() !== 0) failures++;
  if (await runLint() !== 0) failures++;
  if (await runUnit() !== 0) failures++;
  if (await runE2E() !== 0) failures++;

  console.log('');
  log('═══════════════════════════════════════');
  if (failures === 0) {
    ok('All frontend checks passed');
  } else {
    fail(`${failures} check(s) had issues`);
  }
  log('═══════════════════════════════════════');

  return failures;
}

const commands: Record<string, () => Promise<number>> = {
  check: runCheck,
  lint: runLint,
  unit: runUnit,
  e2e: () => runE2E(process.argv[3]),
  all: runAll,
};

const cmd = process.argv[2] || 'all';

if (!commands[cmd]) {
  console.log('Usage: bun run scripts/test/run-frontend.ts {check|lint|unit|e2e|all}');
  process.exit(1);
}

commands[cmd]()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
