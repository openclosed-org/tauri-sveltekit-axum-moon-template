import os from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { spawn, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');
const tauriDir = path.join(workspaceRoot, 'apps', 'client', 'native', 'src-tauri');

const appBinaryName = process.platform === 'win32' ? 'native-tauri.exe' : 'native-tauri';
const appBinaryPath = path.join(workspaceRoot, 'target', 'debug', appBinaryName);
const appWebOrigin = 'http://tauri.localhost';
const apiBaseUrl = process.env.TAURI_E2E_API_BASE_URL || 'http://127.0.0.1:3001';
const apiPort = (() => {
  const parsed = new URL(apiBaseUrl);
  if (parsed.port) {
    return Number(parsed.port);
  }
  return parsed.protocol === 'https:' ? 443 : 80;
})();
const API_START_TIMEOUT_MS = 180_000;

let tauriDriver;
let apiServer;
let isShuttingDown = false;
let ownsApiServer = false;

function resolveBinaryFromPath(binaryName) {
  const lookupCommand = process.platform === 'win32' ? 'where' : 'which';
  const result = spawnSync(lookupCommand, [binaryName], {
    encoding: 'utf8',
    shell: true,
  });

  if (result.status !== 0 || !result.stdout) {
    return null;
  }

  return result.stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean) ?? null;
}

function resolveTauriDriverBinary() {
  const binaryName = process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver';
  const cargoBinCandidate = path.resolve(os.homedir(), '.cargo', 'bin', binaryName);
  if (existsSync(cargoBinCandidate)) {
    return cargoBinCandidate;
  }

  return resolveBinaryFromPath('tauri-driver');
}

function resolveNativeDriverBinary() {
  if (process.platform !== 'win32') {
    return null;
  }

  const configuredPath = process.env.TAURI_NATIVE_DRIVER;
  if (configuredPath) {
    const resolvedPath = path.resolve(configuredPath);
    if (existsSync(resolvedPath)) {
      return resolvedPath;
    }
  }

  const pathBinary = resolveBinaryFromPath('msedgedriver');
  if (pathBinary) {
    return pathBinary;
  }

  const repoLocalBinary = path.join(workspaceRoot, 'msedgedriver.exe');
  if (existsSync(repoLocalBinary)) {
    return repoLocalBinary;
  }

  return null;
}

function closeTauriDriver() {
  if (tauriDriver) {
    isShuttingDown = true;
    tauriDriver.kill();
    tauriDriver = undefined;
  }
}

function killProcessTree(pid) {
  if (!pid) {
    return;
  }

  if (process.platform === 'win32') {
    spawnSync('taskkill', ['/PID', String(pid), '/F', '/T'], {
      stdio: 'inherit',
    });
  } else {
    process.kill(pid, 'SIGTERM');
  }
}

function closeApiServer() {
  if (apiServer && ownsApiServer) {
    killProcessTree(apiServer.pid);
  }
  apiServer = undefined;
  ownsApiServer = false;
}

function waitForPort(port, timeoutMs) {
  return new Promise((resolve) => {
    const start = Date.now();
    const interval = setInterval(() => {
      if (Date.now() - start >= timeoutMs) {
        clearInterval(interval);
        resolve(false);
        return;
      }

      const socket = net.createConnection({ port, host: '127.0.0.1' }, () => {
        clearInterval(interval);
        socket.end();
        resolve(true);
      });

      socket.on('error', () => {
        socket.destroy();
      });
    }, 500);
  });
}

async function startApiServer() {
  const alreadyRunning = await waitForPort(apiPort, 1000);
  if (alreadyRunning) {
    console.log(`[wdio] Reusing existing API server on ${apiBaseUrl}`);
    return;
  }

  console.log(`[wdio] Starting API server on ${apiBaseUrl}...`);
  apiServer = spawn('cargo', ['run', '-p', 'runtime_server'], {
    cwd: workspaceRoot,
    stdio: ['ignore', 'inherit', 'inherit'],
    shell: true,
  });

  apiServer.on('error', (error) => {
    console.error('[wdio] Failed to start runtime_server:', error);
  });

  ownsApiServer = true;
  const ready = await waitForPort(apiPort, API_START_TIMEOUT_MS);
  if (!ready) {
    closeApiServer();
    throw new Error(
      `runtime_server did not become ready on 127.0.0.1:${apiPort} within ${Math.floor(API_START_TIMEOUT_MS / 1000)}s`
    );
  }

  console.log(`[wdio] API server ready on ${apiBaseUrl}`);
}

