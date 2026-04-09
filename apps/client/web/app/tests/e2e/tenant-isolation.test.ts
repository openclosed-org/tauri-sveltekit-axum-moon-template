import { type APIRequestContext, expect, test } from '@playwright/test';
import { buildTenantAuthHeaders } from '../fixtures/auth';
import { ensureApiReady } from '../fixtures/runtime';
import { TENANT_A, TENANT_B, resetTenantPair } from '../fixtures/tenant';

const TENANT_INIT_URL = 'http://127.0.0.1:3001/api/tenant/init';
const COUNTER_VALUE_URL = 'http://127.0.0.1:3001/api/counter/value';
const COUNTER_INCREMENT_URL = 'http://127.0.0.1:3001/api/counter/increment';
const AGENT_CONVERSATIONS_URL = 'http://127.0.0.1:3001/api/agent/conversations';

test.describe('Tenant Isolation (E2E)', () => {
  test.describe.configure({ mode: 'serial' });
  const COUNTER_START = 0;
  const TENANT_A_MUTATIONS = 2;

  test.beforeEach(async ({ page }) => {
    await ensureApiReady();
    await resetTenantPair(page);
  });

  test('tenant-1 write does not alter tenant-2 value (run-1)', async ({ request }) => {
    await assertTenantIsolationFlow({
      request,
      runLabel: 'run-1',
      seed: COUNTER_START,
      mutations: TENANT_A_MUTATIONS,
    });
  });

  test('tenant-1 write does not alter tenant-2 value (run-2, same seed)', async ({ request }) => {
    await assertTenantIsolationFlow({
      request,
      runLabel: 'run-2',
      seed: COUNTER_START,
      mutations: TENANT_A_MUTATIONS,
    });
  });

  test('settings and agent conversation are isolated by tenant+user', async ({ page, request }) => {
    await assertSettingsIsolation(request);
    await assertAgentIsolation(page);
  });

  test('theme preference is isolated per user', async ({ browser }) => {
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    await pageA.goto('/');
    await pageA.evaluate((value) => localStorage.setItem('theme-preference', value), 'dark');
    const themeA = await pageA.evaluate(() => localStorage.getItem('theme-preference'));
    expect(themeA, `[${TENANT_A.label}] expected stored theme to be dark`).toBe('dark');

    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();
    await pageB.goto('/');
    const themeBBefore = await pageB.evaluate(() => localStorage.getItem('theme-preference'));
    expect(
      themeBBefore,
      `[${TENANT_B.label}] expected no inherited theme from ${TENANT_A.label}`,
    ).toBeNull();
    await pageB.evaluate((value) => localStorage.setItem('theme-preference', value), 'light');
    const themeBAfter = await pageB.evaluate(() => localStorage.getItem('theme-preference'));
    expect(themeBAfter, `[${TENANT_B.label}] expected stored theme to be light`).toBe('light');

    const themeAAfter = await pageA.evaluate(() => localStorage.getItem('theme-preference'));
    expect(themeAAfter, `[${TENANT_A.label}] leaked theme after ${TENANT_B.label} mutation`).toBe(
      'dark',
    );

    await contextA.close();
    await contextB.close();
  });
});

type IsolationFlowArgs = {
  request: APIRequestContext;
  runLabel: string;
  seed: number;
  mutations: number;
};

async function readTenantCounter(request: APIRequestContext, userSub: string): Promise<number> {
  const response = await request.get(COUNTER_VALUE_URL, {
    headers: buildTenantAuthHeaders(userSub),
  });
  const body = (await response.json().catch(() => ({}))) as { value?: number };
  if (response.status() !== 200 || typeof body.value !== 'number') {
    throw new Error(
      `read counter failed for ${userSub}: status=${response.status()}, body=${JSON.stringify(body)}`,
    );
  }
  return body.value;
}

async function incrementTenantCounter(
  request: APIRequestContext,
  userSub: string,
  times: number,
): Promise<void> {
  for (let i = 0; i < times; i += 1) {
    const response = await request.post(COUNTER_INCREMENT_URL, {
      headers: buildTenantAuthHeaders(userSub),
    });
    const body = (await response.json().catch(() => ({}))) as { value?: number };
    if (response.status() !== 200 || typeof body.value !== 'number') {
      throw new Error(
        `increment failed for ${userSub} at step ${i + 1}: status=${response.status()}, body=${JSON.stringify(body)}`,
      );
    }
  }
}

