/**
 * Verify Handoff — Validate that a subagent's changes are ready
 * for convergence and total verification.
 *
 * This script:
 * 1. Checks that modified files are within the subagent's writable boundaries
 * 2. Runs the subagent's scoped gates
 * 3. Reports readiness for total verify
 *
 * Usage:
 *   bun run scripts/verify-handoff.ts contract-agent
 *   bun run scripts/verify-handoff.ts app-shell-agent
 *   bun run scripts/verify-handoff.ts server-agent
 *   bun run scripts/verify-handoff.ts service-agent
 *   bun run scripts/verify-handoff.ts worker-agent
 *   bun run scripts/verify-handoff.ts platform-ops-agent
 */

import { run } from './lib/spawn.ts';
import process from 'node:process';

interface SubagentBoundary {
  writable: string[];
  readonly: string[];
}

// Boundary definitions — derived from agent/manifests/subagents.yml
// When updating boundaries, also update the manifest
const SUBAGENT_BOUNDARIES: Record<string, SubagentBoundary> = {
  'contract-agent': {
    writable: ['packages/contracts/', 'docs/contracts/', 'verification/contract/'],
    readonly: ['packages/sdk/', 'docs/generated/', 'infra/kubernetes/rendered/', 'platform/catalog/'],
  },
  'app-shell-agent': {
    writable: [],
    readonly: ['services/', 'workers/', 'infra/', 'packages/sdk/'],
  },
  'server-agent': {
    writable: ['servers/', 'packages/contracts/'],
    readonly: ['services/', 'infra/', 'apps/', 'workers/'],
  },
  'service-agent': {
    writable: ['services/', 'fixtures/', 'verification/', 'packages/contracts/'],
    readonly: ['infra/', 'packages/', 'apps/', 'servers/', 'workers/'],
  },
  'worker-agent': {
    writable: ['workers/', 'verification/resilience/', 'verification/topology/'],
    readonly: ['apps/', 'packages/sdk/', 'infra/'],
  },
  'platform-ops-agent': {
    writable: ['platform/model/', 'platform/schema/', 'platform/generators/', 'platform/validators/', 'infra/', 'ops/', 'docs/platform-model/', 'docs/operations/'],
    readonly: ['platform/catalog/', 'infra/kubernetes/rendered/', 'docs/generated/', 'packages/sdk/'],
  },
};

function pathMatchesBoundary(path: string, boundary: string): boolean {
  return path.startsWith(boundary) || path.includes('/' + boundary);
}

async function getModifiedPaths(): Promise<string[]> {
  const result = await run('git', ['diff', '--staged', '--name-only', '--diff-filter=ACMR']);
  if (result.success && result.output.trim()) {
    return result.output.trim().split('\n').filter(Boolean);
  }

  const result2 = await run('git', ['diff', '--name-only']);
  if (result2.success && result2.output.trim()) {
    return result2.output.trim().split('\n').filter(Boolean);
  }

  return [];
}

function checkBoundary(paths: string[], boundary: SubagentBoundary): { valid: string[]; violations: string[] } {
  const valid: string[] = [];
  const violations: string[] = [];

  for (const path of paths) {
    const inWritable = boundary.writable.some((b) => pathMatchesBoundary(path, b));
    const inReadonly = boundary.readonly.some((b) => pathMatchesBoundary(path, b));

    if (inWritable) {
      valid.push(path);
    } else if (inReadonly) {
      violations.push(`${path} (read-only — generated or owned by another agent)`);
    } else {
      // Path not in explicit writable or readonly — allow (planner-level files)
      valid.push(path);
    }
  }

  return { valid, violations };
}

async function main(): Promise<number> {
  const args = process.argv.slice(2);
  const agent = args[0];

  if (!agent) {
    console.error('Usage: bun run scripts/verify-handoff.ts <subagent-name>');
    console.error('Available: contract-agent, app-shell-agent, server-agent, service-agent, worker-agent, platform-ops-agent');
    return 1;
  }

  const boundary = SUBAGENT_BOUNDARIES[agent];

  if (!boundary) {
    console.error(`Unknown subagent: ${agent}`);
    console.error('Available subagents:');
    for (const name of Object.keys(SUBAGENT_BOUNDARIES)) {
      console.error(`  ${name}`);
    }
    return 1;
  }

  console.log(`\n=== Verifying handoff for ${agent} ===\n`);

  const paths = await getModifiedPaths();

  if (paths.length === 0) {
    console.log('No modified files to verify.');
    console.log('Run scoped gates anyway...');
  }

  console.log(`Modified files: ${paths.length}`);
  for (const p of paths) {
    console.log(`  ${p}`);
  }

  // Boundary check
  console.log('\n--- Boundary Check ---');
  const { valid, violations } = checkBoundary(paths, boundary);

  if (violations.length > 0) {
    console.error('\n✗ Boundary violations:');
    for (const v of violations) {
      console.error(`  ${v}`);
    }
    console.error('\nThese files are read-only or owned by another subagent.');
    console.error('Please revert changes to these files.');
    return 1;
  }

  console.log(`✓ All ${valid.length} modified files are within writable boundaries`);

  if (agent === 'app-shell-agent') {
    console.log('\nNo root-scoped verification is defined for app-shell-agent.');
    console.log('Validate retained app shells from their own local command surface if those directories remain in the repo.');
    console.log('\n=== Handoff Verified ===');
    console.log(`${agent} changes are ready for convergence.`);
    console.log('Next step: run total verify (just verify)');
    return 0;
  }

  // Run scoped gates
  console.log('\n--- Scoped Gates ---');
  const result = await run('bun', ['run', 'scripts/run-scoped-gates.ts', agent]);

  if (!result.success) {
    console.error('\n✗ Scoped gates failed — handoff not ready');
    return 1;
  }

  console.log('\n=== Handoff Verified ===');
  console.log(`${agent} changes are ready for convergence.`);
  console.log('Next step: run total verify (just verify)');

  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
