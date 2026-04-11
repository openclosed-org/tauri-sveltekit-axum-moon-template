import { test, expect, type APIRequestContext } from '@playwright/test';
import { TENANT_A, TENANT_B, buildTenantAuthHeaders } from '../fixtures/tenant';
import { spawn, type ChildProcess } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const API_BASE_URL = 'http://127.0.0.1:3001';
const TENANT_INIT_URL = `${API_BASE_URL}/api/tenant/init`;
const COUNTER_RESET_URL = `${API_BASE_URL}/api/counter/reset`;
const COUNTER_INCREMENT_URL = `${API_BASE_URL}/api/counter/increment`;
const COUNTER_VALUE_URL = `${API_BASE_URL}/api/counter/value`;
const AGENT_CONVERSATIONS_URL = `${API_BASE_URL}/api/agent/conversations`;
const RETRY_LIMIT = 3;
const RETRY_DELAY_MS = 1200;
const API_READY_URL = `${API_BASE_URL}/readyz`;

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const workspaceRoot = path.resolve(__dirname, '..', '..', '..');

let ownedApiProcess: ChildProcess | null = null;

test.describe('Tauri Desktop Tenant Isolation', () => {
	test.describe.configure({ mode: 'serial' });
	const BASELINE = 0;
	const TENANT_A_WRITES = 2;

	test.beforeAll(async () => {
		await ensureApiReady();
	});

	test.afterAll(async () => {
		stopOwnedApiProcess();
	});

	test.beforeEach(async ({ request }) => {
		await withRetry('resetTenantPair', () => resetTenantPair(request, BASELINE));
	});

	test('uses the same fixed tenant pair as web harness', async () => {
		expect(TENANT_A.userSub).toBe('tenant_a_user');
		expect(TENANT_B.userSub).toBe('tenant_b_user');
	});

	test('tenant-1 write does not alter tenant-2 value (run-1)', async ({ request }) => {
		await assertIsolationFlow(request, 'run-1', BASELINE, TENANT_A_WRITES);
	});

	test('tenant-1 write does not alter tenant-2 value (run-2, same seed)', async ({ request }) => {
		await assertIsolationFlow(request, 'run-2', BASELINE, TENANT_A_WRITES);
	});

	test('settings and agent conversation are isolated by tenant+user', async ({ request, page }) => {
		await assertSettingsIsolation(request);
		await assertAgentIsolation(page);
	});

	test('theme preference is isolated per user context', async ({ page, context }) => {
		await page.goto(API_READY_URL);
		await page.evaluate((value) => localStorage.setItem('theme-preference', value), 'dark');
		const themeA = await page.evaluate(() => localStorage.getItem('theme-preference'));
		expect(themeA, `[${TENANT_A.label}] expected stored theme to be dark`).toBe('dark');

		const secondContext = await context.browser()?.newContext();
		if (!secondContext) {
			throw new Error(`[${TENANT_B.label}] failed to create isolated browser context`);
		}
		const secondPage = await secondContext.newPage();
		await secondPage.goto(API_READY_URL);
		const themeBBefore = await secondPage.evaluate(() => localStorage.getItem('theme-preference'));
		expect(themeBBefore, `[${TENANT_B.label}] expected no inherited theme from ${TENANT_A.label}`).toBeNull();
		await secondPage.evaluate((value) => localStorage.setItem('theme-preference', value), 'light');
		const themeBAfter = await secondPage.evaluate(() => localStorage.getItem('theme-preference'));
		expect(themeBAfter, `[${TENANT_B.label}] expected stored theme to be light`).toBe('light');

		const themeAAfter = await page.evaluate(() => localStorage.getItem('theme-preference'));
		expect(themeAAfter, `[${TENANT_A.label}] leaked theme after ${TENANT_B.label} mutation`).toBe('dark');

		await secondContext.close();
	});
});

function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

async function withRetry<T>(operationName: string, operation: () => Promise<T>): Promise<T> {
	let lastError: unknown;
	for (let attempt = 1; attempt <= RETRY_LIMIT; attempt += 1) {
		try {
			return await operation();
		} catch (error) {
			lastError = error;
			if (attempt === RETRY_LIMIT) break;
			await sleep(RETRY_DELAY_MS);
		}
	}

	throw new Error(
		`[tenant-isolation] ${operationName} failed after ${RETRY_LIMIT} attempts: ${
			lastError instanceof Error ? lastError.message : String(lastError)
		}`
	);
}

async function waitForApiReady(timeoutMs = 120_000): Promise<boolean> {
	const start = Date.now();
	while (Date.now() - start < timeoutMs) {
		try {
			const response = await fetch(API_READY_URL);
			if (response.ok) {
				return true;
			}
		} catch {
			// keep polling
		}

		await sleep(500);
	}

	return false;
}

function runtimeServerBinaryPath(): string {
	const binary = process.platform === 'win32' ? 'runtime_server.exe' : 'runtime_server';
	return path.join(workspaceRoot, 'target', 'debug', binary);
}

