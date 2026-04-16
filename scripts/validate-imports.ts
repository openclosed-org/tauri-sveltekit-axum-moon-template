import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

type Mode = 'warn' | 'strict';
type ModuleFamily = 'apps' | 'services' | 'servers' | 'workers' | 'packages' | 'platform';

interface ImportRule {
  name?: string;
  from: string | string[];
  allow?: string[];
  disallow?: string[];
  except_same_module?: boolean;
  except?: string[];  // Exceptions: paths that are allowed despite disallow rule
}

interface CodeMap {
  rules?: {
    imports?: ImportRule[];
  };
}

interface WorkspaceCargoToml {
  workspace?: {
    dependencies?: Record<string, { path?: string }>;
  };
}

interface ManifestInfo {
  relativePath: string;
  packageName: string;
  dependencies: string[];
}

interface ValidationIssue {
  level: 'warn' | 'error';
  scope: string;
  message: string;
}

const workspaceRoot = process.cwd();

function parseArgs(argv: string[]): { mode: Mode } {
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
    console.error('Usage: bun run scripts/validate-imports.ts [--mode warn|strict]');
    process.exit(1);
  }

  return { mode };
}

async function readYaml<T>(relativePath: string): Promise<T> {
  const fullPath = path.join(workspaceRoot, relativePath);
  const content = await Bun.file(fullPath).text();
  return Bun.YAML.parse(content) as T;
}

async function readToml<T>(relativePath: string): Promise<T> {
  const fullPath = path.join(workspaceRoot, relativePath);
  const content = await Bun.file(fullPath).text();
  return Bun.TOML.parse(content) as T;
}

async function loadWorkspaceDependencyPaths(): Promise<Map<string, string>> {
  const workspaceManifest = await readToml<WorkspaceCargoToml>('Cargo.toml');
  const dependencyMap = new Map<string, string>();

  for (const [dependencyName, value] of Object.entries(workspaceManifest.workspace?.dependencies ?? {})) {
    if (value && typeof value.path === 'string') {
      dependencyMap.set(dependencyName, value.path.replaceAll('\\', '/'));
    }
  }

  return dependencyMap;
}

function discoverCargoManifests(): string[] {
  const roots = ['apps', 'packages', 'platform', 'servers', 'services', 'workers'];
  const manifests: string[] = [];

  for (const root of roots) {
    const absoluteRoot = path.join(workspaceRoot, root);
    if (!existsSync(absoluteRoot)) {
      continue;
    }

    const stack = [absoluteRoot];
    while (stack.length > 0) {
      const current = stack.pop();
      if (!current) continue;

      const entries = readdirSync(current, { withFileTypes: true });
      for (const entry of entries) {
        const entryPath = path.join(current, entry.name);
        if (entry.isDirectory()) {
          stack.push(entryPath);
          continue;
        }

        if (entry.isFile() && entry.name === 'Cargo.toml') {
          manifests.push(path.relative(workspaceRoot, entryPath));
        }
      }
    }
  }

  return manifests.sort();
}

function determineFamily(relativePath: string): ModuleFamily | null {
  const [topLevel] = relativePath.split(path.sep);
  if (
    topLevel === 'apps' ||
    topLevel === 'services' ||
    topLevel === 'servers' ||
    topLevel === 'workers' ||
    topLevel === 'packages' ||
    topLevel === 'platform'
  ) {
    return topLevel;
  }

  return null;
}

function matchesPattern(target: string, pattern: string): boolean {
  if (pattern === `${target}/**`) {
    return true;
  }

  if (pattern.endsWith('/**')) {
    const base = pattern.slice(0, -3);
    return target === base || target.startsWith(`${base}/`);
  }

  if (pattern.endsWith('/*')) {
    const base = pattern.slice(0, -2);
    if (!target.startsWith(`${base}/`)) {
      return false;
    }

    const rest = target.slice(base.length + 1);
    return rest.length > 0 && !rest.includes('/');
  }

  return target === pattern;
}

function manifestPathFromCrate(cratePath: string): string {
  return path.relative(workspaceRoot, cratePath).replaceAll('\\', '/');
}

