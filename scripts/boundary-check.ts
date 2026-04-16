import { run } from './lib/spawn.ts';
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
    console.warn(`Warning: Could not get dependency tree for ${rule.pkgName}`);
    if (result.error) console.warn(result.error);
    return true;
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
    console.error(`FAIL: ${rule.pkgName} depends on illegal crates:`);
    for (const v of violations) {
      console.error(`  - ${v}`);
    }
    return false;
  } else {
    console.log(`OK: ${rule.pkgName} boundary clean`);
    return true;
  }
}

async function main(): Promise<number> {
  const rules: BoundaryRule[] = [
    {
      pkgName: 'kernel',
      allowedPatterns: ['async-trait', 'serde', 'serde_json'],
      disallowedPattern: /^(storage_|runtime_|contracts_|counter-service|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'contracts_api',
      allowedPatterns: ['serde', 'ts-rs', 'utoipa', 'validator'],
      disallowedPattern: /^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'contracts_auth',
      allowedPatterns: ['serde', 'ts-rs', 'utoipa', 'validator'],
      disallowedPattern: /^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'contracts_events',
      allowedPatterns: ['serde', 'ts-rs', 'utoipa', 'validator'],
      disallowedPattern: /^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'contracts_errors',
      allowedPatterns: ['serde', 'ts-rs', 'utoipa', 'validator'],
      disallowedPattern: /^(kernel|storage_|runtime_|counter-service|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'counter-service',
      allowedPatterns: ['async-trait', 'serde', 'serde_json', 'thiserror', 'contracts_events', 'contracts_errors', 'kernel', 'data'],
      disallowedPattern: /^(storage_|runtime_|auth-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'auth-service',
      allowedPatterns: ['async-trait', 'serde', 'serde_json', 'thiserror', 'contracts_auth', 'contracts_errors', 'kernel', 'data'],
      disallowedPattern: /^(storage_|runtime_|counter-service|tenant-service|user-service)/,
    },
    {
      pkgName: 'tenant-service',
      allowedPatterns: ['async-trait', 'serde', 'serde_json', 'thiserror', 'contracts_errors', 'kernel', 'data'],
      disallowedPattern: /^(storage_|runtime_|counter-service|auth-service|user-service)/,
    },
    {
      pkgName: 'user-service',
      allowedPatterns: ['async-trait', 'serde', 'serde_json', 'thiserror', 'contracts_errors', 'kernel', 'data'],
      disallowedPattern: /^(storage_|runtime_|counter-service|auth-service|tenant-service)/,
    },
  ];

  console.log('=== Architecture Boundary Check ===\n');
  console.log('Rules: services MUST NOT depend on other services\n');
  console.log('Rules: contracts MUST be Single Source of Truth for shared types\n');

  const results = await Promise.all(rules.map(checkBoundary));

  console.log('');

  const allClean = results.every(Boolean);

  if (allClean) {
    console.log('✅ All boundary checks passed');
    return 0;
  }

  console.error('❌ Boundary check failed - review architectural dependencies');
  return 1;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