async function ensureApiReady(): Promise<void> {
	if (await waitForApiReady(1_000)) {
		return;
	}

	const runtimeBinary = runtimeServerBinaryPath();
	if (existsSync(runtimeBinary)) {
		ownedApiProcess = spawn(runtimeBinary, [], {
			cwd: workspaceRoot,
			stdio: 'ignore',
			shell: false
		});
	} else {
		ownedApiProcess = spawn('cargo', ['run', '-p', 'runtime_server'], {
			cwd: workspaceRoot,
			stdio: 'ignore',
			shell: process.platform === 'win32'
		});
	}

	const ready = await waitForApiReady(120_000);
	if (!ready) {
		stopOwnedApiProcess();
		throw new Error('runtime_server did not become ready at /readyz within timeout');
	}
}

function stopOwnedApiProcess(): void {
	if (!ownedApiProcess) {
		return;
	}

	if (!ownedApiProcess.killed) {
		if (process.platform === 'win32') {
			spawn('taskkill', ['/PID', String(ownedApiProcess.pid), '/F', '/T'], {
				stdio: 'ignore',
				shell: false
			});
		} else {
			ownedApiProcess.kill('SIGTERM');
		}
	}

	ownedApiProcess = null;
}

function authHeaders(userSub: string): Record<string, string> {
	return buildTenantAuthHeaders(userSub);
}

async function initTenantPair(request: APIRequestContext): Promise<void> {
	for (const tenant of [TENANT_A, TENANT_B]) {
		const response = await request.post(TENANT_INIT_URL, {
			headers: authHeaders(tenant.userSub),
			data: { user_sub: tenant.userSub, user_name: tenant.userName }
		});
		const body = (await response.json().catch(() => ({}))) as { tenant_id?: string };
		if (response.status() !== 200 || typeof body.tenant_id !== 'string' || body.tenant_id.length === 0) {
			throw new Error(
				`[${tenant.label}] tenant init failed: status=${response.status()}, body=${JSON.stringify(body)}`
			);
		}
	}
}

async function readTenantCounter(request: APIRequestContext, userSub: string): Promise<number> {
	const response = await request.get(COUNTER_VALUE_URL, {
		headers: authHeaders(userSub)
	});
	const body = (await response.json().catch(() => ({}))) as { value?: number };
	if (response.status() !== 200 || typeof body.value !== 'number') {
		throw new Error(`read counter failed for ${userSub}: status=${response.status()}, body=${JSON.stringify(body)}`);
	}
	return body.value;
}

async function incrementTenantCounter(request: APIRequestContext, userSub: string, times = 1): Promise<void> {
	for (let i = 0; i < times; i += 1) {
		const response = await request.post(COUNTER_INCREMENT_URL, {
			headers: authHeaders(userSub)
		});
		const body = (await response.json().catch(() => ({}))) as { value?: number };
		if (response.status() !== 200 || typeof body.value !== 'number') {
			throw new Error(
				`increment failed for ${userSub} at step ${i + 1}: status=${response.status()}, body=${JSON.stringify(body)}`
			);
		}
	}
}

async function resetTenantPair(request: APIRequestContext, seedValue = 0): Promise<void> {
	await initTenantPair(request);

	for (const tenant of [TENANT_A, TENANT_B]) {
		const resetResponse = await request.post(COUNTER_RESET_URL, {
			headers: authHeaders(tenant.userSub)
		});
		const resetBody = (await resetResponse.json().catch(() => ({}))) as { value?: number };
		if (resetResponse.status() !== 200 || typeof resetBody.value !== 'number') {
			throw new Error(
				`[${tenant.label}] reset failed: status=${resetResponse.status()}, body=${JSON.stringify(resetBody)}`
			);
		}

		if (seedValue > 0) {
			await incrementTenantCounter(request, tenant.userSub, seedValue);
		}

		const current = await readTenantCounter(request, tenant.userSub);
		if (current !== seedValue) {
			throw new Error(`[${tenant.label}] reset baseline mismatch: expected ${seedValue}, got ${current}`);
		}
	}
}

async function assertIsolationFlow(request: APIRequestContext, runLabel: string, seed: number, writes: number): Promise<void> {
	const tenantAStart = await withRetry(`${runLabel}:readTenantCounter(tenant-1:start)`, () =>
		readTenantCounter(request, TENANT_A.userSub)
	);
	expect(tenantAStart, `[${runLabel}] tenant-1 baseline mismatch: expected ${seed}, got ${tenantAStart}`).toBe(seed);

	await withRetry(`${runLabel}:incrementTenantCounter(tenant-1)`, () =>
		incrementTenantCounter(request, TENANT_A.userSub, writes)
	);

	const tenantAAfter = await withRetry(`${runLabel}:readTenantCounter(tenant-1:after)`, () =>
		readTenantCounter(request, TENANT_A.userSub)
	);
	const expectedTenantA = seed + writes;
	expect(
		tenantAAfter,
		`[${runLabel}] tenant-1 write result mismatch: expected ${expectedTenantA}, got ${tenantAAfter}`
	).toBe(expectedTenantA);

	const tenantBAfter = await withRetry(`${runLabel}:readTenantCounter(tenant-2:after)`, () =>
		readTenantCounter(request, TENANT_B.userSub)
	);
	expect(
		tenantBAfter,
		`[${runLabel}] tenant-2 leaked after tenant-1 writes: expected ${seed}, got ${tenantBAfter}`
	).toBe(seed);
}

