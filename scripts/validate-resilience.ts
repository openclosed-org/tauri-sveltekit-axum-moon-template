/**
 * Validate Resilience — Scan workers and verify resilience strategies
 * declared in agent/codemap.yml are present in the code.
 *
 * Usage:
 *   bun run scripts/validate-resilience.ts [--mode warn|strict]
 *
 * Modes:
 *   warn   — report missing strategies but exit 0
 *   strict — exit 1 if any worker is missing a required strategy
 */

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

// ── Types ────────────────────────────────────────────────────────────────────

type Mode = 'warn' | 'strict';

interface WorkerDecl {
  path: string;
  status?: string;
  notes?: string;
  must_have: string[];
}

interface ResilienceCheck {
  workerName: string;
  workerPath: string;
  strategy: string;
  present: boolean;
  evidence: string;
}

interface ValidationIssue {
  level: 'warn' | 'error';
  worker: string;
  strategy: string;
  message: string;
}

interface ParsedArgs {
  mode: Mode;
}

// ── Strategy detection rules ─────────────────────────────────────────────────

/**
 * For each resilience strategy, define how to detect it in a worker's source tree.
 * Each rule has one or more detection methods (checked in order).
 */
const STRATEGY_DETECTORS: Record<string, Array<{
  label: string;
  detect: (srcDir: string) => string | null;
}>> = {
  checkpoint: [
    {
      label: 'checkpoint module directory',
      detect: (srcDir) => {
        const checkpointDir = path.join(srcDir, 'checkpoint');
        if (existsSync(checkpointDir)) {
          const entries = readdirSync(checkpointDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `checkpoint/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        return null;
      },
    },
    {
      label: 'checkpoint in mod declarations or source content',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+checkpoint/.test(content)) {
            return 'mod checkpoint declared in main.rs';
          }
          if (/checkpoint/i.test(content)) {
            return 'checkpoint references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  dedupe: [
    {
      label: 'dedupe module directory',
      detect: (srcDir) => {
        const dedupeDir = path.join(srcDir, 'dedupe');
        if (existsSync(dedupeDir)) {
          const entries = readdirSync(dedupeDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `dedupe/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        return null;
      },
    },
    {
      label: 'dedupe/dedup in mod declarations or source content',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+dedupe/.test(content)) {
            return 'mod dedupe declared in main.rs';
          }
          if (/\b(dedupe|dedup)\b/i.test(content)) {
            return 'dedupe/dedup references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  idempotency: [
    {
      label: 'idempotency module directory',
      detect: (srcDir) => {
        const idempotencyDir = path.join(srcDir, 'idempotency');
        if (existsSync(idempotencyDir)) {
          const entries = readdirSync(idempotencyDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `idempotency/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        // Also check for idempotent.rs or similar
        const idempotentFile = path.join(srcDir, 'idempotent.rs');
        if (existsSync(idempotentFile)) {
          return 'idempotent.rs found';
        }
        return null;
      },
    },
    {
      label: 'idempotency keyword or module in source',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+idempoten/.test(content)) {
            return 'idempotency module declared in main.rs';
          }
          if (/\b(idempoten|idempotency_key|idempotent)\b/i.test(content)) {
            return 'idempotency references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  retry_policy: [
    {
      label: 'retry module directory',
      detect: (srcDir) => {
        const retryDir = path.join(srcDir, 'retry');
        if (existsSync(retryDir)) {
          const entries = readdirSync(retryDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `retry/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        return null;
      },
    },
    {
      label: 'retry configuration or module in source',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+retry/.test(content)) {
            return 'mod retry declared in main.rs';
          }
          if (/\b(retry_policy|retry_count|ExponentialBackoff|retry\b)/i.test(content)) {
            return 'retry references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  replay_strategy: [
    {
      label: 'replay module directory',
      detect: (srcDir) => {
        const replayDir = path.join(srcDir, 'replay');
        if (existsSync(replayDir)) {
          const entries = readdirSync(replayDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `replay/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        return null;
      },
    },
    {
      label: 'replay in mod declarations or source content',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+replay/.test(content)) {
            return 'mod replay declared in main.rs';
          }
          if (/\breplay/i.test(content)) {
            return 'replay references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  conflict_strategy: [
    {
      label: 'conflict module directory',
      detect: (srcDir) => {
        const conflictDir = path.join(srcDir, 'conflict');
        if (existsSync(conflictDir)) {
          const entries = readdirSync(conflictDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `conflict/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        return null;
      },
    },
    {
      label: 'conflict in mod declarations or source content',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+conflict/.test(content)) {
            return 'mod conflict declared in main.rs';
          }
          if (/\bconflict/i.test(content)) {
            return 'conflict references found in main.rs';
          }
        }
        return null;
      },
    },
  ],

  compensation_strategy: [
    {
      label: 'compensation module directory',
      detect: (srcDir) => {
        const compDir = path.join(srcDir, 'compensation');
        if (existsSync(compDir)) {
          const entries = readdirSync(compDir);
          if (entries.some((e) => e.endsWith('.rs'))) {
            return `compensation/ module found (${entries.filter((e) => e.endsWith('.rs')).join(', ')})`;
          }
        }
        // Also check for saga-related files
        const sagaFile = path.join(srcDir, 'saga.rs');
        if (existsSync(sagaFile)) {
          return 'saga.rs found';
        }
        return null;
      },
    },
    {
      label: 'compensation or saga in source content',
      detect: (srcDir) => {
        const mainRs = path.join(srcDir, 'main.rs');
        if (existsSync(mainRs)) {
          const content = readFileSync(mainRs, 'utf-8');
          if (/mod\s+compensation/.test(content)) {
            return 'mod compensation declared in main.rs';
          }
          if (/\b(compensation|compensate|saga)\b/i.test(content)) {
            return 'compensation/saga references found in main.rs';
          }
        }
        return null;
      },
    },
  ],
};

// Also check all .rs files recursively within src for deeper detection
function scanAllRustFiles(srcDir: string, patterns: RegExp[]): string | null {
  const results: string[] = [];
  const stack = [srcDir];

  while (stack.length > 0) {
    const current = stack.pop();
    if (!current) continue;

    try {
      const entries = readdirSync(current, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(current, entry.name);
        if (entry.isDirectory()) {
          stack.push(fullPath);
        } else if (entry.isFile() && entry.name.endsWith('.rs')) {
          const content = readFileSync(fullPath, 'utf-8');
          for (const pattern of patterns) {
            if (pattern.test(content)) {
              const relPath = path.relative(srcDir, fullPath);
              results.push(relPath);
              break; // Only count each file once
            }
          }
        }
      }
    } catch {
      // ignore permission errors
    }
  }

  if (results.length > 0) {
    return `${results.length} Rust file(s) with matches: ${results.join(', ')}`;
  }
  return null;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

const workspaceRoot = process.cwd();

function parseArgs(argv: string[]): ParsedArgs {
  let mode: Mode = 'warn';

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--mode') {
      const value = argv[index + 1];
      if (value === 'warn' || value === 'strict') {
        mode = value;
        index += 1;
        continue;
      }
    }

    console.error(`Unknown argument: ${arg}`);
    console.error('Usage: bun run scripts/validate-resilience.ts [--mode warn|strict]');
    process.exit(1);
  }

  return { mode };
}

async function readYaml<T>(relativePath: string): Promise<T> {
  const fullPath = path.join(workspaceRoot, relativePath);
  const content = await Bun.file(fullPath).text();
  return Bun.YAML.parse(content) as T;
}

/**
 * Strategy name aliases — map codemap names to detector keys.
 * The codemap may use compound names like "dedupe_or_resume_strategy".
 */
const STRATEGY_ALIASES: Record<string, string[]> = {
  dedupe_or_resume_strategy: ['dedupe', 'checkpoint'],
  dedupe_or_resume: ['dedupe', 'checkpoint'],
};

/** Detect whether a strategy is present in a worker's src directory. */
function detectStrategy(srcDir: string, strategy: string): { present: boolean; evidence: string } {
  // Check if this is an alias that maps to multiple detectors
  const aliases = STRATEGY_ALIASES[strategy];
  if (aliases) {
    const results = aliases.map((alias) => ({
      alias,
      ...detectStrategy(srcDir, alias),
    }));
    const anyPresent = results.some((r) => r.present);
    const evidenceParts = results.map((r) => `${r.alias}: ${r.present ? '✓' : '✗'} ${r.evidence}`);
    return {
      present: anyPresent,
      evidence: evidenceParts.join(' | '),
    };
  }

  const detectors = STRATEGY_DETECTORS[strategy];
  if (!detectors) {
    return { present: false, evidence: `unknown strategy: ${strategy}` };
  }

  // Try each detector in order
  for (const detector of detectors) {
    const result = detector.detect(srcDir);
    if (result) {
      return { present: true, evidence: `${detector.label}: ${result}` };
    }
  }

  // Fallback: scan all .rs files with strategy-specific patterns
  const fallbackPatterns: Record<string, RegExp[]> = {
    checkpoint: [/\bcheckpoint\b/i],
    dedupe: [/\b(dedupe|dedup)\b/i],
    idempotency: [/\b(idempoten|idempotency_key|idempotent)\b/i],
    retry_policy: [/\b(retry_policy|retry_count|ExponentialBackoff|retry\b)/i],
    replay_strategy: [/\breplay\b/i],
    conflict_strategy: [/\bconflict\b/i],
    compensation_strategy: [/\b(compensation|compensate|saga)\b/i],
  };

  const patterns = fallbackPatterns[strategy] ?? [];
  if (patterns.length > 0) {
    const deepResult = scanAllRustFiles(srcDir, patterns);
    if (deepResult) {
      return { present: true, evidence: `deep scan: ${deepResult}` };
    }
  }

  return { present: false, evidence: 'not found in module structure or source content' };
}

/** Check if a worker is planned-only (not yet implemented). */
function isPlannedWorker(worker: WorkerDecl): boolean {
  const note = worker.notes ?? '';
  const status = worker.status ?? '';
  return status === 'planned' || note.includes('尚未实现') || note.includes('占位');
}

// ── Main ─────────────────────────────────────────────────────────────────────

async function main(): Promise<number> {
  const { mode } = parseArgs(process.argv.slice(2));

  // Read codemap for worker must_have declarations
  interface CodemapYaml {
    modules: {
      workers: Record<string, {
        path: string;
        status?: string;
        notes?: string;
        must_have?: string[];
      }>;
    };
  }

  const codemap = await readYaml<CodemapYaml>('agent/codemap.yml');
  const workers = codemap.modules?.workers ?? {};

  console.log(`\n=== validate-resilience (${mode}) ===\n`);

  const issues: ValidationIssue[] = [];
  const checks: ResilienceCheck[] = [];

  for (const [workerName, worker] of Object.entries(workers)) {
    const workerPath = path.join(workspaceRoot, worker.path);
    const srcDir = path.join(workerPath, 'src');

    // Skip planned workers
    const decl: WorkerDecl = {
      path: worker.path,
      status: worker.status,
      notes: worker.notes,
      must_have: worker.must_have ?? [],
    };

    if (isPlannedWorker(decl)) {
      console.log(`  ⊘ ${workerName} (${worker.path}) — planned, skipping`);
      continue;
    }

    // Check if worker directory exists
    if (!existsSync(workerPath)) {
      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        worker: workerName,
        strategy: '(directory)',
        message: `worker directory does not exist: ${worker.path}`,
      });
      console.log(`  ✗ ${workerName} (${worker.path}) — directory missing`);
      continue;
    }

    // Check if src directory exists
    if (!existsSync(srcDir)) {
      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        worker: workerName,
        strategy: '(src)',
        message: `src directory missing for ${worker.path}`,
      });
      console.log(`  ✗ ${workerName} (${worker.path}) — src/ missing`);
      continue;
    }

    const mustHave = decl.must_have;
    if (mustHave.length === 0) {
      console.log(`  - ${workerName} (${worker.path}) — no must_have strategies declared`);
      continue;
    }

    console.log(`  → ${workerName} (${worker.path}) — checking ${mustHave.join(', ')}`);

    for (const strategy of mustHave) {
      const { present, evidence } = detectStrategy(srcDir, strategy);

      checks.push({
        workerName,
        workerPath: worker.path,
        strategy,
        present,
        evidence,
      });

      if (!present) {
        issues.push({
          level: mode === 'strict' ? 'error' : 'warn',
          worker: workerName,
          strategy,
          message: `missing required resilience strategy: ${strategy} — ${evidence}`,
        });
      }
    }
  }

  // Print detailed report
  console.log('\n--- Resilience Strategy Report ---\n');

  // Group by worker
  const byWorker = new Map<string, ResilienceCheck[]>();
  for (const check of checks) {
    const existing = byWorker.get(check.workerName) ?? [];
    existing.push(check);
    byWorker.set(check.workerName, existing);
  }

  for (const [workerName, workerChecks] of byWorker) {
    const allPresent = workerChecks.every((c) => c.present);
    const icon = allPresent ? '✓' : '✗';
    console.log(`  ${icon} ${workerName}:`);

    for (const check of workerChecks) {
      const statusIcon = check.present ? '  ✓' : '  ✗';
      console.log(`    ${statusIcon} ${check.strategy}: ${check.evidence}`);
    }
    console.log();
  }

  // Print issues
  if (issues.length === 0) {
    console.log('No resilience issues found');
    return 0;
  }

  console.log('--- Issues ---\n');

  for (const issue of issues) {
    const marker = issue.level === 'error' ? 'ERROR' : 'WARN';
    console.log(`  [${marker}] ${issue.worker}/${issue.strategy}: ${issue.message}`);
  }

  const errorCount = issues.filter((i) => i.level === 'error').length;
  const warnCount = issues.length - errorCount;
  console.log(`\nSummary: ${errorCount} error(s), ${warnCount} warning(s)`);

  return errorCount > 0 ? 1 : 0;
}

main()
  .then((code) => process.exit(code))
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(message);
    process.exit(1);
  });
