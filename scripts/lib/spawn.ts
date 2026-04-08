/**
 * Cross-platform spawn utilities
 * 
 * Shared helpers for all scripts to avoid duplication
 */

import { spawn, spawnSync, type SpawnOptions } from 'node:child_process';
import process from 'node:process';

export interface CommandResult {
  success: boolean;
  output: string;
  error: string;
  exitCode: number;
}

export interface SpawnOptions {
  cwd?: string;
  env?: Record<string, string | undefined>;
  stdio?: 'pipe' | 'inherit' | 'ignore';
}

const isWindows = process.platform === 'win32';

/**
 * Run a command asynchronously with cross-platform support
 */
export function run(
  cmd: string,
  args: string[] = [],
  options: SpawnOptions = {}
): Promise<CommandResult> {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, {
      stdio: options.stdio === 'inherit' ? 'inherit' : ['pipe', 'pipe', 'pipe'],
      shell: isWindows,
      cwd: options.cwd,
      env: options.env,
    });

    let stdout = '';
    let stderr = '';

    if (child.stdout) {
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
    }

    if (child.stderr) {
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
    }

    child.on('close', (code) => {
      resolve({
        success: code === 0,
        output: stdout.trim(),
        error: stderr.trim(),
        exitCode: code ?? 1,
      });
    });

    child.on('error', (err) => {
      resolve({
        success: false,
        output: '',
        error: err.message,
        exitCode: 1,
      });
    });
  });
}

/**
 * Run a command synchronously with cross-platform support
 */
export function runSync(
  cmd: string,
  args: string[] = [],
  options: SpawnOptions = {}
): CommandResult {
  const result = spawnSync(cmd, args, {
    stdio: options.stdio === 'inherit' ? 'inherit' : ['pipe', 'pipe', 'pipe'],
    shell: isWindows,
    cwd: options.cwd,
    env: options.env,
    encoding: 'utf8',
  });

  return {
    success: result.status === 0,
    output: result.stdout?.toString().trim() || '',
    error: result.stderr?.toString().trim() || '',
    exitCode: result.status ?? 1,
  };
}

/**
 * Run a command and print output to console (inherit stdio)
 */
export function runInherit(
  cmd: string,
  args: string[] = [],
  options: SpawnOptions = {}
): Promise<number> {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, {
      stdio: 'inherit',
      shell: isWindows,
      cwd: options.cwd,
      env: options.env,
    });

    child.on('close', (code) => {
      resolve(code ?? 1);
    });

    child.on('error', () => {
      resolve(1);
    });
  });
}

/**
 * Check if a command-line tool is available
 */
export async function hasTool(name: string): Promise<boolean> {
  const checkCmd = isWindows ? 'where' : 'command';
  const checkArgs = isWindows ? [name] : ['-v', name];

  return new Promise((resolve) => {
    const child = spawn(checkCmd, checkArgs, {
      stdio: 'pipe',
      shell: isWindows,
    });

    child.on('close', (code) => {
      resolve(code === 0);
    });

    child.on('error', () => {
      resolve(false);
    });
  });
}

/**
 * Require a tool or exit with helpful message
 */
export async function requireTool(name: string, installHint: string): Promise<void> {
  const available = await hasTool(name);
  if (!available) {
    console.error(`Error: '${name}' not found`);
    console.error(`Install hint: ${installHint}`);
    process.exit(1);
  }
}

/**
 * Kill a process and its children (cross-platform)
 */
export function killProcess(pid: number | undefined): void {
  if (!pid) return;

  if (isWindows) {
    spawnSync('taskkill', ['/PID', String(pid), '/F', '/T'], {
      stdio: 'inherit',
    });
  } else {
    try {
      process.kill(-pid, 'SIGTERM');
    } catch {
      process.kill(pid, 'SIGTERM');
    }
  }
}

/**
 * Wait for a TCP port to become available
 */
export function waitForPort(
  port: number,
  maxSeconds: number,
  host: string = 'localhost'
): Promise<boolean> {
  return new Promise((resolve) => {
    const startTime = Date.now();
    const interval = setInterval(() => {
      const elapsed = (Date.now() - startTime) / 1000;
      if (elapsed >= maxSeconds) {
        clearInterval(interval);
        resolve(false);
        return;
      }

      const net = require('node:net');
      const socket = net.createConnection({ port, host }, () => {
        clearInterval(interval);
        socket.end();
        resolve(true);
      });

      socket.on('error', () => {
        socket.destroy();
      });
    }, 1000);
  });
}

/**
 * Check if a TCP port is occupied
 */
export function isPortOccupied(port: number, host: string = '127.0.0.1'): Promise<boolean> {
  return new Promise((resolve) => {
    const net = require('node:net');
    const socket = net.createConnection({ host, port }, () => {
      socket.end();
      resolve(true);
    });

    socket.on('error', () => {
      socket.destroy();
      resolve(false);
    });
  });
}
