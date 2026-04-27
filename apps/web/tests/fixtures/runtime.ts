import { type ChildProcess, spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';

const API_HOST = '127.0.0.1';
const API_PORT = 3010;
const API_READY_URL = `http://${API_HOST}:${API_PORT}/readyz`;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS = 120_000;
const WEB_TYPES_DIR = path.join('apps', 'client', 'web', 'app', '.svelte-kit', 'types');

const workspaceRoot = path.resolve(process.cwd(), '..', '..', '..', '..');

let ownedApiProcess: ChildProcess | null = null;
let cleanupRegistered = false;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function webBffBinaryPath(): string {
  const binary = process.platform === 'win32' ? 'web-bff.exe' : 'web-bff';
  return path.join(workspaceRoot, 'target', 'debug', binary);
}

async function waitForApiReady(timeoutMs: number): Promise<boolean> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(API_READY_URL);
      if (response.ok) {
        return true;
      }
    } catch {
      // keep polling until timeout
    }

    await sleep(500);
  }

  return false;
}

function registerCleanupOnce(): void {
  if (cleanupRegistered) {
    return;
  }

  cleanupRegistered = true;
  process.on('exit', () => {
    stopOwnedApiProcess();
  });
}

function startOwnedApiProcess(): void {
  if (ownedApiProcess) {
    return;
  }

  const webBffBinary = webBffBinaryPath();
  if (existsSync(webBffBinary)) {
    ownedApiProcess = spawn(webBffBinary, [], {
      cwd: workspaceRoot,
      stdio: 'ignore',
      shell: false,
    });
    return;
  }

  ownedApiProcess = spawn('cargo', ['run', '-p', 'web-bff'], {
    cwd: workspaceRoot,
    stdio: 'ignore',
    shell: process.platform === 'win32',
  });
}

export async function ensureApiReady(timeoutMs = DEFAULT_BOOTSTRAP_TIMEOUT_MS): Promise<void> {
  if (await waitForApiReady(1_000)) {
    return;
  }

  registerCleanupOnce();
  startOwnedApiProcess();

  const ready = await waitForApiReady(timeoutMs);
  if (!ready) {
    const pid = ownedApiProcess?.pid ?? 'none';
    stopOwnedApiProcess();
    throw new Error(
      `[runtime] API not ready at ${API_READY_URL} within ${timeoutMs}ms (owned_pid=${pid}, port=${API_PORT})`,
    );
  }
}

export async function ensureWebE2EPreflight(
  timeoutMs = DEFAULT_BOOTSTRAP_TIMEOUT_MS,
): Promise<void> {
  await ensureApiReady(timeoutMs);

  const typesDir = path.join(workspaceRoot, WEB_TYPES_DIR);
  if (!existsSync(typesDir)) {
    throw new Error(
      `[runtime] missing SvelteKit type artifacts at ${typesDir}; run: rtk bun run --cwd apps/web check`,
    );
  }
}

export function stopOwnedApiProcess(): void {
  if (!ownedApiProcess) {
    return;
  }

  if (!ownedApiProcess.killed) {
    if (process.platform === 'win32') {
      spawn('taskkill', ['/PID', String(ownedApiProcess.pid), '/F', '/T'], {
        stdio: 'ignore',
        shell: false,
      });
    } else {
      ownedApiProcess.kill('SIGTERM');
    }
  }

  ownedApiProcess = null;
}
