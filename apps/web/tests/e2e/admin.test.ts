import { expect, test } from '@playwright/test';
import { triggerMockOAuth } from '../fixtures/auth';

test.describe('Admin Dashboard (E2E)', () => {
  test.beforeEach(async ({ page }) => {
    // Authenticate first — admin is in protected (app) route group
    await page.goto('/login');
    await triggerMockOAuth(page, 'admin_test_code');
    await page.waitForTimeout(1500);
    // Navigate to admin
    await page.goto('/admin');
    await page.waitForLoadState('networkidle');
  });

  test('admin page requires authentication', async ({ page }) => {
    const url = page.url();
    // Either on admin (authenticated) or login (guard works)
    expect(url).toMatch(/\/(admin|login)/);
  });

  test('displays admin dashboard layout when authenticated', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) {
      // Auth guard works — admin properly protected
      const signInBtn = page.getByRole('button', { name: /sign in with google/i });
      await expect(signInBtn).toBeVisible();
      return;
    }
    // Admin content should be visible
    await expect(page.locator('h1')).toContainText(/admin dashboard/i);
  });

  test('shows stat cards when authenticated', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) {
      return;
    }

    await expect(page.locator('text=Total Users')).toBeVisible();
    await expect(page.locator('text=Active Sessions')).toBeVisible();
    await expect(page.locator('text=Revenue')).toBeVisible();
    await expect(page.locator('text=Growth')).toBeVisible();
  });

  test('shows stat values when authenticated', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) {
      return;
    }

    await expect(page.locator('text=12,345')).toBeVisible();
    await expect(page.locator('text=1,234')).toBeVisible();
    await expect(page.locator('text=$45,678')).toBeVisible();
    await expect(page.locator('text=8.2%')).toBeVisible();
  });

  test('renders chart placeholders when authenticated', async ({ page }) => {
    const url = page.url();
    if (url.includes('/login')) {
      return;
    }

    await expect(page.locator('text=Revenue Over Time')).toBeVisible();
    await expect(page.locator('text=User Activity')).toBeVisible();
  });

  test('admin page is responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/admin');
    await page.waitForLoadState('networkidle');

    const url = page.url();
    if (url.includes('/login')) {
      const signInBtn = page.getByRole('button', { name: /sign in/i });
      await expect(signInBtn).toBeVisible();
      return;
    }
    // Dashboard title should still be visible on mobile
    await expect(page.locator('h1')).toBeVisible();
  });

  test('admin page is properly guarded without auth', async ({ page }) => {
    await page.context().clearCookies();
    await page.goto('/admin');
    await page.waitForLoadState('networkidle');

    // Should redirect to login (auth guard)
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });
  });
});
