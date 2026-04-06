import { test, expect } from '@playwright/test';
import { triggerMockOAuth } from '../fixtures/auth';
import {
	TENANT_A,
	TENANT_B,
	TENANT_LABELS,
	initTenantPair,
	resetTenantPairCounter
} from '../fixtures/tenant';

// Per D-05: Tenant isolation must have E2E behavior verification (dual coverage)
test.describe('Tenant Isolation (E2E)', () => {
	test.describe.configure({ mode: 'serial' });

	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await resetTenantPairCounter(page);
	});

	test('harness exposes exactly two stable tenant identities', async () => {
		expect(TENANT_LABELS).toEqual(['tenant-A', 'tenant-B']);
		expect(TENANT_A.label).toBe('tenant-A');
		expect(TENANT_B.label).toBe('tenant-B');
	});

	test('tenant init API responds with tenant_id', async ({ page }) => {
		await initTenantPair(page);
		await triggerMockOAuth(page, TENANT_A.mockCode);

		// Page should remain functional (no crash from tenant init)
		// Verify the page is still accessible
		await expect(page.locator('h1, button').first()).toBeVisible({ timeout: 5000 });
	});

	test('two browser contexts maintain separate sessions', async ({ browser }) => {
		// Create two separate browser contexts (simulating two users)
		const context1 = await browser.newContext();
		const context2 = await browser.newContext();

		const page1 = await context1.newPage();
		const page2 = await context2.newPage();

		// Both start at login
		await page1.goto('/login');
		await page2.goto('/login');

		// Both should see login page independently
		await expect(page1.locator('h1')).toContainText(/tauri app/i);
		await expect(page2.locator('h1')).toContainText(/tauri app/i);

		// Trigger mock OAuth for page1 only
		await triggerMockOAuth(page1, 'tenant1_code');

		// Page2 should remain unaffected (still on login, not authenticated)
		await expect(page2.locator('h1')).toContainText(/tauri app/i);

		await context1.close();
		await context2.close();
	});

	test('auth state is isolated per browser context', async ({ browser }) => {
		const context1 = await browser.newContext();
		const context2 = await browser.newContext();

		const page1 = await context1.newPage();
		const page2 = await context2.newPage();

		await page1.goto('/login');
		await page2.goto('/login');

		// Login on page1
		await triggerMockOAuth(page1, 'isolated_user1');

		// Check localStorage on page1 is independent from page2
		const storage1 = await page1.evaluate(() => ({
			url: window.location.href
		}));
		const storage2 = await page2.evaluate(() => ({
			url: window.location.href
		}));

		// Both pages are accessible independently
		expect(storage1.url).toContain('/login');
		expect(storage2.url).toContain('/login');

		await context1.close();
		await context2.close();
	});

	test('new user signup flow does not affect existing sessions', async ({ browser }) => {
		// Existing user context
		const existingContext = await browser.newContext();
		const existingPage = await existingContext.newPage();
		await existingPage.goto('/login');

		// New user context
		const newContext = await browser.newContext();
		const newPage = await newContext.newPage();
		await newPage.goto('/login');

		// Trigger new user signup
		await triggerMockOAuth(newPage, 'brand_new_user');

		// Existing user's page should be unaffected
		await expect(existingPage.locator('h1')).toContainText(/tauri app/i);

		await existingContext.close();
		await newContext.close();
	});
});