function registerShutdown() {
  const cleanup = () => {
    closeTauriDriver();
    closeApiServer();
  };
  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  // Windows-specific signal
  if (process.platform === 'win32') {
    process.on('SIGBREAK', cleanup);
  }
}

registerShutdown();

export const config = {
  host: '127.0.0.1',
  port: 4444,
  outputDir: './test-results/wdio-logs',
  specs: ['./specs/**/*.e2e.mjs'],
  exclude: ['./specs/debug-*.e2e.mjs'],
  maxInstances: 1,
  maxInstancesPerCapability: 1,
  capabilities: [
    {
      maxInstances: 1,
      browserName: 'wry',
      'tauri:options': {
        application: appBinaryPath,
      },
    },
  ],
  logLevel: 'info',
  reporters: [
    'spec',
    ['junit', {
      outputDir: 'test-results/junit',
      outputFileFormat: (opts) => `wdio-results-${opts.cid}.xml`,
      errorOptions: { error: 'stack' },
    }],
  ],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  onPrepare: async () => {
    const tauriDriverPath = resolveTauriDriverBinary();
    if (!tauriDriverPath) {
      throw new Error(
        'tauri-driver not found.\n' +
        `Install it with: cargo install tauri-driver --locked`
      );
    }

    const nativeDriverPath = resolveNativeDriverBinary();
    if (process.platform === 'win32' && !nativeDriverPath) {
      throw new Error(
        'msedgedriver.exe not found.\n' +
        'Install matching Edge Driver and add it to PATH, or set TAURI_NATIVE_DRIVER.'
      );
    }

    console.log(`[wdio] Platform: ${process.platform}`);
    console.log(`[wdio] Tauri driver: ${tauriDriverPath}`);
    console.log(`[wdio] App binary: ${appBinaryPath}`);
    if (nativeDriverPath) {
      console.log(`[wdio] Native driver: ${nativeDriverPath}`);
    }

    const shouldBuild =
      process.env.CI === 'true' ||
      process.env.TAURI_E2E_REBUILD === '1' ||
      !existsSync(appBinaryPath);

    if (shouldBuild) {
      console.log('[wdio] Building app binary...');
      const result = spawnSync(
        'cargo',
        ['tauri', 'build', '--debug', '--no-bundle'],
        {
          cwd: tauriDir,
          stdio: 'inherit',
          shell: true,
        }
      );
      
      if (result.status !== 0) {
        throw new Error('Failed to build tauri app binary');
      }
    } else {
      console.log('[wdio] Using existing binary');
    }

    await startApiServer();
  },
  beforeSession: () => {
    const tauriDriverPath = resolveTauriDriverBinary();
    if (!tauriDriverPath) {
      throw new Error('tauri-driver not found before session start');
    }

    const args = [];

    if (process.platform === 'win32') {
      const nativeDriverPath = resolveNativeDriverBinary();
      if (!nativeDriverPath) {
        throw new Error('msedgedriver.exe not found before session start');
      }
      args.push('--native-driver', nativeDriverPath);
      console.log(`[wdio] Using msedgedriver: ${nativeDriverPath}`);
    }

    tauriDriver = spawn(tauriDriverPath, args, {
      stdio: [null, process.stdout, process.stderr],
    });

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });

    tauriDriver.on('exit', (code) => {
      if (!isShuttingDown && code !== 0) {
        console.error('tauri-driver exited unexpectedly with code:', code);
        process.exit(1);
      }
    });
  },
  before: async () => {
    await browser.url(appWebOrigin);
  },
  afterSession: () => {
    closeTauriDriver();
  },
  onComplete: () => {
    closeApiServer();
  },
};
