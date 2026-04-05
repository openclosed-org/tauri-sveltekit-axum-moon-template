import assert from 'node:assert/strict';
import { getBodyText, waitForAnyText } from '../helpers/navigate.mjs';

describe('Tauri Desktop Smoke', () => {
  it('loads login shell in desktop runtime', async () => {
    // Wait for the app to fully initialize and render
    // Tauri app starts with the main window, but the webview may take time to load
    await browser.waitUntil(
      async () => {
        try {
          const url = await browser.getUrl();
          if (url === 'about:blank') {
            return false;
          }
          // Try to find any content on the page
          const title = await browser.getTitle();
          return title.length > 0;
        } catch {
          return false;
        }
      },
      {
        timeout: 30000,
        timeoutMsg: 'App did not initialize within 30 seconds',
        interval: 500,
      }
    );

    await waitForAnyText(['Tauri App', 'Sign in with Google', 'Counter', 'Admin Dashboard'], 30000);

    const body = await getBodyText();
    assert.ok(body.length > 0, 'Body should contain visible text');

    const title = await browser.getTitle();
    assert.ok(title.length > 0);
  });
});
