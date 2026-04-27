/**
 * Validate Contracts — Verify server handlers align with contracts.
 *
 * This script performs the following checks:
 * 1. Scans packages/contracts/ for API DTOs and event types
 * 2. Scans servers/ for handler implementations
 * 3. Verifies that server Cargo.toml files depend on the contract crates they use
 * 4. Checks that openapi.yaml files are not empty when handlers exist
 * 5. Reports any drift between contract definitions and server implementations
 *
 * Usage:
 *   bun run scripts/validate-contracts.ts [--mode warn|strict]
 */

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

type Mode = 'warn' | 'strict';

interface ValidationIssue {
  level: 'warn' | 'error';
  scope: string;
  message: string;
}

interface ContractCrate {
  name: string;
  path: string;
  exportedTypes: string[];
}

interface ServerModule {
  name: string;
  path: string;
  hasCargoToml: boolean;
  hasOpenApi: boolean;
  hasRuntimeOpenApi: boolean;
  hasHandlers: boolean;
  hasRoutes: boolean;
  contractDependencies: string[];
}

type ContractUsageMap = Map<string, string[]>;

interface ParsedArgs {
  mode: Mode;
}

const workspaceRoot = process.cwd();
const contractsDir = path.join(workspaceRoot, 'packages', 'contracts');
const serversDir = path.join(workspaceRoot, 'servers');

// ── Argument Parsing ──────────────────────────────────────────

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
    console.error('Usage: bun run scripts/validate-contracts.ts [--mode warn|strict]');
    process.exit(1);
  }

  return { mode };
}

// ── Contract Discovery ────────────────────────────────────────

function discoverContractCrates(): ContractCrate[] {
  const crates: ContractCrate[] = [];

  if (!existsSync(contractsDir)) {
    return crates;
  }

  const entries = readdirSync(contractsDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory() || entry.name === 'bindings' || entry.name === 'generated') {
      continue;
    }

    const cratePath = path.join(contractsDir, entry.name);
    const cargoToml = path.join(cratePath, 'Cargo.toml');
    const srcLib = path.join(cratePath, 'src', 'lib.rs');

    if (!existsSync(cargoToml) || !existsSync(srcLib)) {
      continue;
    }

    const exportedTypes = extractExportedTypes(srcLib);
    const packageName = extractPackageName(cargoToml);

    crates.push({
      name: packageName || entry.name,
      path: path.relative(workspaceRoot, cratePath),
      exportedTypes,
    });
  }

  return crates.sort((a, b) => a.name.localeCompare(b.name));
}

function extractExportedTypes(libRsPath: string): string[] {
  try {
    const content = readFileSync(libRsPath, 'utf-8');
    const types: string[] = [];

    // Match struct and enum definitions with #[derive(...ToSchema...)] or #[ts(export)]
    const structRegex = /pub\s+(struct|enum)\s+(\w+)/g;
    let match: RegExpExecArray | null;

    while ((match = structRegex.exec(content)) !== null) {
      const typeName = match[2];
      // Check if this type is exported for TS generation
      const contextStart = Math.max(0, match.index - 200);
      const context = content.slice(contextStart, match.index);

      // Check for ToSchema or ts(export) attribute nearby
      if (context.includes('ToSchema') || context.includes('ts(export)') || context.includes('#[ts(')) {
        types.push(typeName);
      }
    }

    return types.sort();
  } catch {
    return [];
  }
}

function extractPackageName(cargoTomlPath: string): string | null {
  try {
    const content = readFileSync(cargoTomlPath, 'utf-8');
    const nameMatch = content.match(/^name\s*=\s*"([^"]+)"/m);
    return nameMatch?.[1] ?? null;
  } catch {
    return null;
  }
}

// ── Server Discovery ──────────────────────────────────────────

