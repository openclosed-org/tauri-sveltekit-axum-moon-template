import { test, expect } from '../fixtures/tauri';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Admin Dashboard', () => {
	test('RED: migration sentinel for admin spec', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/admin`);
		await expect(tauriPage.getByText('MIGRATION_RED_SENTINEL_ADMIN')).toBeVisible();
	});
});
