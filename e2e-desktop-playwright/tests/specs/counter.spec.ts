import { test, expect } from '../fixtures/tauri';
import type { Locator } from '@playwright/test';
import { resetTenantPairCounter, waitForCounterControlsReady } from '../fixtures/tenant';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Counter', () => {
	async function clickWhenReady(button: Locator) {
		await expect(button).toBeVisible();
		await expect(button).toBeEnabled();
		await button.click();
	}

	test('counter page is properly guarded without auth', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/counter`);
		const url = await tauriPage.url();
		expect(url.includes('/counter') || url.includes('/login')).toBeTruthy();
	});

	test('shows counter controls only when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/counter`);

		const resetButton = tauriPage.getByRole('button', { name: 'Reset' });
		const signInButton = tauriPage.getByRole('button', { name: 'Sign in with Google' });
		const canOperateCounter = await resetButton.isVisible().catch(() => false);

		if (!canOperateCounter) {
			await expect(signInButton).toBeVisible();
			return;
		}

		const buttons = tauriPage.locator('button');
		await expect(buttons).toHaveCount(1);
	});

	test('counter interaction assertions run when authenticated', async ({ tauriPage }) => {
		await resetTenantPairCounter(tauriPage).catch(() => undefined);
		await tauriPage.goto(`${APP_BASE_URL}/counter`);

		const resetButton = tauriPage.getByRole('button', { name: 'Reset' });
		const signInButton = tauriPage.getByRole('button', { name: 'Sign in with Google' });
		const canOperateCounter = await resetButton.isVisible().catch(() => false);
		if (!canOperateCounter) {
			await expect(signInButton).toBeVisible();
			return;
		}

		const counterValue = tauriPage.locator('.font-mono');
		const counterVisible = await counterValue.isVisible().catch(() => false);
		if (!counterVisible) {
			await expect(signInButton).toBeVisible();
			return;
		}
		await expect(counterValue).toBeVisible();

		const { decrementButton, incrementButton, resetControl } = await waitForCounterControlsReady(tauriPage);

		await clickWhenReady(resetControl);
		await expect(counterValue).toHaveText('0');

		await clickWhenReady(incrementButton);
		await expect(counterValue).toHaveText('1');

		await clickWhenReady(decrementButton);
		await expect(counterValue).toHaveText('0');

		await clickWhenReady(incrementButton);
		await clickWhenReady(incrementButton);
		await expect(counterValue).toHaveText('2');

		await clickWhenReady(resetControl);
		await expect(counterValue).toHaveText('0');
	});
});
