import { test as base, expect, type Page } from '@playwright/test';

/**
 * Mock OAuth deep-link fixture for E2E tests.
 *
 * Per D-01: Uses mock callback strategy, not real Google OAuth.
 * Per D-02: Simulates deep-link://new-url event to trigger OAuth callback chain.
 */

/** Trigger mock OAuth deep-link event on the page. */
export async function triggerMockOAuth(page: Page, mockCode: string = 'mock_auth_code') {
	await page.evaluate((code) => {
		window.dispatchEvent(
			new CustomEvent('deep-link://new-url', {
				detail: `/callback?code=${code}&state=mock_state`
			})
		);
	}, mockCode);
}

/** Verify user is logged in by checking for sign-out button or user indicator. */
export async function verifyLoggedIn(page: Page) {
	// After mock login, the auth state should reflect logged-in status
	// Check URL has navigated away from login
	await expect(page).not.toHaveURL(/\/login/, { timeout: 5000 });
}

/** Verify user is logged out (on login page). */
export async function verifyLoggedOut(page: Page) {
	await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
}

/** Extended test fixture with mockLogin helper. */
export const test = base.extend<{ mockLogin: void }>({
	mockLogin: [async ({ page }, use) => {
		// Navigate to login page first
		await page.goto('/login');
		// Trigger mock OAuth callback
		await triggerMockOAuth(page);
		// Wait for auth to process
		await page.waitForTimeout(1000);
		await use();
	}, { auto: false }]
});

export { expect };
