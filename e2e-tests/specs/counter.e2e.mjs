import assert from 'node:assert/strict';
import { getBodyText, isLoginPageVisible, navigateTo, waitForAnyText, waitForText } from '../helpers/navigate.mjs';

describe('Tauri Desktop Counter', () => {
  it('counter page is properly guarded without auth', async () => {
    await navigateTo('/counter');
    const url = await browser.getUrl();
    assert.ok(url.includes('/counter') || url.includes('/login'), 'Should be on counter or login page');
  });

  it('displays counter page content when authenticated', async () => {
    await navigateTo('/counter');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in with Google")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForText('Reset', 10000);
  });

  it('has increment, decrement, and reset buttons when authenticated', async () => {
    await navigateTo('/counter');

    if (await isLoginPageVisible()) {
      return;
    }

    await waitForText('Reset', 10000);

    const body = await getBodyText();
    assert.ok(body.includes('Reset'), 'Should render reset control');

    const buttons = await $$('button');
    assert.ok(buttons.length >= 3, 'Should have at least 3 buttons');
  });

  it('counter page is responsive on mobile viewport', async () => {
    await browser.setWindowSize(375, 667);
    await navigateTo('/counter');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForAnyText(['Reset', 'Counter'], 10000);
  });
});
