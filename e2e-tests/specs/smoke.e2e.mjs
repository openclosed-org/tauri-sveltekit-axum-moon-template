import assert from 'node:assert/strict';

describe('Tauri Desktop Smoke', () => {
  it('loads login shell in desktop runtime', async () => {
    const heading = await $('//h1[contains(., "Tauri App")]');
    await heading.waitForDisplayed({ timeout: 20000 });

    const signInButton = await $('//button[contains(., "Sign in with Google")]');
    await signInButton.waitForDisplayed({ timeout: 20000 });

    const title = await browser.getTitle();
    assert.ok(title.length > 0);
  });
});
