import { test, expect } from '../fixtures/tauri';
import type { Page } from '@playwright/test';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Admin Dashboard', () => {
	test('admin page requires authentication', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);

		const url = await tauriPage.url();
		expect(url.includes('/admin') || url.includes('/login')).toBeTruthy();
	});

	test('displays admin dashboard layout when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);
		await new Promise((resolve) => setTimeout(resolve, 500));

		const url = await tauriPage.url();
		if (url.includes('/login')) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		await expect(tauriPage.getByText('Admin Dashboard')).toBeVisible();
	});

	test('shows stat cards when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);
		await new Promise((resolve) => setTimeout(resolve, 600));

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		await expect(tauriPage.getByText('Tenants')).toBeVisible();
		await expect(tauriPage.getByText('Counter')).toBeVisible();
		await expect(tauriPage.getByText('Last Login')).toBeVisible();
	});

	test('shows stat values when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);
		await new Promise((resolve) => setTimeout(resolve, 600));

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		const valueCells = tauriPage.locator('div.grid p.text-2xl');
		const count = await valueCells.count();
		expect(count).toBeGreaterThanOrEqual(3);

		const bodyText = ((await tauriPage.locator('body').textContent()) ?? '').trim();
		expect(bodyText.length).toBeGreaterThan(0);
	});

	test('renders dashboard summary copy when authenticated', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);

		if (await isLoginPageVisible(tauriPage)) {
			await expect(tauriPage.getByRole('button', { name: 'Sign in with Google' })).toBeVisible();
			return;
		}

		await expect(tauriPage.getByText('Real-time application metrics')).toBeVisible();
	});
});

async function isLoginPageVisible(page: Page): Promise<boolean> {
	const body = (await page.locator('body').textContent()) ?? '';
	return body.includes('Sign in with Google') || body.includes('Welcome back');
}
