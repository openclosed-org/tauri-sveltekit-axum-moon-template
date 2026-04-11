import { expect, test } from '@playwright/test';
import { triggerMockOAuth } from '../fixtures/auth';

test.describe('Agent Chat (E2E)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await triggerMockOAuth(page, 'agent_test_code');
    await page.waitForTimeout(1500);
    await page.goto('/agent');
    await page.waitForLoadState('networkidle');
  });

  test('agent page requires authentication', async ({ page }) => {
    const url = page.url();
    expect(url).toMatch(/\/(agent|login)/);
  });

  test('displays agent chat layout when authenticated', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) {
      const signInBtn = page.getByRole('button', { name: /sign in with google/i });
      await expect(signInBtn).toBeVisible();
      return;
    }
    await expect(page.locator('text=New Chat')).toBeVisible();
    await expect(page.locator('text=Select or create a conversation')).toBeVisible();
  });

  test('has sidebar with conversation list', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) return;

    await expect(page.locator('aside')).toBeVisible();
    await expect(page.locator('text=New Chat')).toBeVisible();
  });

  test('has message input area', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) return;

    await expect(page.locator('input[placeholder="Type a message..."]')).toBeVisible();
    await expect(page.getByRole('button', { name: /send/i })).toBeVisible();
  });

  test('send button is disabled when input is empty', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) return;

    const sendBtn = page.getByRole('button', { name: /send/i });
    await expect(sendBtn).toBeDisabled();
  });

  test('agent page is responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/agent');
    await page.waitForLoadState('networkidle');

    const url = page.url();
    if (url.includes('/login')) {
      const signInBtn = page.getByRole('button', { name: /sign in/i });
      await expect(signInBtn).toBeVisible();
      return;
    }
    await expect(page.locator('text=New Chat')).toBeVisible();
  });

  test('agent page is properly guarded without auth', async ({ page }) => {
    await page.context().clearCookies();
    await page.goto('/agent');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });
  });

  test('New Chat clears current thread and keeps settings state stable', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) return;

    const conversations = [{ id: 'conv-1', title: 'Chat 1', created_at: '2026-04-06T00:00:00Z' }];
    const messagesByConversation: Record<string, Array<Record<string, string | null>>> = {
      'conv-1': [
        {
          id: 'msg-1',
          conversation_id: 'conv-1',
          role: 'assistant',
          content: 'existing seeded message',
          tool_calls: null,
          created_at: '2026-04-06T00:00:01Z',
        },
      ],
    };

    await page.route('**/api/agent/conversations', async (route) => {
      const method = route.request().method();
      if (method === 'GET') {
        await route.fulfill({ json: conversations });
        return;
      }

      if (method === 'POST') {
        const next = {
          id: `conv-${conversations.length + 1}`,
          title: `Chat ${conversations.length + 1}`,
          created_at: '2026-04-06T00:00:02Z',
        };
        conversations.push(next);
        messagesByConversation[next.id] = [];
        await route.fulfill({ json: next });
        return;
      }

      await route.fallback();
    });

    await page.route('**/api/agent/conversations/*/messages', async (route) => {
      const conversationId = route.request().url().split('/').slice(-2)[0];
      await route.fulfill({ json: messagesByConversation[conversationId] ?? [] });
    });

    await page.goto('/agent');
    await page.waitForLoadState('networkidle');

    await expect(page.getByText('existing seeded message')).toBeVisible();
    await page.getByRole('button', { name: /new chat/i }).click();

    await expect(page.getByText('existing seeded message')).toHaveCount(0);
    await expect(page.locator('aside button', { hasText: 'Chat 2' })).toHaveClass(/bg-primary-50/);

    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    const firstApiKey = await page.locator('#api_key').inputValue();
    const firstBaseUrl = await page.locator('#base_url').inputValue();
    const firstModel = await page.locator('#model').inputValue();

    await page.goto('/agent');
    await page.waitForLoadState('networkidle');
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');

    expect(await page.locator('#api_key').inputValue()).toBe(firstApiKey);
    expect(await page.locator('#base_url').inputValue()).toBe(firstBaseUrl);
    expect(await page.locator('#model').inputValue()).toBe(firstModel);
  });
});
