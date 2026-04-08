/**
 * Rust Test Runner
 * 
 * Unified interface for all Rust test suites
 * Stage: Testing
 */

import { runInherit, requireTool } from '../lib/spawn.js';
import process from 'node:process';

const GREEN = '\x1b[0;32m';
const RED = '\x1b[0;31m';
const YELLOW = '\x1b[1;33m';
const BLUE = '\x1b[0;34m';
const NC = '\x1b[0m';

const log = (...args: string[]) => console.log(`${BLUE}[test]${NC}`, ...args);
const ok = (...args: string[]) => console.log(`${GREEN}[✓]${NC}`, ...args);
const fail = (...args: string[]) => console.log(`${RED}[✗]${NC}`, ...args);
const warn = (...args: string[]) => console.log(`${YELLOW}[!]${NC}`, ...args);

async function runNextest(extraArgs: string[] = []): Promise<number> {
  log('Running cargo-nextest...');
  await requireTool('cargo-nextest', 'cargo install cargo-nextest --locked');

  const profile = process.env.PROFILE || 'default';
  log(`Profile: ${profile}`);

  const exitCode = await runInherit('cargo', ['nextest', 'run', '--workspace', '--profile', profile, ...extraArgs]);
  if (exitCode === 0) {
    ok('All nextest tests passed');
  } else {
    fail(`nextest tests failed (exit code: ${exitCode})`);
  }
  return exitCode;
}

async function runCoverage(extraArgs: string[] = []): Promise<number> {
  log('Running cargo-llvm-cov...');
  await requireTool('cargo-llvm-cov', 'cargo install cargo-llvm-cov --locked');

  const outputFormat = process.env.COV_FORMAT || 'lcov';
  const outputPath = 'target/lcov.info';

  log(`Format: ${outputFormat} → ${outputPath}`);

  const exitCode = await runInherit('cargo', [
    'llvm-cov', '--workspace',
    `--${outputFormat}`,
    '--output-path', outputPath,
    '--ignore-filename-regex', 'tests/',
    ...extraArgs,
  ]);

  if (exitCode === 0) {
    ok(`Coverage report generated: ${outputPath}`);
    await runInherit('cargo', ['llvm-cov', '--workspace', '--summary-only']);
  } else {
    fail(`Coverage run failed (exit code: ${exitCode})`);
  }
  return exitCode;
}

async function runHack(extraArgs: string[] = []): Promise<number> {
  log('Running cargo-hack feature powerset...');
  await requireTool('cargo-hack', 'cargo install cargo-hack --locked');

  const exitCode = await runInherit('cargo', ['hack', 'check', '--workspace', '--feature-powerset', ...extraArgs]);
  if (exitCode === 0) {
    ok('All feature combinations compile');
  } else {
    fail(`Some feature combinations failed (exit code: ${exitCode})`);
  }
  return exitCode;
}

async function runMutants(extraArgs: string[] = []): Promise<number> {
  log('Running cargo-mutants...');
  await requireTool('cargo-mutants', 'cargo install cargo-mutants --locked');

  const exitCode = await runInherit('cargo', ['mutants', '--workspace', ...extraArgs]);
  if (exitCode === 0) {
    ok('All mutants caught by tests');
  } else {
    warn('Some mutants survive — tests may need strengthening');
  }
  return 0;
}

async function runQuick(): Promise<number> {
  log('Running quick smoke test...');

  let exitCode = await runInherit('cargo', ['check', '--workspace', '--quiet']);
  if (exitCode === 0) {
    ok('cargo check');
  } else {
    fail('cargo check');
    return 1;
  }

  exitCode = await runInherit('cargo', ['test', '--workspace', '--lib', '--quiet']);
  if (exitCode === 0) {
    ok('cargo test --lib');
  } else {
    fail('cargo test --lib');
    return 1;
  }

  ok('Quick smoke test passed');
  return 0;
}

async function runAll(): Promise<number> {
  let failures = 0;

  if (await runQuick() !== 0) failures++;
  if (await runNextest() !== 0) failures++;
  if (await runCoverage() !== 0) failures++;
  if (await runHack() !== 0) failures++;
  await runMutants();

  console.log('');
  log('═══════════════════════════════════════');
  if (failures === 0) {
    ok('All test suites passed');
  } else {
    fail(`${failures} suite(s) had issues`);
  }
  log('═══════════════════════════════════════');

  return failures;
}

const commands: Record<string, (args: string[]) => Promise<number>> = {
  nextest: runNextest,
  coverage: runCoverage,
  hack: runHack,
  mutants: runMutants,
  quick: runQuick,
  all: runAll,
};

const cmd = process.argv[2] || 'nextest';
const extraArgs = process.argv.slice(3);

if (!commands[cmd]) {
  console.log(`Usage: bun run scripts/test/run.ts {nextest|coverage|hack|mutants|quick|all}`);
  console.log('');
  console.log('Commands:');
  console.log('  nextest    Run tests with cargo-nextest (default)');
  console.log('  coverage   Run tests with cargo-llvm-cov');
  console.log('  hack       Run cargo-hack feature powerset check');
  console.log('  mutants    Run cargo-mutants mutation testing');
  console.log('  quick      Quick smoke test (unit only)');
  console.log('  all        Run all test suites');
  process.exit(1);
}

commands[cmd](extraArgs)
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
