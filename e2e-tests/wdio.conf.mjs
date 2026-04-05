import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');
const tauriDir = path.join(workspaceRoot, 'apps', 'client', 'native', 'src-tauri');

const appBinaryName = process.platform === 'win32' ? 'native-tauri.exe' : 'native-tauri';
const appBinaryPath = path.join(workspaceRoot, 'target', 'debug', appBinaryName);
const appWebOrigin = 'http://tauri.localhost';

let tauriDriver;
let isShuttingDown = false;

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

function registerShutdown() {
  const cleanup = () => closeTauriDriver();
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
  onPrepare: () => {
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
};
