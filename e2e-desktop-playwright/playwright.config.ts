import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/specs',
  outputDir: 'test-results',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['junit', { outputFile: 'playwright-results.xml' }],
    ['json', { outputFile: 'playwright-results.json' }]
  ],
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 10_000,
    navigationTimeout: 30_000
  },
  projects: [
    {
      name: 'tauri',
      use: {
        mode: 'cdp'
      }
    }
  ],
  webServer: {
    command:
      'cmd /C "set WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 && cargo tauri dev --features e2e-testing --config tauri.e2e.conf.json"',
    url: 'http://127.0.0.1:9222/json/version',
    cwd: '../apps/client/native/src-tauri',
    reuseExistingServer: !process.env.CI,
    timeout: 180_000
  },
  timeout: 90_000,
  expect: {
    timeout: 10_000
  }
});
