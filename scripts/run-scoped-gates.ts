/**
 * Gate selection helper.
 *
 * Gates are selected from changed paths, risk category, and evidence level.
 * Subagent identity routes work but no longer makes a gate required.
 *
 * Usage:
 *   bun run scripts/run-scoped-gates.ts --list
 *   bun run scripts/run-scoped-gates.ts service-agent
 */

import process from 'node:process';

const gateMatrix = 'agent/manifests/gate-matrix.yml';

const legacyAgents = [
  'contract-agent',
  'app-shell-agent',
  'server-agent',
  'service-agent',
  'worker-agent',
  'platform-ops-agent',
];

function printGuidance(): void {
  console.log('\n=== Gate Selection ===\n');
  console.log('Gate selection is path/risk/evidence based, not subagent based.');
  console.log(`Use ${gateMatrix} to select advisory, guardrail, or invariant gates.`);
  console.log('Default backend-core guardrail: just verify-backend-primary');
  console.log('Broader repo-wide guardrail when needed: just verify');
  console.log('Release/P0 invariant gate only when justified: just gate-release');
  console.log('\nLegacy agent names accepted by this helper:');
  for (const agent of legacyAgents) {
    console.log(`  - ${agent}`);
  }
}

function main(): number {
  const args = process.argv.slice(2);

  if (args.includes('--list') || args.includes('-l')) {
    printGuidance();
    return 0;
  }

  const agent = args[0];
  if (!agent) {
    console.error('Usage: bun run scripts/run-scoped-gates.ts [--list|<subagent-name>]');
    return 1;
  }

  if (!legacyAgents.includes(agent)) {
    console.error(`Unknown legacy subagent: ${agent}`);
    console.error('Run with --list for guidance.');
    return 1;
  }

  console.log(`\n=== Gate Selection for ${agent} ===\n`);
  console.log('No gate is required solely because this subagent handled the change.');
  console.log(`Select gates from changed paths, risk, and evidence level in ${gateMatrix}.`);
  console.log('This compatibility helper does not run heavy gates automatically.');
  return 0;
}

process.exit(main());
