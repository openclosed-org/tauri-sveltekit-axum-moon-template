/**
 * Dual-path agent client — Tauri IPC (Channel streaming) or HTTP SSE fallback.
 *
 * Runtime detection: `(window as { __TAURI__?: unknown }).__TAURI__`
 * - Tauri path: invoke('agent_chat', { channel }) via Tauri 2 Channel API
 * - Browser path: fetch() + SSE parsing (compatible with existing Axum backend)
 */

import { invoke, Channel } from '@tauri-apps/api/core';

const API_BASE = 'http://localhost:3001';

export async function* agentChatStream(params: {
	conversationId: string;
	content: string;
	apiKey: string;
	baseUrl: string;
	model: string;
}): AsyncGenerator<string, void, unknown> {
	const isTauri = typeof window !== 'undefined' && !!(window as { __TAURI__?: unknown }).__TAURI__;

	if (isTauri) {
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
	const channel = new Channel<string>();
	let resolveNext: ((value: IteratorResult<string>) => void) | null = null;
	let done = false;

	channel.onmessage = (chunk: string) => {
		if (resolveNext) {
			const resolve = resolveNext;
			resolveNext = null;
			resolve({ value: chunk, done: false });
		}
	};

	const invokePromise = invoke('agent_chat', {
		conversationId: params.conversationId,
		content: params.content,
		apiKey: params.apiKey,
		baseUrl: params.baseUrl,
		model: params.model,
		channel
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
				model: params.model
			})
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
