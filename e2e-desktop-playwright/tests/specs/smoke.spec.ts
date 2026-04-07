import { test, expect } from '../fixtures/tauri';

test('desktop runtime loads login shell', async ({ tauriPage }) => {
  await tauriPage.waitForFunction('document.readyState === "complete"', 30_000);

  const loginTitle = tauriPage.getByText('Sign in with Google');
  const appTitle = tauriPage.getByText('Tauri App');

  const hasLogin = await loginTitle.isVisible().catch(() => false);
  const hasAppTitle = await appTitle.isVisible().catch(() => false);

  expect(hasLogin || hasAppTitle).toBeTruthy();
});
