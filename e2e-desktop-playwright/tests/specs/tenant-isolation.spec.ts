import { test, expect } from '../fixtures/tauri';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Tenant Isolation', () => {
	test('RED: migration sentinel for tenant isolation spec', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/counter`);
		await expect(tauriPage.getByText('MIGRATION_RED_SENTINEL_TENANT')).toBeVisible();
	});
});
