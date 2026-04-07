import { test as base, expect, type Page } from '@playwright/test';

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

/** Verify user is logged in by URL guard behavior. */
export async function verifyLoggedIn(page: Page) {
	await expect(page).not.toHaveURL(/\/login/, { timeout: 5000 });
}

/** Verify user is logged out (on login page). */
export async function verifyLoggedOut(page: Page) {
	await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
}

/** Extended test fixture with mockLogin helper. */
export const test = base.extend<{ mockLogin: void }>({
	mockLogin: [
		async ({ page }, use) => {
			await page.goto('/login');
			await triggerMockOAuth(page);
			await page.waitForTimeout(1000);
			await use();
		},
		{ auto: false }
	]
});

export { expect };
