import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const e2eRoot = path.resolve(__dirname, '..');
const diagnosticsDir = path.join(e2eRoot, 'test-results', 'diagnostics');

const TENANT_MAPPING_EVIDENCE = {
  source: 'e2e-tests/helpers/tenant.mjs',
  generated_by: 'e2e-tests/scripts/run-desktop-e2e.mjs',
  generated_at: new Date().toISOString(),
  tenants: [
    {
      label: 'tenant-1',
      userSub: 'tenant_a_user',
      userName: 'Tenant A User',
    },
    {
      label: 'tenant-2',
      userSub: 'tenant_b_user',
      userName: 'Tenant B User',
    },
  ],
};

const force = process.argv.includes('--force');
const ciMode = process.argv.includes('--ci');
const supported = process.platform === 'linux' || process.platform === 'win32';

// macOS is unsupported for WebDriver testing
if (process.platform === 'darwin') {
  console.log('[desktop-e2e] Skip: macOS does not support WebDriver testing.');
  console.log('[desktop-e2e] This is a known limitation (no native WKWebView driver on macOS).');
  console.log('[desktop-e2e] Use --force to attempt local execution anyway.');
  if (!force) {
    process.exit(0);
  }
}

if (ciMode) {
  console.log('[desktop-e2e] CI mode enabled');
  console.log('[desktop-e2e] Platform:', process.platform);

  if (!supported) {
    console.log('[desktop-e2e] Error: Running on unsupported platform in CI mode');
    console.log('[desktop-e2e] Expected: linux or win32');
    process.exit(1);
  }
} else if (!supported && !force) {
  console.log('[desktop-e2e] Skip: tauri-driver currently supports desktop WebDriver on Linux/Windows only.');
  console.log('[desktop-e2e] Current platform:', process.platform);
  console.log('[desktop-e2e] Use `bun run test:force` to attempt local execution anyway.');
  process.exit(0);
}

console.log('[desktop-e2e] Starting WDIO test run...');
console.log('[desktop-e2e] Platform:', process.platform);

mkdirSync(diagnosticsDir, { recursive: true });
writeFileSync(
  path.join(diagnosticsDir, 'tenant-mapping.json'),
  `${JSON.stringify(TENANT_MAPPING_EVIDENCE, null, 2)}\n`,
  'utf8',
);
writeFileSync(
  path.join(diagnosticsDir, 'run-context.json'),
  `${JSON.stringify(
    {
      platform: process.platform,
      ciMode,
      force,
      cwd: e2eRoot,
      generatedAt: new Date().toISOString(),
    },
    null,
    2,
  )}\n`,
  'utf8',
);

const result = spawnSync('wdio', ['run', 'wdio.conf.mjs'], {
  cwd: e2eRoot,
  env: {
    ...process.env,
    CI: ciMode ? 'true' : process.env.CI,
  },
  stdio: 'inherit',
  shell: true,
});

if (typeof result.status === 'number') {
  if (result.status === 0) {
    console.log('[desktop-e2e] ✅ All tests passed successfully');
  } else {
    console.error(`[desktop-e2e] ❌ Tests failed with exit code: ${result.status}`);
  }
  process.exit(result.status);
}

console.error('[desktop-e2e] Unknown error occurred');
process.exit(1);
