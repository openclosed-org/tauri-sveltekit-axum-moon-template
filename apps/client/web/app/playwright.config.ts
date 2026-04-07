import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests/e2e',
	globalSetup: './tests/fixtures/global-setup.ts',
	outputDir: 'test-results',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: [
		['html', { outputFolder: 'playwright-report' }],
		['junit', { outputFile: 'playwright-results.xml' }],
		['json', { outputFile: 'playwright-results.json' }]
	],
	use: {
		baseURL: 'http://localhost:5173',
		trace: 'on-first-retry',
		screenshot: 'only-on-failure',
		video: 'retain-on-failure',
		actionTimeout: 10000,
		navigationTimeout: 30000
	},
	projects: [
		{
			name: 'desktop-chrome',
			use: { ...devices['Desktop Chrome'] }
		},
		{
			name: 'desktop-firefox',
			use: { ...devices['Desktop Firefox'] }
		},
		{
			name: 'desktop-safari',
			use: { ...devices['Desktop Safari'] }
		},
		{
			name: 'mobile-chrome',
			use: { ...devices['Pixel 5'] }
		},
		{
			name: 'mobile-safari',
			use: { ...devices['iPhone 12'] }
		}
	],
	webServer: {
		command: 'bun run dev',
		url: 'http://localhost:5173',
		reuseExistingServer: !process.env.CI,
		timeout: 120000
	},
	timeout: 60000,
	expect: {
		timeout: 5000
	}
});
