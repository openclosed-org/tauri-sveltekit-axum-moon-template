import type { Page } from '@playwright/test';
import { triggerMockOAuth } from './auth';

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

type InitTenantResponse = {
	tenant_id?: string;
	role?: string;
	created?: boolean;
};

async function callTenantInit(page: Page, tenant: TenantIdentity): Promise<void> {
	const response = await page.request.post(TENANT_INIT_URL, {
		headers: {
			'content-type': 'application/json'
		},
		data: {
			user_sub: tenant.userSub,
			user_name: tenant.userName
		}
	});

	const body = (await response.json().catch(() => ({}))) as InitTenantResponse;
	if (response.status() !== 200 || typeof body.tenant_id !== 'string' || body.tenant_id.length === 0) {
		throw new Error(
			`[${tenant.label}] tenant init failed: status=${response.status()}, body=${JSON.stringify(body)}`
		);
	}
}

export async function initTenantPair(page: Page): Promise<void> {
	for (const tenant of TENANTS) {
		await callTenantInit(page, tenant);
	}
}

async function openCounterPageAsTenant(page: Page, tenant: TenantIdentity): Promise<void> {
	await page.goto('/login');
	await triggerMockOAuth(page, tenant.mockCode);
	await page.waitForTimeout(800);
	await page.goto('/counter');
	await page.waitForLoadState('networkidle');
}

async function ensureCounterIsReset(page: Page, tenant: TenantIdentity): Promise<void> {
	const buttons = page.locator('button');
	const count = await buttons.count();
	if (count < 3) {
		throw new Error(`[${tenant.label}] counter reset failed: reset button unavailable`);
	}

	await buttons.nth(2).click();

	const counterDisplay = page.locator('.font-mono');
	await counterDisplay.waitFor({ state: 'visible', timeout: 10000 });
	const value = (await counterDisplay.textContent())?.trim();
	if (value !== '0') {
		throw new Error(`[${tenant.label}] counter reset failed: expected 0, got ${value ?? 'empty'}`);
	}
}

export async function resetTenantPairCounter(page: Page): Promise<void> {
	await initTenantPair(page);

	for (const tenant of TENANTS) {
		try {
			await openCounterPageAsTenant(page, tenant);
			await ensureCounterIsReset(page, tenant);
		} catch (error) {
			throw new Error(
				`[${tenant.label}] reset tenant counter failed: ${error instanceof Error ? error.message : String(error)}`
			);
		}
	}
}
