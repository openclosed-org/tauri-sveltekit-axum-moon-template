import { test, expect } from '../fixtures/tauri';

const APP_BASE_URL = 'http://localhost:5173';

test.describe('Tauri Desktop Agent Chat', () => {
	test('RED: migration sentinel for agent spec', async ({ tauriPage }) => {
		await tauriPage.goto(`${APP_BASE_URL}/agent`);
		await expect(tauriPage.getByText('MIGRATION_RED_SENTINEL_AGENT')).toBeVisible();
	});
});
