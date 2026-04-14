/**
 * Run Scoped Gates — Execute gates specific to a subagent's domain
 * instead of running the full global gate.
 *
 * Usage:
 *   bun run scripts/run-scoped-gates.ts contract-agent
 *   bun run scripts/run-scoped-gates.ts app-shell-agent
 *   bun run scripts/run-scoped-gates.ts server-agent
 *   bun run scripts/run-scoped-gates.ts service-agent
 *   bun run scripts/run-scoped-gates.ts worker-agent
 *   bun run scripts/run-scoped-gates.ts platform-ops-agent
 *   bun run scripts/run-scoped-gates.ts --list    # show all available gates
 */

import { run } from './lib/spawn.ts';
import process from 'node:process';

interface GateStep {
  label: string;
  cmd: string;
  args: string[];
  skip?: boolean;
  todo?: string;
}

interface SubagentGates {
  required: GateStep[];
  conditional: (GateStep & { when: string })[];
}

// Gate definitions — derived from agent/manifests/gate-matrix.yml
// When updating gates, also update the manifest
const SUBAGENT_GATES: Record<string, SubagentGates> = {
  'contract-agent': {
    required: [
      { label: 'Type generation', cmd: 'moon', args: ['run', 'repo:typegen'] },
      { label: 'Typecheck', cmd: 'just', args: ['typecheck'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
    ],
    conditional: [],
  },
  'app-shell-agent': {
    required: [
      { label: 'Frontend type check', cmd: 'just', args: ['web:check'] },
      { label: 'Frontend lint', cmd: 'just', args: ['web:lint'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
    ],
    conditional: [
      { label: 'E2E tests', cmd: 'just', args: ['test-e2e'], when: 'apps/ changed' },
      { label: 'Desktop tests', cmd: 'just', args: ['test-desktop'], when: 'apps/desktop/ changed' },
    ],
  },
  'server-agent': {
    required: [
      { label: 'Typecheck', cmd: 'just', args: ['typecheck'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
    ],
    conditional: [
      { label: 'Contract checks', cmd: 'just', args: ['contracts-check'], when: 'packages/contracts/ changed' },
    ],
  },
  'service-agent': {
    required: [
      { label: 'Typecheck', cmd: 'just', args: ['typecheck'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
    ],
    conditional: [],
  },
  'worker-agent': {
    required: [
      { label: 'Typecheck', cmd: 'just', args: ['typecheck'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
      { label: 'Resilience checks', cmd: 'echo', args: ['TODO: implement resilience checks'], skip: true, todo: 'Script not yet implemented' },
    ],
    conditional: [],
  },
  'platform-ops-agent': {
    required: [
      { label: 'Platform validation', cmd: 'just', args: ['validate-platform'] },
      { label: 'Topology validation', cmd: 'just', args: ['validate-topology'] },
      { label: 'Generated drift checks', cmd: 'just', args: ['verify-generated'] },
      { label: 'Boundary check', cmd: 'just', args: ['boundary-check'] },
    ],
    conditional: [],
  },
};

async function runGate(gate: GateStep, packageName?: string): Promise<{ success: boolean; skipped: boolean }> {
  if (gate.skip) {
    console.warn(`  ⚠ SKIP: ${gate.label} — ${gate.todo || 'not implemented'}`);
    return { success: true, skipped: true };
  }

  const cmd = gate.args.map((a) =>
    a === '<package>' ? (packageName || 'unknown') : a
  );

  console.log(`\n→ ${gate.label}: ${gate.cmd} ${cmd.join(' ')}`);

  const result = await run(gate.cmd, cmd, { stdio: 'pipe' });

  if (result.output) {
    const lines = result.output.split('\n').slice(-5);
    for (const line of lines) {
      console.log(`  ${line}`);
    }
  }

  if (result.success) {
    console.log(`  ✓ ${gate.label}`);
    return { success: true, skipped: false };
  }

  if (result.error) {
    console.error(`  ✗ ${gate.label}: ${result.error}`);
  }

  return { success: false, skipped: false };
}

async function main(): Promise<number> {
  const args = process.argv.slice(2);

  if (args.includes('--list') || args.includes('-l')) {
    console.log('\n=== Available Subagents and Gates ===\n');
    for (const [agent, gates] of Object.entries(SUBAGENT_GATES)) {
      console.log(`${agent}:`);
      console.log(`  Required:   ${gates.required.map((g) => g.label).join(', ')}`);
      if (gates.conditional.length > 0) {
        console.log(`  Conditional: ${gates.conditional.map((c) => `${c.label} (when: ${c.when})`).join(', ')}`);
      }
      console.log();
    }
    console.log('Full definitions: agent/manifests/gate-matrix.yml');
    return 0;
  }

  const agent = args[0];
  if (!agent) {
    console.error('Usage: bun run scripts/run-scoped-gates.ts <subagent-name>');
    console.error('Run with --list to see available subagents');
    return 1;
  }

  const agentGates = SUBAGENT_GATES[agent];

  if (!agentGates) {
    console.error(`Unknown subagent: ${agent}`);
    console.error('Available subagents:');
    for (const name of Object.keys(SUBAGENT_GATES)) {
      console.error(`  ${name}`);
    }
    return 1;
  }

  console.log(`\n=== Running scoped gates for ${agent} ===`);

  const failures: string[] = [];

  // Run required gates
  for (const gate of agentGates.required) {
    const { success, skipped } = await runGate(gate);
    if (!success && !skipped) {
      failures.push(gate.label);
    }
  }

  // Run conditional gates (simplified — runs all conditionals for now)
  for (const gate of agentGates.conditional) {
    console.log(`  (conditional: ${gate.when})`);
    const { success, skipped } = await runGate(gate);
    if (!success && !skipped) {
      failures.push(gate.label);
    }
  }

  if (failures.length > 0) {
    console.error(`\n✗ ${agent} gates failed: ${failures.join(', ')}`);
    return 1;
  }

  console.log(`\n✓ ${agent} scoped gates passed`);
  console.log('Full gate definitions: agent/manifests/gate-matrix.yml');
  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
