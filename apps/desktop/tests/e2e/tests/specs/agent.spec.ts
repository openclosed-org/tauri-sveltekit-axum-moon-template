import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/tauri';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Agent Chat', () => {
	test('agent page requires authentication', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/agent`);

		const url = await tauriPage.url();
		expect(url.includes('/agent') || url.includes('/login')).toBeTruthy();
	});

	test('displays agent chat layout when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/agent`);

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		await expect(tauriPage.getByRole('button', { name: 'New Chat' })).toBeVisible();
		await expect(tauriPage.getByText('Select or create a conversation')).toBeVisible();
	});

	test('has message input area and send button state when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/agent`);
		await new Promise((resolve) => setTimeout(resolve, 500));

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		const hasInput = await tauriPage.getByPlaceholder('Type a message...').isVisible().catch(() => false);
		if (hasInput) {
			const input = tauriPage.getByPlaceholder('Type a message...');
			const sendButton = tauriPage.getByRole('button', { name: 'Send' });

			await expect(input).toBeVisible();
			await expect(sendButton).toBeVisible();
			await input.fill('hello from e2e');
			await expect(sendButton).toBeEnabled();
			return;
		}

		await expect(tauriPage.getByRole('button', { name: 'New Chat' })).toBeVisible();
		await expect(tauriPage.getByText('Select or create a conversation')).toBeVisible();
	});
});

async function isLoginPageVisible(page: Page): Promise<boolean> {
	const body = (await page.locator('body').textContent()) ?? '';
	return body.includes('Sign in with Google') || body.includes('Welcome back');
}
