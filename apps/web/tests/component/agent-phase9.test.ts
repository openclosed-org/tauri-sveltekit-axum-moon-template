import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AgentPage from '../../src/routes/(app)/agent/+page.svelte';

const {
  listConversationsMock,
  createConversationMock,
  getConversationMessagesMock,
  agentChatStreamMock,
  storeGetMock,
  storeLoadMock,
} = vi.hoisted(() => ({
  listConversationsMock: vi.fn(),
  createConversationMock: vi.fn(),
  getConversationMessagesMock: vi.fn(),
  agentChatStreamMock: vi.fn(),
  storeGetMock: vi.fn(),
  storeLoadMock: vi.fn(),
}));

vi.mock('$lib/ipc/agent', () => ({
  listConversations: listConversationsMock,
  createConversation: createConversationMock,
  getConversationMessages: getConversationMessagesMock,
  agentChatStream: agentChatStreamMock,
}));

vi.mock('@tauri-apps/plugin-store', () => ({
  Store: {
    load: storeLoadMock,
  },
}));

describe('AgentPage Phase 9 regressions', () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = { core: {} };

    listConversationsMock.mockResolvedValue([
      { id: 'conv-old', title: 'Old Chat' },
      { id: 'conv-new', title: 'Chat 2' },
    ]);
    createConversationMock.mockResolvedValue({ id: 'conv-new', title: 'Chat 2' });
    getConversationMessagesMock.mockImplementation(async (id: string) =>
      id === 'conv-old'
        ? [
            {
              id: 'msg-old',
              conversation_id: 'conv-old',
              role: 'assistant',
              content: 'old thread message',
              tool_calls: null,
              created_at: new Date().toISOString(),
            },
          ]
        : [],
    );
    agentChatStreamMock.mockImplementation(async function* () {
      yield 'ok';
    });

    storeGetMock.mockImplementation(async (key: string) => {
      if (key === 'api_key') return 'sk-keep';
      if (key === 'base_url') return 'https://api.example.com/v1';
      if (key === 'model') return 'gpt-4.1-mini';
      return null;
    });
    storeLoadMock.mockResolvedValue({ get: storeGetMock });
  });

  afterEach(() => {
    (window as Window & { __TAURI__?: unknown }).__TAURI__ = undefined;
    cleanup();
  });

  it('clicking New Chat selects the new thread and clears old messages', async () => {
    const { getByRole, queryByText, getByText } = render(AgentPage);

    await waitFor(() => {
      expect(getByText('Old Chat')).toBeTruthy();
    });

    await fireEvent.click(getByText('Old Chat'));

    await waitFor(() => {
      expect(queryByText('old thread message')).toBeTruthy();
    });

    await fireEvent.click(getByRole('button', { name: /new chat/i }));

    await waitFor(() => {
      expect(queryByText('old thread message')).toBeNull();
    });
  });

  it('New Chat does not clobber persisted api_key/base_url/model settings', async () => {
    const { getByRole, getByPlaceholderText } = render(AgentPage);

    await fireEvent.click(getByRole('button', { name: /new chat/i }));

    await fireEvent.input(getByPlaceholderText('Type a message...'), {
      target: { value: 'retained settings check' },
    });
    await fireEvent.click(getByRole('button', { name: /send/i }));

    await waitFor(() => {
      expect(agentChatStreamMock).toHaveBeenCalledTimes(1);
    });

    for (const [params] of agentChatStreamMock.mock.calls) {
      expect(params.apiKey).toBe('sk-keep');
      expect(params.baseUrl).toBe('https://api.example.com/v1');
      expect(params.model).toBe('gpt-4.1-mini');
    }
  });

  it('settings read failure shows actionable guidance and falls back to defaults', async () => {
    storeLoadMock.mockRejectedValueOnce(new Error('cannot open settings.json'));

    const { getByRole, getByPlaceholderText, getByText } = render(AgentPage);

    await fireEvent.click(getByRole('button', { name: /new chat/i }));
    await fireEvent.input(getByPlaceholderText('Type a message...'), {
      target: { value: 'test fallback' },
    });
    await fireEvent.click(getByRole('button', { name: /send/i }));

    await waitFor(() => {
      expect(
        getByText(
          /Could not read settings\.json\. Open Settings and re-save API key, Base URL, and Model\./i,
        ),
      ).toBeTruthy();
    });

    await waitFor(() => {
      expect(agentChatStreamMock).toHaveBeenCalled();
    });

    const [params] = agentChatStreamMock.mock.calls.at(-1) ?? [];
    expect(params.apiKey).toBe('');
    expect(params.baseUrl).toBe('https://api.openai.com/v1');
    expect(params.model).toBe('gpt-4o-mini');
  });
});
