/**
 * Route Task — Determine which subagent(s) should handle a given task
 * based on touched paths and routing rules.
 *
 * Usage:
 *   bun run scripts/route-task.ts                              # analyze staged changes
 *   bun run scripts/route-task.ts --paths apps/web/src/**      # analyze specific paths
 *   bun run scripts/route-task.ts --diff HEAD~1..HEAD          # analyze commit range
 *   bun run scripts/route-task.ts --list                       # show all routing rules
 */

import { run } from './lib/spawn.ts';
import process from 'node:process';

interface RoutingRule {
  pattern: string;
  agent: string;
}

interface RoutingResult {
  affectedAgents: string[];
  dispatchOrder: string[];
  touchedPaths: Map<string, string[]>;
}

// Routing rules — single source of truth derived from agent/manifests/routing-rules.yml
// When updating rules, also update agent/manifests/routing-rules.yml
const ROUTING_RULES: RoutingRule[] = [
  { pattern: 'packages/contracts/', agent: 'contract-agent' },
  { pattern: 'platform/model/', agent: 'platform-ops-agent' },
  { pattern: 'platform/schema/', agent: 'platform-ops-agent' },
  { pattern: 'platform/generators/', agent: 'platform-ops-agent' },
  { pattern: 'platform/validators/', agent: 'platform-ops-agent' },
  { pattern: 'apps/web/', agent: 'app-shell-agent' },
  { pattern: 'apps/desktop/', agent: 'app-shell-agent' },
  { pattern: 'apps/mobile/', agent: 'app-shell-agent' },
  { pattern: 'packages/ui/', agent: 'app-shell-agent' },
  { pattern: 'servers/', agent: 'server-agent' },
  { pattern: 'services/', agent: 'service-agent' },
  { pattern: 'workers/', agent: 'worker-agent' },
  { pattern: 'infra/', agent: 'platform-ops-agent' },
  { pattern: 'ops/', agent: 'platform-ops-agent' },
  { pattern: 'verification/e2e/', agent: 'app-shell-agent' },
  { pattern: 'verification/resilience/', agent: 'worker-agent' },
  { pattern: 'verification/topology/', agent: 'platform-ops-agent' },
  { pattern: 'verification/contract/', agent: 'contract-agent' },
];

// Dependency order for dispatch — derived from agent/manifests/routing-rules.yml
const DISPATCH_ORDER = [
  'platform-ops-agent',
  'contract-agent',
  'service-agent',
  'server-agent',
  'worker-agent',
  'app-shell-agent',
];

function routePath(path: string): string | null {
  for (const rule of ROUTING_RULES) {
    if (path.startsWith(rule.pattern) || path.includes('/' + rule.pattern)) {
      return rule.agent;
    }
  }
  return null;
}

async function getStagedPaths(): Promise<string[]> {
  const result = await run('git', ['diff', '--staged', '--name-only', '--diff-filter=ACMR']);
  if (!result.success) {
    const result2 = await run('git', ['diff', '--name-only']);
    if (!result2.success) {
      console.error('Warning: Could not read git diff. No paths to analyze.');
      return [];
    }
    return result2.output.trim().split('\n').filter(Boolean);
  }
  return result.output.trim().split('\n').filter(Boolean);
}

async function getCommitPaths(diffRange: string): Promise<string[]> {
  const result = await run('git', ['diff', '--name-only', diffRange]);
  if (!result.success) {
    console.error(`Warning: Could not read git diff for ${diffRange}`);
    return [];
  }
  return result.output.trim().split('\n').filter(Boolean);
}

function routePaths(paths: string[]): RoutingResult {
  const touchedByAgent = new Map<string, string[]>();

  for (const path of paths) {
    const agent = routePath(path);
    if (agent) {
      if (!touchedByAgent.has(agent)) {
        touchedByAgent.set(agent, []);
      }
      touchedByAgent.get(agent)!.push(path);
    }
  }

  // Build dispatch order (only agents with touched paths)
  const dispatchOrderFiltered = DISPATCH_ORDER.filter(
    (a) => touchedByAgent.has(a)
  );

  // Add "(verify)" at end if any agents dispatched
  if (dispatchOrderFiltered.length > 0) {
    dispatchOrderFiltered.push('(verify)');
  }

  const affectedAgents = [...touchedByAgent.keys()];

  return {
    affectedAgents,
    dispatchOrder: dispatchOrderFiltered,
    touchedPaths: touchedByAgent,
  };
}

function printResult(result: RoutingResult, allPaths: string[]): void {
  console.log('\n=== Task Routing Result ===\n');

  if (result.affectedAgents.length === 0) {
    console.log('No subagent domains affected by touched paths.');
    console.log('Planner can handle this directly.');
    console.log('\nTouched paths:');
    for (const p of allPaths) {
      console.log(`  ${p}`);
    }
    return;
  }

  console.log(`Affected domains:    ${result.affectedAgents.join(', ')}`);
  console.log(`Dispatch order:      ${result.dispatchOrder.join(' → ')}`);

  console.log('\nPath → Agent mapping:');
  for (const [agent, paths] of result.touchedPaths) {
    console.log(`\n  ${agent}:`);
    for (const p of paths) {
      console.log(`    ${p}`);
    }
  }

  console.log('\n💡 For full routing rules, see agent/manifests/routing-rules.yml');
}

function printRules(): void {
  console.log('\n=== Routing Rules ===\n');
  console.log('Path Pattern → Subagent\n');
  for (const rule of ROUTING_RULES) {
    console.log(`  ${rule.pattern.padEnd(35)} → ${rule.agent}`);
  }
  console.log(`\nDispatch order: ${DISPATCH_ORDER.join(' → ')} → (verify)`);
  console.log('\nFull rules: agent/manifests/routing-rules.yml');
}

async function main(): Promise<number> {
  const args = process.argv.slice(2);

  if (args.includes('--list') || args.includes('-l')) {
    printRules();
    return 0;
  }

  let paths: string[] = [];

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--paths') {
      paths.push(...args.slice(i + 1));
      break;
    } else if (args[i] === '--diff') {
      const diffRange = args[i + 1];
      if (!diffRange) {
        console.error('Error: --diff requires a range argument (e.g., HEAD~1..HEAD)');
        return 1;
      }
      paths = await getCommitPaths(diffRange);
      break;
    } else if (args[i] === '--help' || args[i] === '-h') {
      console.log('Usage:');
      console.log('  bun run scripts/route-task.ts                              # analyze staged changes');
      console.log('  bun run scripts/route-task.ts --paths apps/web/src/**      # analyze specific paths');
      console.log('  bun run scripts/route-task.ts --diff HEAD~1..HEAD          # analyze commit range');
      console.log('  bun run scripts/route-task.ts --list                       # show all routing rules');
      return 0;
    }
  }

  if (paths.length === 0) {
    paths = await getStagedPaths();
  }

  if (paths.length === 0) {
    console.log('No files to analyze. Stage changes or specify paths.');
    return 0;
  }

  const result = routePaths(paths);
  printResult(result, paths);

  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
