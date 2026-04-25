import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

type Mode = 'warn' | 'strict';

interface ValidationIssue {
  level: 'warn' | 'error';
  scope: string;
  message: string;
}

const workspaceRoot = process.cwd();
const servicesRoot = path.join(workspaceRoot, 'services');

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
    console.error('Usage: bun run scripts/validate-contract-boundaries.ts [--mode warn|strict]');
    process.exit(1);
  }

  return { mode };
}

function listRustFiles(dir: string): string[] {
  const files: string[] = [];
  const stack = [dir];

  while (stack.length > 0) {
    const current = stack.pop();
    if (!current || !existsSync(current)) {
      continue;
    }

    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const entryPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === 'target') {
          continue;
        }
        stack.push(entryPath);
        continue;
      }

      if (entry.isFile() && entry.name.endsWith('.rs')) {
        files.push(entryPath);
      }
    }
  }

  return files.sort();
}

function stripRustComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/^\s*\/\/.*$/gm, '');
}

async function serviceUsesSharedAppEvent(serviceSrcDir: string): Promise<boolean> {
  const rustFiles = listRustFiles(serviceSrcDir);
  for (const filePath of rustFiles) {
    const content = await Bun.file(filePath).text();
    const code = stripRustComments(content);
    if (/contracts_events::AppEvent|use\s+contracts_events::\{[^}]*AppEvent/.test(code)) {
      return true;
    }
  }

  return false;
}

async function findIssuesForFile(filePath: string, serviceHasSharedAppEvent: boolean): Promise<ValidationIssue[]> {
  const relativePath = path.relative(workspaceRoot, filePath).replaceAll('\\', '/');
  const source = await Bun.file(filePath).text();
  const code = stripRustComments(source);
  const issues: ValidationIssue[] = [];

  const hasServiceLocalEventType = /pub\s+enum\s+\w+Event|pub\s+struct\s+\w+Event/.test(code);
  const writesOutbox = /INSERT\s+INTO\s+event_outbox|UPDATE\s+event_outbox/.test(code);

  if (hasServiceLocalEventType && writesOutbox) {
    issues.push({
      level: 'error',
      scope: relativePath,
      message:
        'service-local event definitions must not live in files that write to event_outbox; promote cross-boundary events to contracts_events::AppEvent first',
    });
  }

  if (writesOutbox && !serviceHasSharedAppEvent) {
    issues.push({
      level: 'error',
      scope: relativePath,
      message:
        'services writing to event_outbox must define the cross-process payload via contracts_events::AppEvent somewhere in the same service crate',
    });
  }

  return issues;
}

async function main(): Promise<number> {
  const { mode } = parseArgs(process.argv.slice(2));

  if (!existsSync(servicesRoot)) {
    console.log('No services directory found; skipping contract boundary validation');
    return 0;
  }

  const serviceDirectories = readdirSync(servicesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(servicesRoot, entry.name, 'src'));

  const issueLists: ValidationIssue[][] = [];
  for (const serviceSrcDir of serviceDirectories) {
    const serviceHasSharedAppEvent = await serviceUsesSharedAppEvent(serviceSrcDir);
    const rustFiles = listRustFiles(serviceSrcDir);
    issueLists.push(
      ...(await Promise.all(
        rustFiles.map((filePath) => findIssuesForFile(filePath, serviceHasSharedAppEvent)),
      )),
    );
  }
  const issues = issueLists.flat();

  console.log('=== Contract Boundary Validation ===');

  if (issues.length === 0) {
    console.log('✓ shared contract boundaries clean');
    return 0;
  }

  for (const issue of issues) {
    const prefix = issue.level === 'error' ? 'ERROR' : 'WARN';
    console.log(`${prefix}: ${issue.scope} - ${issue.message}`);
  }

  const hasErrors = issues.some((issue) => issue.level === 'error');
  if (hasErrors && mode === 'strict') {
    console.error('contract boundary validation failed');
    return 1;
  }

  if (hasErrors) {
    console.warn('contract boundary validation completed with warnings');
  }

  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(message);
    process.exit(1);
  });
