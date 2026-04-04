import { spawnSync } from 'node:child_process';
import process from 'node:process';

const force = process.argv.includes('--force');
const supported = process.platform === 'linux' || process.platform === 'win32';

if (!supported && !force) {
  console.log('[desktop-e2e] Skip: tauri-driver currently supports desktop WebDriver on Linux/Windows only.');
  console.log('[desktop-e2e] Current platform:', process.platform);
  console.log('[desktop-e2e] Use `bun run test:force` to attempt local execution anyway.');
  process.exit(0);
}

const result = spawnSync('wdio', ['run', 'wdio.conf.mjs'], {
  stdio: 'inherit',
  shell: true,
});

if (typeof result.status === 'number') {
  process.exit(result.status);
}

process.exit(1);
