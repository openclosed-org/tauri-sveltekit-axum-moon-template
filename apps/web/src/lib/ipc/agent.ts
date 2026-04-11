import type { ChatMessage } from '$lib/generated/api/ChatMessage';
import { safeInvoke, isTauri, safeChannel } from '$lib/ipc/bridge';

const API_BASE = 'http://localhost:3001';

export type Conversation = {
  id: string;
  title: string;
  created_at?: string;
};

function isTauriRuntime() {
  return typeof window !== 'undefined' && !!(window as { __TAURI__?: unknown }).__TAURI__;
}

function shouldPreferIpc() {
  return typeof window !== 'undefined';
}

async function parseJson<T>(response: Response): Promise<T> {
  const data = (await response.json()) as T | { error?: string };

  if (!response.ok) {
    throw new Error(`Agent request failed with status ${response.status}`);
  }

  if (
    typeof data === 'object' &&
    data !== null &&
    'error' in data &&
    typeof data.error === 'string' &&
    data.error.length > 0
  ) {
    throw new Error(data.error);
  }

  return data as T;
}

export async function listConversations(): Promise<Conversation[]> {
  if (shouldPreferIpc()) {
    try {
      return (await safeInvoke('agent_list_conversations')) as Conversation[];
    } catch (error) {
      if (isTauriRuntime()) {
        throw error;
      }
    }
  }

  const response = await fetch(`${API_BASE}/api/agent/conversations`);
  return parseJson<Conversation[]>(response);
}

export async function createConversation(title: string): Promise<Conversation> {
  if (shouldPreferIpc()) {
    try {
      return (await safeInvoke('agent_create_conversation', { title })) as Conversation;
    } catch (error) {
      if (isTauriRuntime()) {
        throw error;
      }
    }
  }

  const response = await fetch(`${API_BASE}/api/agent/conversations`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
  });

  return parseJson<Conversation>(response);
}

export async function getConversationMessages(id: string): Promise<ChatMessage[]> {
  if (shouldPreferIpc()) {
    try {
      return (await safeInvoke('agent_get_messages', { id })) as ChatMessage[];
    } catch (error) {
      if (isTauriRuntime()) {
        throw error;
      }
    }
  }

  const response = await fetch(`${API_BASE}/api/agent/conversations/${id}/messages`);
  return parseJson<ChatMessage[]>(response);
}

export async function* agentChatStream(params: {
  conversationId: string;
  content: string;
  apiKey: string;
  baseUrl: string;
  model: string;
}): AsyncGenerator<string, void, unknown> {
  if (isTauriRuntime()) {
    yield* tauriPath(params);
  } else {
    yield* browserPath(params);
  }
}

/**
 * Tauri path: invoke Rust agent_chat command with Channel for streaming.
 */
async function* tauriPath(params: {
  conversationId: string;
  content: string;
  apiKey: string;
  baseUrl: string;
  model: string;
}): AsyncGenerator<string, void, unknown> {
  const channel = await safeChannel<string>();
  if (!channel) {
    throw new Error('Channel unavailable in web mode');
  }
  let resolveNext: ((value: IteratorResult<string>) => void) | null = null;
  let done = false;

  channel.onmessage = (chunk: string) => {
    if (resolveNext) {
      const resolve = resolveNext;
      resolveNext = null;
      resolve({ value: chunk, done: false });
    }
  };

  const invokePromise = safeInvoke('agent_chat', {
    conversationId: params.conversationId,
    content: params.content,
    apiKey: params.apiKey,
    baseUrl: params.baseUrl,
    model: params.model,
    channel,
  }).catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    if (resolveNext) {
      const resolve = resolveNext;
      resolveNext = null;
      resolve({ value: `Error: ${message}`, done: false });
    }
    done = true;
  });

  try {
    while (!done) {
      const result = await new Promise<IteratorResult<string>>((resolve) => {
        resolveNext = resolve;
      });
      if (result.done) break;
      yield result.value;
    }
  } finally {
    await invokePromise;
  }
}

/**
 * Browser path: HTTP POST + SSE parsing (compatible with existing Axum backend).
 */
async function* browserPath(params: {
  conversationId: string;
  content: string;
  apiKey: string;
  baseUrl: string;
  model: string;
}): AsyncGenerator<string, void, unknown> {
  try {
    const resp = await fetch(`${API_BASE}/api/agent/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        conversation_id: params.conversationId,
        content: params.content,
        api_key: params.apiKey,
        base_url: params.baseUrl,
        model: params.model,
      }),
    });

    if (!resp.body) throw new Error('No response body');

    const reader = resp.body.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const text = decoder.decode(value, { stream: true });
      for (const line of text.split('\n')) {
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6);
        if (data === '[DONE]') continue;
        if (data.startsWith('Error:')) continue;
        yield data;
      }
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    yield `Error: ${message}`;
  }
}