async function assertTenantIsolationFlow({
  request,
  runLabel,
  seed,
  mutations,
}: IsolationFlowArgs): Promise<void> {
  const tenantAStart = await readTenantCounter(request, TENANT_A.userSub);
  expect(
    tenantAStart,
    `[${runLabel}] expected tenant-1 baseline ${seed}, got ${tenantAStart}`,
  ).toBe(seed);

  await incrementTenantCounter(request, TENANT_A.userSub, mutations);
  const tenantAAfter = await readTenantCounter(request, TENANT_A.userSub);
  const expectedTenantA = seed + mutations;
  expect(
    tenantAAfter,
    `[${runLabel}] tenant-1 expected ${expectedTenantA} after ${mutations} writes, got ${tenantAAfter}`,
  ).toBe(expectedTenantA);

  const tenantBAfter = await readTenantCounter(request, TENANT_B.userSub);
  expect(
    tenantBAfter,
    `[${runLabel}] tenant-2 leaked value after tenant-1 writes: expected ${seed}, got ${tenantBAfter}`,
  ).toBe(seed);
}

async function initTenantAndReadConfig(
  request: APIRequestContext,
  tenant: { label: string; userSub: string; userName: string },
): Promise<{ tenantId: string; role: string }> {
  const response = await request.post(TENANT_INIT_URL, {
    headers: buildTenantAuthHeaders(tenant.userSub),
    data: { user_sub: tenant.userSub, user_name: tenant.userName },
  });
  const body = (await response.json().catch(() => ({}))) as { tenant_id?: string; role?: string };
  if (
    response.status() !== 200 ||
    typeof body.tenant_id !== 'string' ||
    typeof body.role !== 'string'
  ) {
    throw new Error(
      `[${tenant.label}] settings init failed: status=${response.status()}, body=${JSON.stringify(body)}`,
    );
  }
  return { tenantId: body.tenant_id, role: body.role };
}

async function assertSettingsIsolation(request: APIRequestContext): Promise<void> {
  const aFirst = await initTenantAndReadConfig(request, TENANT_A);
  const bFirst = await initTenantAndReadConfig(request, TENANT_B);

  expect(aFirst.role, `[${TENANT_A.label}] expected role to exist`).toBeTruthy();
  expect(bFirst.role, `[${TENANT_B.label}] expected role to exist`).toBeTruthy();
  expect(aFirst.tenantId, `[${TENANT_A.label}] tenant id leaked from ${TENANT_B.label}`).not.toBe(
    bFirst.tenantId,
  );

  const aSecond = await initTenantAndReadConfig(request, TENANT_A);
  const bSecond = await initTenantAndReadConfig(request, TENANT_B);
  expect(aSecond.tenantId, `[${TENANT_A.label}] tenant config not stable across retries`).toBe(
    aFirst.tenantId,
  );
  expect(bSecond.tenantId, `[${TENANT_B.label}] tenant config not stable across retries`).toBe(
    bFirst.tenantId,
  );
}

async function assertAgentIsolation(page: import('@playwright/test').Page): Promise<void> {
  const aTitle = `tenant-a-conv-${Date.now()}`;
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
      const created = {
        id: `${userKey}-${current.length + 1}`,
        title: payload.title ?? 'Untitled',
      };
      userConversations.set(userKey, [...current, created]);
      await route.fulfill({ status: 200, json: created });
      return;
    }

    await route.fulfill({ status: 405, json: { error: 'Method not allowed' } });
  });

  await page.goto('/');

  const createA = await page.evaluate(
    async ({ url, headers, title }) => {
      const response = await fetch(url, {
        method: 'POST',
        headers,
        body: JSON.stringify({ title }),
      });
      return { status: response.status, body: await response.json().catch(() => ({})) };
    },
    {
      url: AGENT_CONVERSATIONS_URL,
      headers: buildTenantAuthHeaders(TENANT_A.userSub),
      title: aTitle,
    },
  );

  if (createA.status !== 200) {
    throw new Error(
      `[${TENANT_A.label}] create agent conversation failed: status=${createA.status}`,
    );
  }

  const listA = await page.evaluate(
    async ({ url, headers }) => {
      const response = await fetch(url, { method: 'GET', headers });
      return { status: response.status, body: await response.json().catch(() => []) };
    },
    { url: AGENT_CONVERSATIONS_URL, headers: buildTenantAuthHeaders(TENANT_A.userSub) },
  );

  const listABody = (listA.body ?? []) as Array<{ id?: string; title?: string }>;
  expect(listA.status, `[${TENANT_A.label}] list conversations failed`).toBe(200);
  expect(
    listABody.some((item) => item.title === aTitle),
    `[${TENANT_A.label}] cannot observe its own conversation`,
  ).toBe(true);

  const listB = await page.evaluate(
    async ({ url, headers }) => {
      const response = await fetch(url, { method: 'GET', headers });
      return { status: response.status, body: await response.json().catch(() => []) };
    },
    { url: AGENT_CONVERSATIONS_URL, headers: buildTenantAuthHeaders(TENANT_B.userSub) },
  );

  const listBBody = (listB.body ?? []) as Array<{ id?: string; title?: string }>;
  expect(listB.status, `[${TENANT_B.label}] list conversations failed`).toBe(200);
  expect(
    listBBody.some((item) => item.title === aTitle),
    `[${TENANT_B.label}] leaked agent conversation from ${TENANT_A.label}`,
  ).toBe(false);

  await page.unroute('**/agent/conversations');
}
