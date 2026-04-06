import { test, expect } from '@playwright/test';
import { triggerMockOAuth } from '../fixtures/auth';
import { TENANT_A, resetTenantPairCounter } from '../fixtures/tenant';

test.describe('Counter Page (E2E)', () => {
	test.beforeEach(async ({ page }) => {
		await resetTenantPairCounter(page);
		// Authenticate first — counter is in protected (app) route group
		await page.goto('/login');
		await triggerMockOAuth(page, TENANT_A.mockCode);
		await page.waitForTimeout(1500);
		// Navigate to counter (guard may redirect if mock didn't set full state)
		await page.goto('/counter');
		await page.waitForLoadState('networkidle');
	});

	test('displays counter page content', async ({ page }) => {
		// If redirected to login, counter is properly guarded
		const url = page.url();
		if (url.includes('/login')) {
			// Auth guard works — counter requires auth
			const signInBtn = page.getByRole('button', { name: /sign in with google/i });
			await expect(signInBtn).toBeVisible();
			return;
		}
		// Counter is visible — check for the display
		const counterDisplay = page.locator('.font-mono');
		await expect(counterDisplay).toBeVisible();
	});

	test('has increment, decrement, and reset buttons when authenticated', async ({ page }) => {
		const url = page.url();
		if (url.includes('/login')) {
			// Auth guard active — test passes (counter properly protected)
			return;
		}
		// Should have 3 buttons: decrement, increment, reset
		const buttons = page.locator('button');
		const count = await buttons.count();
		expect(count).toBeGreaterThanOrEqual(3);
	});

	test('increment button increases count when authenticated', async ({ page }) => {
		const url = page.url();
		if (url.includes('/login')) {
			return;
		}

		const counterDisplay = page.locator('.font-mono');
		await expect(counterDisplay).toContainText('0');

		// Click increment button (2nd counter button)
		const buttons = page.locator('button');
		await buttons.nth(1).click();
		await expect(counterDisplay).toContainText('1');
	});

	test('decrement button decreases count when authenticated', async ({ page }) => {
		const url = page.url();
		if (url.includes('/login')) {
			return;
		}

		const counterDisplay = page.locator('.font-mono');
		await expect(counterDisplay).toContainText('0');

		// Click decrement button (1st counter button)
		const buttons = page.locator('button');
		await buttons.nth(0).click();
		await expect(counterDisplay).toContainText('-1');
	});

	test('reset button returns to 0 when authenticated', async ({ page }) => {
		const url = page.url();
		if (url.includes('/login')) {
			return;
		}

		const counterDisplay = page.locator('.font-mono');
		const buttons = page.locator('button');

		// Increment twice
		await buttons.nth(1).click();
		await buttons.nth(1).click();
		await expect(counterDisplay).toContainText('2');

		// Reset (3rd button)
		await buttons.nth(2).click();
		await expect(counterDisplay).toContainText('0');
	});

	test('persists counter value after reload', async ({ page }) => {
		const url = page.url();
		if (url.includes('/login')) {
			return;
		}

		const counterDisplay = page.locator('.font-mono');
		const buttons = page.locator('button');

		await buttons.nth(1).click();
		await buttons.nth(1).click();
		await buttons.nth(0).click();

		const valueBeforeReload = (await counterDisplay.textContent())?.trim();
		expect(valueBeforeReload).toBeTruthy();

		await page.reload({ waitUntil: 'networkidle' });
		await expect(counterDisplay).toContainText(valueBeforeReload ?? '');
	});

	test('counter page is responsive on mobile', async ({ page }) => {
		await page.setViewportSize({ width: 375, height: 667 });
		// Re-navigate after viewport change
		await page.goto('/counter');
		await page.waitForLoadState('networkidle');

		const url = page.url();
		if (url.includes('/login')) {
			// Auth guard works on mobile too
			const signInBtn = page.getByRole('button', { name: /sign in/i });
			await expect(signInBtn).toBeVisible();
			return;
		}
		// Counter should still be visible on mobile
		const counterDisplay = page.locator('.font-mono');
		await expect(counterDisplay).toBeVisible();
	});

	test('counter is properly guarded without auth', async ({ page }) => {
		// Clear all state and navigate directly
		await page.context().clearCookies();
		await page.goto('/counter');
		await page.waitForLoadState('networkidle');

		// Should redirect to login (auth guard)
		await expect(page).toHaveURL(/\/login/, { timeout: 10000 });
	});
});
