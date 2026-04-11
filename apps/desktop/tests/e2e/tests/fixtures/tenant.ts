import type { Locator, Page } from '@playwright/test';
import { triggerMockOAuth } from './auth';

const APP_BASE_URL = 'http://localhost:5173';

function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

type TenantLabel = 'tenant-A' | 'tenant-B';

export interface TenantIdentity {
	label: TenantLabel;
	userSub: string;
	userName: string;
	mockCode: string;
}

export const TENANT_A: TenantIdentity = {
	label: 'tenant-A',
	userSub: 'tenant_a_user',
	userName: 'Tenant A User',
	mockCode: 'tenant_a_user'
};

export const TENANT_B: TenantIdentity = {
	label: 'tenant-B',
	userSub: 'tenant_b_user',
	userName: 'Tenant B User',
	mockCode: 'tenant_b_user'
};

export const TENANT_LABELS = [TENANT_A.label, TENANT_B.label] as const;

const TENANTS = [TENANT_A, TENANT_B] as const;
const TENANT_INIT_URL = 'http://127.0.0.1:3001/api/tenant/init';
const RETRY_LIMIT = 3;
const RETRY_DELAY_MS = 800;

type InitTenantResponse = {
	tenant_id?: string;
	role?: string;
	created?: boolean;
};

async function callTenantInit(page: Page, tenant: TenantIdentity): Promise<void> {
	const response = await fetch(TENANT_INIT_URL, {
		method: 'POST',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify({
			user_sub: tenant.userSub,
			user_name: tenant.userName
		})
	});

	const body = (await response.json().catch(() => ({}))) as InitTenantResponse;
	if (response.status !== 200 || typeof body.tenant_id !== 'string' || body.tenant_id.length === 0) {
		throw new Error(
			`[${tenant.label}] tenant init failed: status=${response.status}, body=${JSON.stringify(body)}`
		);
	}
}

function toBase64Url(input: string): string {
	return Buffer.from(input).toString('base64').replace(/=/g, '').replace(/\+/g, '-').replace(/\//g, '_');
}

function makeTenantToken(userSub: string): string {
	const header = toBase64Url(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
	const payload = toBase64Url(JSON.stringify({ sub: userSub, exp: 4_102_444_800 }));
	return `${header}.${payload}.desktop-e2e-fixture`;
}

async function withRetry<T>(operationName: string, operation: () => Promise<T>): Promise<T> {
	let lastError: unknown;
	for (let attempt = 1; attempt <= RETRY_LIMIT; attempt += 1) {
		try {
			return await operation();
		} catch (error) {
			lastError = error;
			if (attempt === RETRY_LIMIT) {
				break;
			}
			await sleep(RETRY_DELAY_MS);
		}
	}

	throw new Error(
		`${operationName} failed after ${RETRY_LIMIT} attempts: ${lastError instanceof Error ? lastError.message : String(lastError)}`
	);
}

export async function initTenantPair(page: Page): Promise<void> {
	for (const tenant of TENANTS) {
		await withRetry(`[${tenant.label}] tenant init`, () => callTenantInit(page, tenant));
	}
}

async function openCounterPageAsTenant(page: Page, tenant: TenantIdentity): Promise<void> {
	await (page as any).goto(`${APP_BASE_URL}/login`);
	await triggerMockOAuth(page, tenant.mockCode);
	await sleep(800);
	await (page as any).goto(`${APP_BASE_URL}/counter`);
	await page.waitForFunction('document.readyState === "complete"', 10_000);
}

async function clickWhenReady(button: Locator): Promise<void> {
	await button.waitFor({ state: 'visible', timeout: 10_000 });
	if (!(await button.isEnabled())) {
		throw new Error('counter control is disabled');
	}
	await button.click();
}

async function ensureCounterIsReset(page: Page, tenant: TenantIdentity): Promise<void> {
	const resetButton = page.getByRole('button', { name: 'Reset' });
	const resetVisible = await resetButton.isVisible().catch(() => false);
	if (!resetVisible) {
		throw new Error(`[${tenant.label}] counter reset failed: reset button unavailable`);
	}

	await clickWhenReady(resetButton);

	const counterDisplay = page.locator('.font-mono');
	await counterDisplay.waitFor({ state: 'visible', timeout: 10000 });
	const value = (await counterDisplay.textContent())?.trim();
	if (value !== '0') {
		throw new Error(`[${tenant.label}] counter reset failed: expected 0, got ${value ?? 'empty'}`);
	}
}

export async function resetTenantPair(page: Page): Promise<void> {
	await initTenantPair(page);

	for (const tenant of TENANTS) {
		try {
			await withRetry(`[${tenant.label}] open counter page`, () => openCounterPageAsTenant(page, tenant));
			await ensureCounterIsReset(page, tenant);
		} catch (error) {
			throw new Error(
				`[${tenant.label}] reset tenant counter failed: ${error instanceof Error ? error.message : String(error)}`
			);
		}
	}
}

export async function resetTenantPairCounter(page: Page): Promise<void> {
	await resetTenantPair(page);
}

export function buildTenantAuthHeaders(userSub: string): Record<string, string> {
	return {
		Authorization: `Bearer ${makeTenantToken(userSub)}`,
		'content-type': 'application/json'
	};
}

export async function waitForCounterControlsReady(page: Page): Promise<{
	decrementButton: Locator;
	incrementButton: Locator;
	resetControl: Locator;
}> {
	const decrementButton = page.locator('button:has(svg[data-lucide="minus"])').first();
	const incrementButton = page.locator('button:has(svg[data-lucide="plus"])').first();
	const resetControl = page.locator('button:has(svg[data-lucide="rotate-ccw"])').first();

	await decrementButton.waitFor({ state: 'visible', timeout: 10_000 });
	await incrementButton.waitFor({ state: 'visible', timeout: 10_000 });
	await resetControl.waitFor({ state: 'visible', timeout: 10_000 });

	if (!(await decrementButton.isEnabled())) {
		throw new Error('decrement button is disabled');
	}
	if (!(await incrementButton.isEnabled())) {
		throw new Error('increment button is disabled');
	}
	if (!(await resetControl.isEnabled())) {
		throw new Error('reset button is disabled');
	}

	return { decrementButton, incrementButton, resetControl };
}
