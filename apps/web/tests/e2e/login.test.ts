import { expect, test, triggerMockOAuth } from '../fixtures/auth';

test.describe('Login Flow (E2E)', () => {
  test('shows login page with Google sign-in button', async ({ page }) => {
    await page.goto('/login');

    // Check for Google sign-in button presence
    const signInBtn = page.getByRole('button', { name: /sign in with google/i });
    await expect(signInBtn).toBeVisible();

    // Check page has proper heading
    await expect(page.locator('h1')).toContainText(/tauri app/i);
  });

  test('shows welcome text on login page', async ({ page }) => {
    await page.goto('/login');

    await expect(page.locator('text=Welcome back')).toBeVisible();
  });

  test('shows disabled email input', async ({ page }) => {
    await page.goto('/login');

    const emailInput = page.locator('input[type="email"]');
    await expect(emailInput).toBeVisible();
    await expect(emailInput).toBeDisabled();
  });

  test('login page is responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/login');

    // Login page should still be usable on mobile
    const signInBtn = page.getByRole('button', { name: /sign in/i });
    await expect(signInBtn).toBeVisible();
  });

  test('mock OAuth callback can be triggered', async ({ page }) => {
    await page.goto('/login');

    // Verify we can dispatch the mock deep-link event without error
    await triggerMockOAuth(page, 'test_mock_code');

    // Page should still be functional (no crash)
    const signInBtn = page.getByRole('button', { name: /sign in with google/i });
    await expect(signInBtn).toBeVisible();
  });
});
