/**
 * Doctor - Toolchain and Config Health Check
 * 
 * Verifies all required tools and config files are present
 * Stage: Setup/bootstrap
 */

import { hasTool } from '../lib/spawn.js';
import { existsSync } from 'node:fs';
import process from 'node:process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

interface ToolCheck {
  name: string;
  cmd: string;
}

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');

async function checkTool({ name, cmd }: ToolCheck): Promise<boolean> {
  const available = await hasTool(cmd);
  if (available) {
    const result = await import('../lib/spawn.js').then(({ run }) => run(cmd, ['--version']));
    console.log(`✅ ${name}: ${result.output}`);
  } else {
    console.log(`❌ MISSING: ${name} — run: just setup`);
  }
  return available;
}

function checkFileExists(filePath: string, label: string): boolean {
  if (existsSync(filePath)) {
    console.log(`✅ ${label}: exists`);
    return true;
  }
  console.log(`❌ MISSING: ${label}`);
  return false;
}

async function main(): Promise<number> {
  console.log('=== Toolchain Check ===\n');

  const tools: ToolCheck[] = [
    { name: 'bun', cmd: 'bun' },
    { name: 'node', cmd: 'node' },
    { name: 'cargo', cmd: 'cargo' },
    { name: 'rustc', cmd: 'rustc' },
    { name: 'moon', cmd: 'moon' },
  ];

  const results = await Promise.all(tools.map(checkTool));

  console.log('\n=== Config Files Check ===\n');

  const configFiles = [
    { path: path.join(workspaceRoot, '.env'), label: '.env' },
    { path: path.join(workspaceRoot, '.env.example'), label: '.env.example' },
    { path: path.join(workspaceRoot, '.tool-versions'), label: '.tool-versions' },
    { path: path.join(workspaceRoot, 'rust-toolchain.toml'), label: 'rust-toolchain.toml' },
  ];

  const fileResults = configFiles.map(({ path: p, label }) => checkFileExists(p, label));

  console.log('\n=== Summary ===\n');

  const toolPass = results.filter(Boolean).length;
  const filePass = fileResults.filter(Boolean).length;

  console.log(`Tools: ${toolPass}/${tools.length} installed`);
  console.log(`Configs: ${filePass}/${configFiles.length} present`);

  const allPassed = results.every(Boolean) && fileResults.every(Boolean);

  if (allPassed) {
    console.log('\n✅ All checks passed');
    return 0;
  }

  console.log('\n⚠️  Some checks failed — run: just setup');
  return 1;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