function discoverServerModules(): ServerModule[] {
  const modules: ServerModule[] = [];

  if (!existsSync(serversDir)) {
    return modules;
  }

  // Recursively find server modules (directories with Cargo.toml)
  const stack = [serversDir];

  // Check if a server has runtime OpenAPI (e.g., /scalar endpoint)
  function checkRuntimeOpenApi(serverPath: string): boolean {
    const libRsPath = path.join(serverPath, 'src', 'lib.rs');
    const mainRsPath = path.join(serverPath, 'src', 'main.rs');
    
    for (const filePath of [libRsPath, mainRsPath]) {
      if (!existsSync(filePath)) continue;
      try {
        const content = readFileSync(filePath, 'utf-8');
        // Check for utoipa Scalar integration or /scalar route
        if (content.includes('utoipa_scalar') || content.includes('Scalar::new') || content.includes('"/scalar"')) {
          return true;
        }
      } catch {
        // Skip files that can't be read
      }
    }
    return false;
  }

  // Check if handlers directory has real handlers (not just .gitkeep)
  function hasRealHandlers(handlersDir: string): boolean {
    if (!existsSync(handlersDir)) return false;
    const entries = readdirSync(handlersDir, { withFileTypes: true });
    return entries.some(entry => {
      if (!entry.isFile()) return false;
      if (entry.name === '.gitkeep') return false;
      return entry.name.endsWith('.rs');
    });
  }

  while (stack.length > 0) {
    const current = stack.pop();
    if (!current) continue;

    const entries = readdirSync(current, { withFileTypes: true });
    let hasCargoToml = false;
    let hasStaticOpenApi = false;
    let hasHandlers = false;
    let hasRoutes = false;
    let contractDependencies: string[] = [];

    for (const entry of entries) {
      const entryPath = path.join(current, entry.name);

      if (entry.isFile()) {
        // Skip .gitkeep files
        if (entry.name === '.gitkeep') continue;
        if (entry.name === 'Cargo.toml') {
          hasCargoToml = true;
          contractDependencies = extractContractDependencies(entryPath);
        }
        if (entry.name === 'openapi.yaml' || entry.name === 'openapi.yml') {
          hasStaticOpenApi = true;
        }
      }

      if (entry.isDirectory()) {
        if (entry.name === 'handlers' || entry.name === 'src') {
          const handlersPath = path.join(current, 'handlers');
          const srcHandlersPath = path.join(current, 'src', 'handlers');
          if (hasRealHandlers(handlersPath) || hasRealHandlers(srcHandlersPath)) {
            hasHandlers = true;
          }
        }
        if (entry.name === 'routes' || entry.name === 'src') {
          const routesPath = path.join(current, 'routes');
          const srcRoutesPath = path.join(current, 'src', 'routes');
          if (existsSync(routesPath) || existsSync(srcRoutesPath)) {
            hasRoutes = true;
          }
        }
        // Recurse into subdirectories
        if (entry.name !== 'target' && entry.name !== 'node_modules') {
          stack.push(entryPath);
        }
      }
    }

    // Check for runtime OpenAPI
    const hasRuntimeOpenApi = checkRuntimeOpenApi(current);
    const hasOpenApi = hasStaticOpenApi || hasRuntimeOpenApi;

    if (hasCargoToml) {
      const relativePath = path.relative(workspaceRoot, current);
      const serverName = path.basename(current);

      modules.push({
        name: serverName,
        path: relativePath,
        hasCargoToml,
        hasOpenApi,
        hasRuntimeOpenApi,
        hasHandlers,
        hasRoutes,
        contractDependencies,
      });
    }
  }

  return modules.sort((a, b) => a.name.localeCompare(b.name));
}

function extractContractDependencies(cargoTomlPath: string): string[] {
  try {
    const content = readFileSync(cargoTomlPath, 'utf-8');
    const deps: string[] = [];

    // Look for contract dependencies (contracts_*, contracts-*)
    const lines = content.split('\n');
    let inDependencies = false;

    for (const line of lines) {
      // Detect dependency sections
      if (line.trim() === '[dependencies]' || line.trim() === '[dev-dependencies]') {
        inDependencies = true;
        continue;
      }

      if (line.trim().startsWith('[') && inDependencies) {
        inDependencies = false;
        continue;
      }

      if (inDependencies) {
        // Match contract dependencies (contracts_api, contracts_auth, etc.)
        const depMatch = line.match(/^\s*(contracts_\w+)\s*=/);
        if (depMatch) {
          deps.push(depMatch[1]);
        }
      }
    }

    return deps.sort();
  } catch {
    return [];
  }
}

