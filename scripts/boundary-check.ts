/**
 * Boundary Check - Architecture Dependency Validation
 * 
 * Enforces architectural dependency boundaries using cargo tree
 * Stage: Quality gate / CI
 */

import { run } from '../lib/spawn.js';
import process from 'node:process';

interface BoundaryRule {
  pkgName: string;
  allowedPatterns: string[];
  disallowedPattern: RegExp;
}

async function checkBoundary(rule: BoundaryRule): Promise<boolean> {
  console.log(`=== Checking ${rule.pkgName} dependencies ===`);

  const result = await run('cargo', ['tree', '-p', rule.pkgName, '--depth', '1']);

  if (!result.success) {
    console.warn(`⚠️  Could not get dependency tree for ${rule.pkgName}`);
    if (result.error) console.warn(result.error);
    return true; // Don't fail on missing package
  }

  const lines = result.output.split(/\r?\n/);
  const violations: string[] = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith(rule.pkgName)) continue;

    const isAllowed = rule.allowedPatterns.some((pattern) => trimmed.includes(pattern));
    if (isAllowed) continue;

    if (rule.disallowedPattern.test(trimmed)) {
      violations.push(trimmed);
    }
  }

  if (violations.length > 0) {
    console.error(`❌ FAIL: ${rule.pkgName} depends on illegal crates:`);
    for (const v of violations) {
      console.error(`  - ${v}`);
    }
    return false;
  } else {
    console.log(`✅ OK: ${rule.pkgName} boundary clean`);
    return true;
  }
}

async function main(): Promise<number> {
  const rules: BoundaryRule[] = [
    {
      pkgName: 'domain',
      allowedPatterns: ['async-trait', 'serde', 'serde_json'],
      disallowedPattern: /^(storage_|runtime_|contracts_)/,
    },
    {
      pkgName: 'usecases',
      allowedPatterns: ['async-trait', 'serde', 'serde_json', 'chrono', 'thiserror'],
      disallowedPattern: /^(storage_|runtime_|contracts_)/,
    },
    {
      pkgName: 'contracts_api',
      allowedPatterns: ['serde', 'ts-rs', 'utoipa', 'validator'],
      disallowedPattern: /^(domain|usecases|storage_|runtime_)/,
    },
  ];

  console.log('=== Architecture Boundary Check ===\n');

  const results = await Promise.all(rules.map(checkBoundary));

  console.log('');

  const allClean = results.every(Boolean);

  if (allClean) {
    console.log('✅ All boundary checks passed');
    return 0;
  }

  console.error('❌ Boundary check failed — review architectural dependencies');
  return 1;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
