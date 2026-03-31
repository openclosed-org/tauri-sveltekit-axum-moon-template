import { render, cleanup } from '@testing-library/svelte';
import { describe, it, expect, vi, afterEach } from 'vitest';

// Mock Tauri and SvelteKit dependencies
vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(() => Promise.resolve(() => {}))
}));

vi.mock('$lib/ipc/auth', () => ({
	getSession: vi.fn(() => Promise.resolve(null)),
	startOAuth: vi.fn(() => Promise.resolve()),
	clearAuthStore: vi.fn(() => Promise.resolve()),
	handleOAuthCallback: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn(() => Promise.resolve())
}));

describe('LoginPage', () => {
	afterEach(() => {
		cleanup();
	});

	it('has a heading element', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const heading = container.querySelector('h1');
		expect(heading).toBeTruthy();
		expect(heading?.textContent).toContain('Tauri App');
	});

	it('renders welcome text in a paragraph', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const paragraphs = container.querySelectorAll('p');
		const welcomeP = Array.from(paragraphs).find((p) => p.textContent?.includes('Welcome back'));
		expect(welcomeP).toBeTruthy();
	});

	it('has a sign-in button', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const buttons = container.querySelectorAll('button');
		const signInBtn = Array.from(buttons).find((b) =>
			b.textContent?.toLowerCase().includes('sign in')
		);
		expect(signInBtn).toBeTruthy();
	});

	it('has a disabled email input', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const emailInput = container.querySelector('input[type="email"]');
		expect(emailInput).toBeTruthy();
		expect(emailInput?.hasAttribute('disabled')).toBe(true);
	});

	it('shows terms of service text', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const allText = container.textContent;
		expect(allText).toContain('Terms of Service');
	});

	it('has or divider text', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		const spans = container.querySelectorAll('span');
		const orSpan = Array.from(spans).find((s) => s.textContent?.trim() === 'or');
		expect(orSpan).toBeTruthy();
	});

	it('does not show error initially', async () => {
		const LoginPage = (await import('../../src/routes/(auth)/login/+page.svelte')).default;
		const { container } = render(LoginPage);
		// No red error div should be present
		const allText = container.textContent;
		expect(allText).not.toContain('auth.authError');
	});
});
