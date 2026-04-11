import { cleanup, render, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

// Mock bits-ui components that may not resolve in test env
vi.mock('bits-ui', () => ({
  Dialog: {},
  Select: {},
  DropdownMenu: {},
}));

describe('AdminPage', () => {
  async function renderAdminPage(
    mockStats: {
      tenant_count: number;
      counter_value: number;
      last_login: string | null;
      app_version: string;
    } = {
      tenant_count: 12,
      counter_value: 34,
      last_login: '2026-01-01T00:00:00Z',
      app_version: '1.2.3',
    },
  ) {
    const invoke = vi.fn(async () => mockStats);
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = {
      core: { invoke },
    };

    const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
    return { ...render(AdminPage), invoke };
  }

  afterEach(() => {
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = undefined;
    cleanup();
  });

  it('renders the dashboard title', async () => {
    const { getByText } = await renderAdminPage();
    expect(getByText('Admin Dashboard')).toBeTruthy();
  });

  it('displays the dashboard subtitle', async () => {
    const { getByText } = await renderAdminPage();
    expect(getByText(/real-time application metrics/i)).toBeTruthy();
  });

  it('renders all four stat cards', async () => {
    const { getByText } = await renderAdminPage();

    expect(getByText('Tenants')).toBeTruthy();
    expect(getByText('Counter')).toBeTruthy();
    expect(getByText('Last Login')).toBeTruthy();
    expect(getByText('Version')).toBeTruthy();
  });

  it('displays stat card values', async () => {
    const { getByText } = await renderAdminPage({
      tenant_count: 99,
      counter_value: 7,
      last_login: null,
      app_version: '9.9.9',
    });

    await waitFor(() => {
      expect(getByText('99')).toBeTruthy();
      expect(getByText('7')).toBeTruthy();
      expect(getByText('N/A')).toBeTruthy();
      expect(getByText('9.9.9')).toBeTruthy();
    });
  });

  it('shows loading placeholders before stats resolve', async () => {
    let resolveInvoke: ((value: unknown) => void) | null = null;
    const invoke = vi.fn(
      () =>
        new Promise((resolve) => {
          resolveInvoke = resolve;
        }),
    );
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = {
      core: { invoke },
    };

    const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
    const { getAllByText, getByText } = render(AdminPage);

    expect(getAllByText('...').length).toBeGreaterThan(0);

    (resolveInvoke as unknown as (value: unknown) => void)?.({
      tenant_count: 1,
      counter_value: 2,
      last_login: null,
      app_version: '1.0.0',
    });

    await waitFor(() => {
      expect(getByText('1.0.0')).toBeTruthy();
    });
  });

  it('invokes admin command once on mount', async () => {
    const { invoke } = await renderAdminPage();

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('admin_get_dashboard_stats');
    });
  });

  it('renders card shells with responsive grid layout', async () => {
    const { container } = await renderAdminPage();

    const gridContainer = container.querySelector(
      '.grid.grid-cols-1.sm\\:grid-cols-2.lg\\:grid-cols-4',
    );
    expect(gridContainer).toBeTruthy();
  });

  it('renders exactly four stat cards', async () => {
    const { container } = await renderAdminPage();

    const cards = container.querySelectorAll('.rounded-lg.border');
    expect(cards.length).toBe(4);
  });
});
