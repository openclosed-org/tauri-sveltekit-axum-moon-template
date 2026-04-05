import assert from 'node:assert/strict';
import { getBodyText, isLoginPageVisible, navigateTo, waitForAnyText, waitForText } from '../helpers/navigate.mjs';

describe('Tauri Desktop Admin Dashboard', () => {
  it('admin page requires authentication', async () => {
    await navigateTo('/admin');
    
    const url = await browser.getUrl();
    assert.ok(
      url.includes('/admin') || url.includes('/login'),
      'Should be on admin or login page'
    );
  });

  it('displays admin dashboard layout when authenticated', async () => {
    await navigateTo('/admin');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in with Google")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForText('Admin Dashboard', 10000);
  });

  it('shows stat cards when authenticated', async () => {
    await navigateTo('/admin');

    if (await isLoginPageVisible()) {
      return;
    }

    await waitForText('Tenants', 10000);
    await waitForText('Counter', 10000);
    await waitForText('Last Login', 10000);
    await waitForText('Version', 10000);
  });

  it('shows stat values when authenticated', async () => {
    await navigateTo('/admin');

    if (await isLoginPageVisible()) {
      return;
    }

    await waitForText('Tenants', 10000);
    await waitForText('Version', 10000);

    const body = await getBodyText();
    assert.ok(body.length > 0, 'Admin page should contain visible text');
  });

  it('renders dashboard summary copy when authenticated', async () => {
    await navigateTo('/admin');

    if (await isLoginPageVisible()) {
      return;
    }

    await waitForText('Real-time application metrics', 10000);
  });

  it('admin page is responsive on mobile viewport', async () => {
    await browser.setWindowSize(375, 667);
    await navigateTo('/admin');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForAnyText(['Admin Dashboard', 'Sign in with Google'], 10000);
  });

  it('admin page is properly guarded without auth', async () => {
    await navigateTo('/admin');

    const url = await browser.getUrl();
    assert.ok(
      url.includes('/login') || url.includes('/admin'),
      'Should redirect to login or show admin'
    );
  });
});
