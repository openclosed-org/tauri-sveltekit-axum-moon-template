import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..');
const nativeProjectRoot = path.join(workspaceRoot, 'apps', 'client', 'native');

const appBinaryName = process.platform === 'win32' ? 'native-tauri.exe' : 'native-tauri';
const appBinaryPath = path.join(workspaceRoot, 'target', 'debug', appBinaryName);

let tauriDriver;
let isShuttingDown = false;

function resolveTauriDriverBinary() {
  const binaryName = process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver';
  return path.resolve(os.homedir(), '.cargo', 'bin', binaryName);
}

function resolveCargoTauriCommand() {
  const cargoBin = path.resolve(os.homedir(), '.cargo', 'bin', 'cargo-tauri');
  if (process.platform === 'win32') {
    return `"${cargoBin}.exe"`;
  }
  return `"${cargoBin}"`;
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
}

registerShutdown();

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./specs/**/*.e2e.mjs'],
  maxInstances: 1,
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
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  onPrepare: () => {
    const buildResult = spawnSync(
      `${resolveCargoTauriCommand()} build --debug --no-bundle --manifest-path src-tauri/Cargo.toml`,
      [],
      {
        cwd: nativeProjectRoot,
        stdio: 'inherit',
        shell: true,
      },
    );

    if (buildResult.status !== 0) {
      throw new Error(`Tauri debug build failed with exit code ${buildResult.status ?? 'unknown'}`);
    }
  },
  beforeSession: () => {
    tauriDriver = spawn(resolveTauriDriverBinary(), [], {
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
  afterSession: () => {
    closeTauriDriver();
  },
};
