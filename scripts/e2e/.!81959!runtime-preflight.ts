import net from 'node:net';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { existsSync } from 'node:fs';

const API_HOST = '127.0.0.1';
const API_PORT = 3001;
const WEB_PORT = 5173;
const READYZ_URL = `http://${API_HOST}:${API_PORT}/readyz`;
const READYZ_TIMEOUT_MS = 20_000;
const READYZ_POLL_INTERVAL_MS = 500;

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..', '..');
const svelteTypesDir = path.join(workspaceRoot, 'apps', 'client', 'web', 'app', '.svelte-kit', 'types');

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function fail(message: string, suggestions: string[] = []): never {