async function loadManifestInfo(relativeManifestPath: string): Promise<ManifestInfo | null> {
  const workspaceDependencyPaths = await loadWorkspaceDependencyPaths();
  const manifest = await readToml<Record<string, Record<string, unknown>>>(relativeManifestPath);
  const packageName = manifest.package?.name;
  if (typeof packageName !== 'string' || packageName.length === 0) {
    return null;
  }

  const dependencySections = ['dependencies', 'dev-dependencies', 'build-dependencies'];
  const dependencies: string[] = [];

  for (const section of dependencySections) {
    const sectionValue = manifest[section];
    if (!sectionValue || typeof sectionValue !== 'object' || Array.isArray(sectionValue)) {
      continue;
    }

    for (const [dependencyName, value] of Object.entries(sectionValue)) {
      if (!value || typeof value !== 'object' || Array.isArray(value)) {
        continue;
      }

      const cratePath = (value as { path?: string }).path;
      if (typeof cratePath !== 'string') {
        if ((value as { workspace?: boolean }).workspace === true) {
          const workspacePath = workspaceDependencyPaths.get(dependencyName);
          if (workspacePath) {
            dependencies.push(workspacePath);
          }
        }

        continue;
      }

      const absoluteDependencyPath = path.resolve(workspaceRoot, path.dirname(relativeManifestPath), cratePath);
      const normalizedPath = manifestPathFromCrate(absoluteDependencyPath);
      dependencies.push(normalizedPath);
    }
  }

  return {
    relativePath: path.dirname(relativeManifestPath),
    packageName,
    dependencies: [...new Set(dependencies)].sort(),
  };
}

function applicableRules(rules: ImportRule[], familyPath: string): ImportRule[] {
  return rules.filter((rule) => {
    const fromPatterns = Array.isArray(rule.from) ? rule.from : [rule.from];
    return fromPatterns.some((pattern) => matchesPattern(familyPath, pattern));
  });
}

function isSameServiceModule(source: string, target: string): boolean {
  const sourceParts = source.split('/');
  const targetParts = target.split('/');
  return sourceParts[0] === targetParts[0] && sourceParts[1] === targetParts[1];
}

function isException(sourcePath: string, targetPath: string, exceptions: string[] | undefined): boolean {
  if (!exceptions || exceptions.length === 0) {
    return false;
  }
  
  // Check if source path matches any exception pattern
  return exceptions.some((pattern) => {
    // Pattern format: "source:target" or just "source" (any target)
    const [exceptionSource, exceptionTarget] = pattern.includes(':') 
      ? pattern.split(':') 
      : [pattern, '*'];
    
    const sourceMatches = matchesPattern(sourcePath, exceptionSource);
    const targetMatches = exceptionTarget === '*' || matchesPattern(targetPath, exceptionTarget);
    
    return sourceMatches && targetMatches;
  });
}

function validateManifestAgainstRules(
  manifest: ManifestInfo,
  rules: ImportRule[],
  mode: Mode,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const sourcePath = manifest.relativePath.replaceAll('\\', '/');
  const family = determineFamily(sourcePath);
  if (!family) {
    return issues;
  }

  const matchedRules = applicableRules(rules, `${family}/**`);

  for (const dependencyPath of manifest.dependencies) {
    const targetPath = dependencyPath.replaceAll('\\', '/');

    for (const rule of matchedRules) {
      if (rule.except_same_module && isSameServiceModule(sourcePath, targetPath)) {
        continue;
      }

      // Check if this is an exception
      if (isException(sourcePath, targetPath, rule.except)) {
        continue;
      }

      const disallowMatch = (rule.disallow ?? []).find((pattern) => matchesPattern(targetPath, pattern));
      if (disallowMatch) {
        issues.push({
          level: mode === 'strict' ? 'error' : 'warn',
          scope: sourcePath,
          message: `depends on forbidden path ${targetPath} (rule: ${rule.name ?? 'unnamed'})`,
        });
        continue;
      }

      if (rule.allow && rule.allow.length > 0) {
        const sameFamily = targetPath.startsWith(`${family}/`);
        const isAllowed = sameFamily || rule.allow.some((pattern) => matchesPattern(targetPath, pattern));
        if (!isAllowed) {
          issues.push({
            level: mode === 'strict' ? 'error' : 'warn',
            scope: sourcePath,
            message: `depends on path outside allowlist: ${targetPath} (rule: ${rule.name ?? 'unnamed'})`,
          });
        }
      }
    }
  }

  return issues;
}

async function main(): Promise<number> {
  const { mode } = parseArgs(process.argv.slice(2));
  const codemap = await readYaml<CodeMap>('agent/codemap.yml');
  const rules = codemap.rules?.imports ?? [];

  const manifests = discoverCargoManifests();
  const manifestInfos = (await Promise.all(manifests.map(loadManifestInfo))).filter(
    (item): item is ManifestInfo => item !== null,
  );

  const issues = manifestInfos.flatMap((manifest) => validateManifestAgainstRules(manifest, rules, mode));

  console.log(`=== validate-imports (${mode}) ===`);

  if (issues.length === 0) {
    console.log('No import rule issues found');
    return 0;
  }

  for (const issue of issues) {
    const marker = issue.level === 'error' ? 'ERROR' : 'WARN';
    console.log(`[${marker}] ${issue.scope}: ${issue.message}`);
  }

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
