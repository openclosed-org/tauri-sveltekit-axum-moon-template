import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import SettingsPage from '../../src/routes/(app)/settings/+page.svelte';

const { storeLoadMock, signOutMock } = vi.hoisted(() => ({
  storeLoadMock: vi.fn(),
  signOutMock: vi.fn(async () => {}),
}));

vi.mock('@tauri-apps/plugin-store', () => ({
  Store: {
    load: storeLoadMock,
  },
}));

vi.mock('$lib/stores/auth.svelte', () => ({
  signOut: signOutMock,
}));

describe('Settings connection diagnostics', () => {
  const storeState = new Map<string, string | null>();
  const storeApi = {
    get: vi.fn(async (key: string) => storeState.get(key) ?? null),
    set: vi.fn(async (key: string, value: string) => {
      storeState.set(key, value);
    }),
    save: vi.fn(async () => {}),
  };

  beforeEach(() => {
    storeState.clear();
    storeState.set('api_key', 'sk-test-123');
    storeState.set('base_url', 'https://api.openai.com/v1');
    storeState.set('model', 'gpt-4o-mini');
    vi.clearAllMocks();
    storeLoadMock.mockResolvedValue(storeApi);
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response(JSON.stringify({ data: [] }), { status: 200 })),
    );

    (window as Window & { __TAURI__?: unknown }).__TAURI__ = { core: { invoke: vi.fn() } };
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = undefined;
  });

  function renderPage() {
    return render(SettingsPage);
  }

  it('shows visible Logout and Test Connection actions', async () => {
    const { getByText } = renderPage();

    expect(getByText('Logout')).toBeTruthy();
    expect(getByText('Test Connection')).toBeTruthy();
  });

  it('renders pass/fail rows for API key, Base URL, and Model with guidance', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          new Response(JSON.stringify({ data: [{ id: 'gpt-4o-mini' }, { id: 'gpt-4.1-mini' }] }), {
            status: 200,
          }),
      ),
    );

    const { getByLabelText, getByText, getByTestId } = renderPage();

    const apiKeyInput = getByLabelText('API Key') as HTMLInputElement;
    const baseUrlInput = getByLabelText('Base URL') as HTMLInputElement;
    const modelInput = getByLabelText('Model') as HTMLInputElement;

    await waitFor(() => {
      expect(apiKeyInput.value).toBe('sk-test-123');
      expect(baseUrlInput.value).toBe('https://api.openai.com/v1');
      expect(modelInput.value).toBe('gpt-4o-mini');
    });

    await fireEvent.input(apiKeyInput, { target: { value: 'bad-key' } });
    await fireEvent.input(baseUrlInput, { target: { value: 'https://example.com/v1' } });
    await fireEvent.input(modelInput, { target: { value: 'missing-model' } });
    await fireEvent.click(getByText('Test Connection'));

    await waitFor(() => {
      expect(getByText('API key')).toBeTruthy();
      expect(getByTestId('connection-Base URL')).toBeTruthy();
      expect(getByTestId('connection-Model')).toBeTruthy();
      expect(getByText(/usually starts with "sk-"/i)).toBeTruthy();
      expect(getByText(/Base URL is reachable/i)).toBeTruthy();
      expect(getByText(/was not found/i)).toBeTruthy();
    });
  });

  it('keeps form values unchanged after failed connection test', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => {
        throw new Error('network down');
      }),
    );

    const { getByLabelText, getByText, getByTestId } = renderPage();

    const apiKeyInput = getByLabelText('API Key') as HTMLInputElement;
    const baseUrlInput = getByLabelText('Base URL') as HTMLInputElement;
    const modelInput = getByLabelText('Model') as HTMLInputElement;

    await waitFor(() => {
      expect(apiKeyInput.value).toBe('sk-test-123');
      expect(baseUrlInput.value).toBe('https://api.openai.com/v1');
      expect(modelInput.value).toBe('gpt-4o-mini');
    });

    await fireEvent.input(apiKeyInput, { target: { value: 'sk-custom-456' } });
    await fireEvent.input(baseUrlInput, { target: { value: 'https://bad-host.invalid/v1' } });
    await fireEvent.input(modelInput, { target: { value: 'gpt-custom' } });

    await fireEvent.click(getByText('Test Connection'));

    await waitFor(() => {
      expect(getByTestId('connection-Base URL')).toBeTruthy();
      expect(getByText(/Cannot reach Base URL/i)).toBeTruthy();
      expect(getByText(/Fix API key\/Base URL first/i)).toBeTruthy();
    });

    expect(apiKeyInput.value).toBe('sk-custom-456');
    expect(baseUrlInput.value).toBe('https://bad-host.invalid/v1');
    expect(modelInput.value).toBe('gpt-custom');
    expect(storeApi.set).not.toHaveBeenCalledWith('api_key', expect.anything());
    expect(storeApi.set).not.toHaveBeenCalledWith('base_url', expect.anything());
    expect(storeApi.set).not.toHaveBeenCalledWith('model', expect.anything());
  });
});
