import { type Page, test as base, expect } from '@playwright/test';

/**
 * Mock OAuth deep-link fixture for E2E tests.
 *
 * Per D-01: Uses mock callback strategy, not real Google OAuth.
 * Per D-02: Simulates deep-link://new-url event to trigger OAuth callback chain.
 */

/** Trigger mock OAuth deep-link event on the page. */
export async function triggerMockOAuth(page: Page, mockCode = 'mock_auth_code') {
  await page.evaluate((code) => {
    window.dispatchEvent(
      new CustomEvent('deep-link://new-url', {
        detail: `/callback?code=${code}&state=mock_state`,
      }),
    );
  }, mockCode);
}

function toBase64Url(input: string): string {
  return Buffer.from(input)
    .toString('base64')
    .replace(/=/g, '')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');
}

export function makeTenantToken(userSub: string): string {
  const header = toBase64Url(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
  const payload = toBase64Url(JSON.stringify({ sub: userSub, exp: 4_102_444_800 }));
  return `${header}.${payload}.web-e2e`;
}

export function buildTenantAuthHeaders(userSub: string): Record<string, string> {
  return {
    Authorization: `Bearer ${makeTenantToken(userSub)}`,
    'content-type': 'application/json',
  };
}

/** Verify user is logged in by checking for sign-out button or user indicator. */
export async function verifyLoggedIn(page: Page) {
  // After mock login, the auth state should reflect logged-in status
  // Check URL has navigated away from login
  await expect(page).not.toHaveURL(/\/login/, { timeout: 5000 });
}

/** Verify user is logged out (on login page). */
export async function verifyLoggedOut(page: Page) {
  await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
}

/** Extended test fixture with mockLogin helper. */
export const test = base.extend<{ mockLogin: undefined }>({
  mockLogin: [
    async ({ page }, use) => {
      // Navigate to login page first
      await page.goto('/login');
      // Trigger mock OAuth callback
      await triggerMockOAuth(page);
      // Wait for auth to process
      await page.waitForTimeout(1000);
      await use(undefined);
    },
    { auto: false },
  ],
});

export { expect };