function discoverWorkspaceContractUsage(): ContractUsageMap {
  const usages = new Map<string, Set<string>>();
  const roots = [
    path.join(workspaceRoot, 'packages'),
    path.join(workspaceRoot, 'services'),
    path.join(workspaceRoot, 'servers'),
    path.join(workspaceRoot, 'workers'),
    path.join(workspaceRoot, 'platform'),
  ];

  const visit = (currentPath: string): void => {
    if (!existsSync(currentPath)) {
      return;
    }

    const entries = readdirSync(currentPath, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name === 'target' || entry.name === 'node_modules' || entry.name === '.git') {
        continue;
      }

      const entryPath = path.join(currentPath, entry.name);
      if (entry.isDirectory()) {
        visit(entryPath);
        continue;
      }

      if (entry.isFile() && entry.name === 'Cargo.toml' && !entryPath.startsWith(contractsDir)) {
        const consumer = path.relative(workspaceRoot, path.dirname(entryPath));
        const deps = extractContractDependencies(entryPath);
        for (const dep of deps) {
          const consumers = usages.get(dep) ?? new Set<string>();
          consumers.add(consumer);
          usages.set(dep, consumers);
        }
      }
    }
  };

  for (const root of roots) {
    visit(root);
  }

  return new Map(
    Array.from(usages.entries(), ([dep, consumers]) => [dep, Array.from(consumers).sort()]),
  );
}

// ── Validation Logic ──────────────────────────────────────────

function validateContractCoverage(
  contractCrates: ContractCrate[],
  serverModules: ServerModule[],
  workspaceUsage: ContractUsageMap,
  mode: Mode,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  // Check that each contract crate is used by at least one server
  for (const contract of contractCrates) {
    const usedByServers = serverModules.filter((s) => s.contractDependencies.includes(contract.name));

    if (usedByServers.length === 0) {
      const workspaceConsumers = workspaceUsage.get(contract.name) ?? [];
      if (workspaceConsumers.length > 0) {
        continue;
      }

      // Not an error — contracts can be used by future servers or external clients
      issues.push({
        level: 'warn',
        scope: contract.path,
        message: `contract crate '${contract.name}' is not directly depended on by any workspace crate`,
      });
    }
  }

  // Check that servers with handlers have openapi.yaml or runtime OpenAPI
  for (const server of serverModules) {
    if (server.hasHandlers && !server.hasOpenApi) {
      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        scope: server.path,
        message: 'has handlers but missing openapi documentation (static or runtime via /scalar)',
      });
    }

    // Check that servers with handlers reference contract types
    if (server.hasHandlers && server.contractDependencies.length === 0) {
      issues.push({
        level: 'warn',
        scope: server.path,
        message: 'has handlers but does not depend on any contracts_* crate — verify this is intentional',
      });
    }
  }

  return issues;
}

function validateOpenApiAlignment(
  serverModules: ServerModule[],
  mode: Mode,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  for (const server of serverModules) {
    // Skip servers with runtime OpenAPI (they generate docs dynamically)
    if (server.hasRuntimeOpenApi) {
      continue;
    }

    // Skip servers without static openapi.yaml
    if (!server.hasOpenApi) {
      continue;
    }

    const openApiPath = path.join(workspaceRoot, server.path, 'openapi.yaml');
    try {
      const content = readFileSync(openApiPath, 'utf-8');

      // Check if paths section is empty
      if (content.includes('paths: {}') || content.includes('paths:{}')) {
        if (server.hasHandlers) {
          issues.push({
            level: 'warn',
            scope: server.path,
            message: 'openapi.yaml has empty paths section but handlers exist — consider documenting endpoints',
          });
        }
      }
    } catch {
      // File exists but can't be read
      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        scope: server.path,
        message: 'openapi.yaml exists but could not be read',
      });
    }
  }

  return issues;
}

