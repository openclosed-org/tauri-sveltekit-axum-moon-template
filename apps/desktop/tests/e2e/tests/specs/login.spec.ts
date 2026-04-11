import { test, expect } from '../fixtures/tauri';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Login', () => {
	test('shows login page with Google sign-in button', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/login`);

		await expect(tauriPage.getByRole('heading', { name: 'Tauri App' })).toBeVisible();
		await expect(tauriPage.getByText('Welcome back')).toBeVisible();
		await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
	});

	test('shows disabled email input', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/login`);

		const emailInput = tauriPage.locator('input[type="email"]');
		await expect(emailInput).toBeVisible();
		await expect(emailInput).toBeDisabled();
	});
});
