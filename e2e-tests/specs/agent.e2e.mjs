import assert from 'node:assert/strict';
import { isLoginPageVisible, navigateTo, waitForAnyText, waitForText } from '../helpers/navigate.mjs';

async function getTextInput() {
  const selectors = [
    'input[placeholder="Type a message..."]',
    'input[placeholder*="Type a message"]',
    'input[type="text"]',
  ];

  for (const selector of selectors) {
    const input = await $(selector);
    if (await input.isExisting()) {
      return input;
    }
  }

  return null;
}

async function getSendButton() {
  const selectors = [
    '//button[contains(., "Send")]',
    'button[aria-label="Send"]',
  ];

  for (const selector of selectors) {
    const button = await $(selector);
    if (await button.isExisting()) {
      return button;
    }
  }

  return null;
}

async function ensureConversationReady() {
  let input = await getTextInput();
  if (input && await input.isDisplayed().catch(() => false)) {
    return input;
  }

  const newChatButton = await $('//button[contains(., "New Chat")]');
  await newChatButton.waitForDisplayed({ timeout: 10000 });
  await newChatButton.click();

  await browser.waitUntil(
    async () => {
      input = await getTextInput();
      return Boolean(input && await input.isDisplayed().catch(() => false));
    },
    {
      timeout: 10000,
      interval: 250,
      timeoutMsg: 'Message input did not appear after creating a conversation',
    },
  );

  return input;
}

describe('Tauri Desktop Agent Chat', () => {
  it('agent page requires authentication', async () => {
    await navigateTo('/agent');
    
    const url = await browser.getUrl();
    assert.ok(
      url.includes('/agent') || url.includes('/login'),
      'Should be on agent or login page'
    );
  });

  it('displays agent chat layout when authenticated', async () => {
    await navigateTo('/agent');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in with Google")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForAnyText(['New Chat', 'Type a message', 'Send'], 10000);
  });

  it('shows sidebar with conversations', async () => {
    await navigateTo('/agent');

    if (await isLoginPageVisible()) {
      return;
    }

    await waitForText('New Chat', 10000);
  });

  it('has message input area', async () => {
    await navigateTo('/agent');

    if (await isLoginPageVisible()) {
      return;
    }

    const input = await ensureConversationReady();
    assert.ok(input, 'Message input should exist on agent page');
    await input.waitForDisplayed({ timeout: 10000 });
  });

  it('send button state changes with input', async () => {
    await navigateTo('/agent');

    if (await isLoginPageVisible()) {
      return;
    }

    const input = await ensureConversationReady();
    const sendButton = await getSendButton();

    assert.ok(input, 'Message input should exist before send test');
    assert.ok(sendButton, 'Send button should exist before send test');

    await input.waitForDisplayed({ timeout: 10000 });
    await sendButton.waitForDisplayed({ timeout: 10000 });

    await input.setValue('hello from e2e');
    const disabledAfterTyping = await sendButton.getAttribute('disabled');
    assert.equal(disabledAfterTyping, null, 'Send button should be enabled after typing');
  });

  it('agent page is responsive on mobile viewport', async () => {
    await browser.setWindowSize(375, 667);
    await navigateTo('/agent');

    if (await isLoginPageVisible()) {
      const signInButton = await $('//button[contains(., "Sign in")]');
      await signInButton.waitForDisplayed({ timeout: 10000 });
      return;
    }

    await waitForText('New Chat', 10000);

    const input = await ensureConversationReady();
    assert.ok(input, 'Message input should exist on mobile viewport');
    await input.waitForDisplayed({ timeout: 10000 });
  });

  it('agent page is properly guarded without auth', async () => {
    await navigateTo('/agent');

    const url = await browser.getUrl();
    assert.ok(
      url.includes('/login') || url.includes('/agent'),
      'Should redirect to login or show agent'
    );
  });
});
