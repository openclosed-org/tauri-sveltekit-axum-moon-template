import { test as base, expect, type Page } from '@playwright/test';

const APP_BASE_URL = 'http://localhost:5173';

function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Trigger mock OAuth deep-link event on the page. */
export async function triggerMockOAuth(page: Page, mockCode: string = 'mock_auth_code') {
	const callbackUrl = `http://localhost:5173/oauth/callback?code=${mockCode}&state=mock_state`;
	await (page as any).evaluate(
		`(async () => {
			const callbackUrl = ${JSON.stringify(callbackUrl)};
			window.dispatchEvent(new CustomEvent('deep-link://new-url', { detail: '/callback?code=' + ${JSON.stringify(mockCode)} + '&state=mock_state' }));
			const tauri = window.__TAURI__;
			if (tauri?.event?.emit) {
				await tauri.event.emit('oauth-callback', callbackUrl);
			}
		})();`
	);
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
			await (page as any).goto(`${APP_BASE_URL}/login`);
			await triggerMockOAuth(page);
			await sleep(1000);
			await use();
		},
		{ auto: false }
	]
});

export { expect };
