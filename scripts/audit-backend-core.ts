import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

type Mode = 'dry-run' | 'strict';

interface Finding {
  file: string;
  needle: string;
  context: string;
}

const workspaceRoot = process.cwd();

const backendEntryFiles = [
  'package.json',
  '.moon/workspace.yml',
  'moon.yml',
  'justfile',
  'justfiles/setup.just',
  'justfiles/build.just',
  'justfiles/dev.just',
  'justfiles/deploy.just',
  'justfiles/processes.just',
  'justfiles/test.just',
  'justfiles/quality.just',
  'justfiles/platform.just',
  'justfiles/sops.just',
  'justfiles/template.just',
  'scripts/typegen.ts',
  'scripts/run-scoped-gates.ts',
  'scripts/verify-handoff.ts',
  'scripts/validate-contracts.ts',
];

const forbiddenNeedles = [
  'apps/web',
  'apps/desktop',
  'apps/mobile',
  'apps/browser-extension',
  'packages/ui',
  'web:',
  'desktop-tauri:',
  'apps/client',
];

function parseMode(argv: string[]): Mode {
  const arg = argv[0] ?? 'dry-run';
  if (arg === 'dry-run' || arg === 'strict') {
    return arg;
  }
  console.error('Usage: bun run scripts/audit-backend-core.ts [dry-run|strict]');
  process.exit(1);
}

function currentSection(line: string, previous: string): string {
  const match = line.match(/^\s{2}([A-Za-z0-9_-]+):\s*$/) ?? line.match(/^([A-Za-z0-9_-]+):\s*$/);
  return match ? `${match[1]}:` : previous;
}

function isAllowed(file: string, section: string, line: string): boolean {
  void file;
  void section;
  void line;
  return false;
}

function scanFile(file: string): Finding[] {
  const fullPath = path.join(workspaceRoot, file);
  if (!existsSync(fullPath)) {
    return [];
  }

  const findings: Finding[] = [];
  const lines = readFileSync(fullPath, 'utf-8').split('\n');
  let section = '';

  lines.forEach((line, index) => {
    section = currentSection(line, section);

    for (const needle of forbiddenNeedles) {
      if (!line.includes(needle)) {
        continue;
      }

      if (isAllowed(file, section, line)) {
        continue;
      }

      findings.push({
        file: `${file}:${index + 1}`,
        needle,
        context: line.trim(),
      });
    }
  });

  return findings;
}

function main(): number {
  const mode = parseMode(process.argv.slice(2));
  const findings = backendEntryFiles.flatMap(scanFile);

  console.log('=== Backend Core Audit ===');
  console.log(`mode: ${mode}`);
  console.log('scope: root backend-core contract');
  console.log('simulated removal: root just/moon/scripts must stay free of apps/** and packages/ui/** references');

  if (findings.length === 0) {
    console.log('\nPASS: root backend-core contract is free of apps/** and packages/ui/** references.');
    console.log('Next proof command: just verify');
    return 0;
  }

  console.log('\nFindings:');
  for (const finding of findings) {
    console.log(`  - ${finding.file}: ${finding.needle} :: ${finding.context}`);
  }

  if (mode === 'strict') {
    return 1;
  }

  console.log('\nDry-run reported findings only. Use strict mode to fail on findings.');
  return 0;
}

process.exit(main());
