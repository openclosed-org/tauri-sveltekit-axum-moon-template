/**
 * Typegen - Contract Binding Generation
 * 
 * Generates TypeScript type bindings from Rust contracts and syncs to frontend
 * Stage: Code generation
 */

import { run } from '../lib/spawn.js';
import process from 'node:process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { existsSync, mkdirSync, cpSync, rmSync, readdirSync } from 'node:fs';

interface BindingPath {
  src: string;
  dest: string;
}

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');

const contractDirs: BindingPath[] = [
  { src: 'packages/contracts/api/bindings/api', dest: 'packages/contracts/generated/api' },
  { src: 'packages/contracts/auth/bindings/auth', dest: 'packages/contracts/generated/auth' },
  { src: 'packages/contracts/events/bindings/events', dest: 'packages/contracts/generated/events' },
];

const frontendDest = 'apps/client/web/app/src/lib/generated';

function safeCopy(src: string, dest: string): void {
  if (!existsSync(src)) {
    console.warn(`  ⚠️  Source directory does not exist: ${src}`);
    return;
  }

  mkdirSync(dest, { recursive: true });

  try {
    cpSync(src, dest, { recursive: true });
    console.log(`  ✓ Copied ${path.basename(src)} → ${path.basename(dest)}`);
  } catch (err) {
    console.warn(`  ⚠️  Could not copy ${src} to ${dest}: ${err}`);
  }
}

function listDirectory(dir: string): void {
  if (!existsSync(dir)) {
    console.log(`  (directory does not exist: ${dir})`);
    return;
  }

  const files = readdirSync(dir, { withFileTypes: true });
  for (const file of files) {
    console.log(`  ${file.isDirectory() ? '📁' : '📄'} ${file.name}`);
  }
}

async function main(): Promise<number> {
  console.log('=== Running typegen ===\n');

  // Step 1: Generate contract bindings
  console.log('[1/4] Generating contract bindings...');
  const testResult = await run('cargo', [
    'test',
    '-p', 'contracts_api',
    '-p', 'contracts_auth',
    '-p', 'contracts_events',
  ]);

  if (!testResult.success) {
    console.error('❌ Contract generation failed:');
    console.error(testResult.error);
    return 1;
  }
  console.log('  ✓ Contract bindings generated\n');

  // Step 2: Clean old generated files
  console.log('[2/4] Cleaning old generated files...');
  for (const { dest } of contractDirs) {
    const fullPath = path.join(workspaceRoot, dest);
    if (existsSync(fullPath)) {
      rmSync(fullPath, { recursive: true, force: true });
    }
  }
  console.log('  ✓ Old files cleaned\n');

  // Step 3: Copy generated types
  console.log('[3/4] Copying generated types...');
  for (const { src, dest } of contractDirs) {
    const fullSrc = path.join(workspaceRoot, src);
    const fullDest = path.join(workspaceRoot, dest);
    safeCopy(fullSrc, fullDest);
  }
  console.log('');

  // Step 4: Sync to frontend
  console.log('[4/4] Syncing to frontend...');
  const fullFrontendDest = path.join(workspaceRoot, frontendDest);
  mkdirSync(fullFrontendDest, { recursive: true });

  for (const { dest } of contractDirs) {
    const fullSrc = path.join(workspaceRoot, dest);
    safeCopy(fullSrc, fullFrontendDest);
  }

  console.log('\n=== Typegen complete ===\n');
  console.log('Frontend generated types:');
  listDirectory(fullFrontendDest);

  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
