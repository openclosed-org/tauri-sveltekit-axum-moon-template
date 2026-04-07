import { createTauriTest } from '@srsholmes/tauri-playwright';

export const { test, expect } = createTauriTest({
  devUrl: 'http://localhost:5173',
  tauriCommand: 'cargo tauri dev',
  tauriCwd: '../apps/client/native/src-tauri',
  tauriFeatures: ['e2e-testing'],
  mcpSocket: './test-results/tauri-playwright.sock',
  startTimeout: 180,
  ipcMocks: {
    start_oauth: () => ({ ok: true })
  }
});