function validateTypeUsage(
  contractCrates: ContractCrate[],
  serverModules: ServerModule[],
  mode: Mode,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  // Build a map of contract types to their crates
  const typeToCrate = new Map<string, string>();
  for (const crate_ of contractCrates) {
    for (const type_ of crate_.exportedTypes) {
      typeToCrate.set(type_, crate_.name);
    }
  }

  // Check server handler files for contract type usage
  for (const server of serverModules) {
    if (!server.hasHandlers) {
      continue;
    }

    const handlersDir = path.join(workspaceRoot, server.path, 'handlers');
    const srcHandlersDir = path.join(workspaceRoot, server.path, 'src', 'handlers');
    const handlerDirs = [handlersDir, srcHandlersDir].filter(existsSync);

    if (handlerDirs.length === 0) {
      continue;
    }

    // Check if server declares contract dependencies but doesn't use any types
    const hasContractDeps = server.contractDependencies.length > 0;
    let usesContractTypes = false;

    for (const handlerDir of handlerDirs) {
      const handlerEntries = readdirSync(handlerDir, { withFileTypes: true });
      for (const entry of handlerEntries) {
        if (!entry.isFile() || !entry.name.endsWith('.rs')) {
          continue;
        }

        const handlerPath = path.join(handlerDir, entry.name);
        try {
          const content = readFileSync(handlerPath, 'utf-8');

          // Check for use of contract types
          for (const [typeName, _crateName] of typeToCrate.entries()) {
            if (content.includes(typeName)) {
              usesContractTypes = true;
              break;
            }
          }
        } catch {
          // Skip files that can't be read
        }

        if (usesContractTypes) {
          break;
        }
      }

      if (usesContractTypes) {
        break;
      }
    }

    if (hasContractDeps && !usesContractTypes) {
      issues.push({
        level: 'warn',
        scope: server.path,
        message: `depends on contracts but may not use exported types: ${server.contractDependencies.join(', ')}`,
      });
    }
  }

  return issues;
}

// ── Output ────────────────────────────────────────────────────

function printIssues(issues: ValidationIssue[]): void {
  if (issues.length === 0) {
    return;
  }

  for (const issue of issues) {
    const marker = issue.level === 'error' ? 'ERROR' : 'WARN';
    console.log(`[${marker}] ${issue.scope}: ${issue.message}`);
  }
}

function printSummary(
  contractCrates: ContractCrate[],
  serverModules: ServerModule[],
  issues: ValidationIssue[],
): void {
  console.log('\n--- Contract-Coverage Summary ---');
  console.log(`Contract crates discovered: ${contractCrates.length}`);
  console.log(`Server modules discovered: ${serverModules.length}`);

  if (contractCrates.length > 0) {
    console.log('\nContract crates:');
    for (const crate_ of contractCrates) {
      console.log(`  - ${crate_.name} (${crate_.exportedTypes.length} exported types)`);
    }
  }

  if (serverModules.length > 0) {
    console.log('\nServer modules:');
    for (const server of serverModules) {
      const deps = server.contractDependencies.length > 0
        ? ` [contracts: ${server.contractDependencies.join(', ')}]`
        : '';
      const flags = [
        server.hasHandlers ? 'handlers' : null,
        server.hasRoutes ? 'routes' : null,
        server.hasOpenApi ? (server.hasRuntimeOpenApi ? 'runtime-openapi' : 'static-openapi') : null,
      ].filter(Boolean).join(', ');

      console.log(`  - ${server.name} [${flags}]${deps}`);
    }
  }

  const errorCount = issues.filter((i) => i.level === 'error').length;
  const warnCount = issues.length - errorCount;
  console.log(`\nContract issues: ${errorCount} error(s), ${warnCount} warning(s)`);
}

// ── Main ──────────────────────────────────────────────────────

async function main(): Promise<number> {
  const { mode } = parseArgs(process.argv.slice(2));

  console.log('=== validate-contracts ===');

  // Discover contract crates
  const contractCrates = discoverContractCrates();

  // Discover server modules
  const serverModules = discoverServerModules();
  const workspaceUsage = discoverWorkspaceContractUsage();

  // Run validation checks
  const issues: ValidationIssue[] = [
    ...validateContractCoverage(contractCrates, serverModules, workspaceUsage, mode),
    ...validateOpenApiAlignment(serverModules, mode),
    ...validateTypeUsage(contractCrates, serverModules, mode),
  ];

  // Print results
  printIssues(issues);
  printSummary(contractCrates, serverModules, issues);

  if (issues.length === 0) {
    console.log('\nNo contract issues found');
    return 0;
  }

  const errorCount = issues.filter((i) => i.level === 'error').length;
  return errorCount > 0 ? 1 : 0;
}

main()
  .then((code) => process.exit(code))
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(message);
    process.exit(1);
  });