async function initTenantAndReadConfig(
	request: APIRequestContext,
	tenant: { label: string; userSub: string; userName: string }
): Promise<{ tenantId: string; role: string }> {
	const response = await request.post(TENANT_INIT_URL, {
		headers: authHeaders(tenant.userSub),
		data: { user_sub: tenant.userSub, user_name: tenant.userName }
	});
	const body = (await response.json().catch(() => ({}))) as { tenant_id?: string; role?: string };
	if (response.status() !== 200 || typeof body.tenant_id !== 'string' || typeof body.role !== 'string') {
		throw new Error(
			`[${tenant.label}] settings init failed: status=${response.status()}, body=${JSON.stringify(body)}`
		);
	}
	return { tenantId: body.tenant_id, role: body.role };
}

async function assertSettingsIsolation(request: APIRequestContext): Promise<void> {
	const aFirst = await initTenantAndReadConfig(request, TENANT_A);
	const bFirst = await initTenantAndReadConfig(request, TENANT_B);

	expect(aFirst.role, `[${TENANT_A.label}] expected role to exist`).toBeTruthy();
	expect(bFirst.role, `[${TENANT_B.label}] expected role to exist`).toBeTruthy();
	expect(aFirst.tenantId, `[${TENANT_A.label}] tenant id leaked from ${TENANT_B.label}`).not.toBe(bFirst.tenantId);

	const aSecond = await initTenantAndReadConfig(request, TENANT_A);
	const bSecond = await initTenantAndReadConfig(request, TENANT_B);
	expect(aSecond.tenantId, `[${TENANT_A.label}] tenant config not stable across retries`).toBe(aFirst.tenantId);
	expect(bSecond.tenantId, `[${TENANT_B.label}] tenant config not stable across retries`).toBe(bFirst.tenantId);
}

async function assertAgentIsolation(page: import('@playwright/test').Page): Promise<void> {
	const aTitle = `tenant-a-desktop-conv-${Date.now()}`;
	const userConversations = new Map<string, Array<{ id: string; title: string }>>();

	await page.route('**/agent/conversations', async (route) => {
		const method = route.request().method();
		const authHeader = route.request().headers().authorization ?? '';
		const userKey = authHeader.length > 0 ? authHeader : 'missing-auth';
		const current = userConversations.get(userKey) ?? [];

		if (method === 'GET') {
			await route.fulfill({ status: 200, json: current });
			return;
		}

		if (method === 'POST') {
			const payload = (route.request().postDataJSON() as { title?: string }) ?? {};
			const created = { id: `${userKey}-${current.length + 1}`, title: payload.title ?? 'Untitled' };
			userConversations.set(userKey, [...current, created]);
			await route.fulfill({ status: 200, json: created });
			return;
		}

		await route.fulfill({ status: 405, json: { error: 'Method not allowed' } });
	});

	await page.goto('about:blank');

	const createA = await page.evaluate(
		async ({ url, headers, title }) => {
			const response = await fetch(url, {
				method: 'POST',
				headers,
				body: JSON.stringify({ title })
			});
			return { status: response.status, body: await response.json().catch(() => ({})) };
		},
		{ url: AGENT_CONVERSATIONS_URL, headers: authHeaders(TENANT_A.userSub), title: aTitle }
	);

	if (createA.status !== 200) {
		throw new Error(`[${TENANT_A.label}] create agent conversation failed: status=${createA.status}`);
	}

	const listA = await page.evaluate(
		async ({ url, headers }) => {
			const response = await fetch(url, { method: 'GET', headers });
			return { status: response.status, body: await response.json().catch(() => []) };
		},
		{ url: AGENT_CONVERSATIONS_URL, headers: authHeaders(TENANT_A.userSub) }
	);

	const listABody = (listA.body ?? []) as Array<{ id?: string; title?: string }>;
	expect(listA.status, `[${TENANT_A.label}] list conversations failed`).toBe(200);
	expect(
		listABody.some((item) => item.title === aTitle),
		`[${TENANT_A.label}] cannot observe its own conversation`
	).toBe(true);

	const listB = await page.evaluate(
		async ({ url, headers }) => {
			const response = await fetch(url, { method: 'GET', headers });
			return { status: response.status, body: await response.json().catch(() => []) };
		},
		{ url: AGENT_CONVERSATIONS_URL, headers: authHeaders(TENANT_B.userSub) }
	);
	const listBBody = (listB.body ?? []) as Array<{ id?: string; title?: string }>;
	expect(listB.status, `[${TENANT_B.label}] list conversations failed`).toBe(200);
	expect(
		listBBody.some((item) => item.title === aTitle),
		`[${TENANT_B.label}] leaked agent conversation from ${TENANT_A.label}`
	).toBe(false);

	await page.unroute('**/agent/conversations');
}
