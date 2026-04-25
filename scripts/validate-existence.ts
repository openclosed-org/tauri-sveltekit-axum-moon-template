import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

type Mode = 'warn' | 'strict';
type ModuleKind = 'service' | 'server' | 'worker';

interface CodeMapRuleSet {
  rules?: {
    required_files?: Record<string, string[]>;
  };
  modules?: Record<string, Record<string, { path?: string; notes?: string; status?: string }>>;
  reference_modules?: Record<string, Record<string, { path?: string; notes?: string; status?: string }>>;
}

interface DeclaredModule {
  kind: ModuleKind;
  notes?: string;
  status?: string;
}

interface ValidationIssue {
  level: 'warn' | 'error';
  scope: string;
  message: string;
}

interface ParsedArgs {
  mode: Mode;
}

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
    console.error('Usage: bun run scripts/validate-existence.ts [--mode warn|strict]');
    process.exit(1);
  }

  return { mode };
}

async function readYaml<T>(relativePath: string): Promise<T> {
  const fullPath = path.join(workspaceRoot, relativePath);
  const content = await Bun.file(fullPath).text();
  return Bun.YAML.parse(content) as T;
}

function normalizeRequiredEntry(modulePath: string, entry: string): string {
  if (entry.includes('<name>')) {
    return entry.replace('<name>', path.basename(modulePath));
  }

  return path.join(modulePath, entry);
}

function pathExists(relativePath: string): boolean {
  return existsSync(path.join(workspaceRoot, relativePath));
}

function collectDeclaredModules(codemap: CodeMapRuleSet): Map<string, DeclaredModule> {
  const declared = new Map<string, DeclaredModule>();

  for (const [section, items] of Object.entries(codemap.modules ?? codemap.reference_modules ?? {})) {
    const kind = section.slice(0, -1) as ModuleKind;
    if (kind !== 'service' && kind !== 'server' && kind !== 'worker') {
      continue;
    }

    for (const item of Object.values(items)) {
      if (item.path) {
        declared.set(item.path, {
          kind,
          notes: item.notes,
          status: item.status,
        });
      }
    }
  }

  return declared;
}

function isPlannedOnlyModule(module: DeclaredModule): boolean {
  const note = module.notes ?? '';
  const status = module.status ?? '';
  return (
    status === 'planned' ||
    note.includes('尚未实现') ||
    note.includes('占位') ||
    note.includes('仅保留语义边界')
  );
}

function discoverActualModules(kind: ModuleKind): string[] {
  const baseDir = path.join(workspaceRoot, `${kind}s`);
  if (!existsSync(baseDir)) {
    return [];
  }

  const discovered = new Set<string>();
  const stack = [baseDir];

  while (stack.length > 0) {
    const current = stack.pop();
    if (!current) continue;

    const entries = readdirSync(current, { withFileTypes: true });
    let hasManifest = false;

    for (const entry of entries) {
      if (entry.isFile() && entry.name === 'Cargo.toml') {
        hasManifest = true;
      }
    }

    if (hasManifest) {
      discovered.add(path.relative(workspaceRoot, current));
      continue;
    }

    for (const entry of entries) {
      if (entry.isDirectory()) {
        stack.push(path.join(current, entry.name));
      }
    }
  }

  return [...discovered].sort();
}

function validateDeclaredModules(
  codemap: CodeMapRuleSet,
  declaredModules: Map<string, DeclaredModule>,
  mode: Mode,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const requiredFiles = codemap.rules?.required_files ?? {};

  for (const [modulePath, module] of declaredModules.entries()) {
    if (isPlannedOnlyModule(module)) {
      continue;
    }

    if (!pathExists(modulePath)) {
      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        scope: modulePath,
        message: 'declared in codemap but directory does not exist',
      });
      continue;
    }

    const entries = requiredFiles[module.kind] ?? [];
    for (const entry of entries) {
      const target = normalizeRequiredEntry(modulePath, entry);
      if (pathExists(target)) {
        continue;
      }

      issues.push({
        level: mode === 'strict' ? 'error' : 'warn',
        scope: modulePath,
        message: `missing required path: ${entry}`,
      });
    }
  }

  return issues;
}

function validateUndeclaredModules(declaredModules: Map<string, DeclaredModule>): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  for (const kind of ['service', 'server', 'worker'] as const) {
    const actualModules = discoverActualModules(kind);
    for (const modulePath of actualModules) {
      if (declaredModules.has(modulePath)) {
        continue;
      }

      issues.push({
        level: 'warn',
        scope: modulePath,
        message: 'exists in repository but is not declared in agent/codemap.yml',
      });
    }
  }

  return issues;
}

function printIssues(issues: ValidationIssue[]): void {
  for (const issue of issues) {
    const marker = issue.level === 'error' ? 'ERROR' : 'WARN';
    console.log(`[${marker}] ${issue.scope}: ${issue.message}`);
  }
}

async function main(): Promise<number> {
  const { mode } = parseArgs(process.argv.slice(2));
  const codemap = await readYaml<CodeMapRuleSet>('agent/codemap.yml');
  const declaredModules = collectDeclaredModules(codemap);

  const issues = [
    ...validateDeclaredModules(codemap, declaredModules, mode),
    ...validateUndeclaredModules(declaredModules),
  ];

  console.log(`=== validate-existence (${mode}) ===`);

  if (issues.length === 0) {
    console.log('No existence issues found');
    return 0;
  }

  printIssues(issues);

  const errorCount = issues.filter((issue) => issue.level === 'error').length;
  const warnCount = issues.length - errorCount;
  console.log(`Summary: ${errorCount} error(s), ${warnCount} warning(s)`);

  return errorCount > 0 ? 1 : 0;
}

main()
  .then((code) => process.exit(code))
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(message);
    process.exit(1);
  });
