import type { Page } from '@playwright/test';
import { makeTenantToken } from './auth';

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
  mockCode: 'tenant_a_user',
};

export const TENANT_B: TenantIdentity = {
  label: 'tenant-B',
  userSub: 'tenant_b_user',
  userName: 'Tenant B User',
  mockCode: 'tenant_b_user',
};

export const TENANT_LABELS = [TENANT_A.label, TENANT_B.label] as const;

const TENANTS = [TENANT_A, TENANT_B] as const;
const TENANT_INIT_URL = 'http://127.0.0.1:3001/api/tenant/init';
const COUNTER_RESET_URL = 'http://127.0.0.1:3001/api/counter/reset';
const RETRY_LIMIT = 3;
const RETRY_DELAY_MS = 800;

type InitTenantResponse = {
  tenant_id?: string;
  role?: string;
  created?: boolean;
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
    `${operationName} failed after ${RETRY_LIMIT} attempts: ${lastError instanceof Error ? lastError.message : String(lastError)}`,
  );
}

async function callTenantInit(page: Page, tenant: TenantIdentity): Promise<void> {
  const response = await page.request.post(TENANT_INIT_URL, {
    headers: {
      Authorization: `Bearer ${makeTenantToken(tenant.userSub)}`,
      'content-type': 'application/json',
    },
    data: {
      user_sub: tenant.userSub,
      user_name: tenant.userName,
    },
  });

  const body = (await response.json().catch(() => ({}))) as InitTenantResponse;
  if (
    response.status() !== 200 ||
    typeof body.tenant_id !== 'string' ||
    body.tenant_id.length === 0
  ) {
    throw new Error(
      `[${tenant.label}] tenant init failed: status=${response.status()}, body=${JSON.stringify(body)}`,
    );
  }
}

async function resetCounterViaApi(page: Page, tenant: TenantIdentity): Promise<void> {
  const response = await page.request.post(COUNTER_RESET_URL, {
    headers: {
      Authorization: `Bearer ${makeTenantToken(tenant.userSub)}`,
      'content-type': 'application/json',
    },
  });

  const body = (await response.json().catch(() => ({}))) as { value?: number };
  if (response.status() !== 200 || body.value !== 0) {
    throw new Error(
      `[${tenant.label}] counter reset failed: status=${response.status()}, body=${JSON.stringify(body)}`,
    );
  }
}

export async function initTenantPair(page: Page): Promise<void> {
  for (const tenant of TENANTS) {
    await withRetry(`[${tenant.label}] tenant init`, () => callTenantInit(page, tenant));
  }
}

export async function resetTenantPair(page: Page): Promise<void> {
  await initTenantPair(page);

  for (const tenant of TENANTS) {
    try {
      await withRetry(`[${tenant.label}] counter reset`, () => resetCounterViaApi(page, tenant));
    } catch (error) {
      throw new Error(
        `[${tenant.label}] reset tenant counter failed: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }
}

export async function resetTenantPairCounter(page: Page): Promise<void> {
  await resetTenantPair(page);
}
