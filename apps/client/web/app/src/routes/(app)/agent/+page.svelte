<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, Card, Input } from '$lib/components';
	import { Plus, Send } from '@jis3r/icons';
	import type { ChatMessage } from '$lib/generated/api/ChatMessage';
	import type { AgentConfig } from '$lib/generated/api/AgentConfig';

	type Conversation = {
		id: string;
		title: string;
		created_at?: string;
	};

	const API_BASE = 'http://localhost:3001';

	let conversations = $state<Conversation[]>([]);
	let activeConversation = $state<string | null>(null);
	let messages = $state<ChatMessage[]>([]);
	let inputText = $state('');
	let streaming = $state(false);

	function getRuntimeApiBase() {
		if (typeof window === 'undefined') return API_BASE;
		return API_BASE;
	}

	async function loadSettings(): Promise<AgentConfig> {
		const defaults: AgentConfig = {
			api_key: '',
			base_url: 'https://api.openai.com/v1',
			model: 'gpt-4o-mini'
		};

		try {
			if (typeof window !== 'undefined' && (window as { __TAURI__?: unknown }).__TAURI__) {
				const { Store } = await import('@tauri-apps/plugin-store');
				const store = await Store.load('settings.json');
				const apiKey = (await store.get('api_key')) as string | null;
				const baseUrl = (await store.get('base_url')) as string | null;
				const model = (await store.get('model')) as string | null;

				return {
					api_key: apiKey ?? defaults.api_key,
					base_url: baseUrl ?? defaults.base_url,
					model: model ?? defaults.model
				};
			}
		} catch {
			// ignore settings read errors and fallback to defaults
		}

		return defaults;
	}

	async function loadConversations() {
		const resp = await fetch(`${getRuntimeApiBase()}/api/agent/conversations`);
		const data = (await resp.json()) as Conversation[] | { error?: string };
		if (!Array.isArray(data)) return;
		conversations = data;

		if (!activeConversation && conversations.length > 0) {
			await selectConversation(conversations[0].id);
		}
	}

	async function createConversation() {
		const resp = await fetch(`${getRuntimeApiBase()}/api/agent/conversations`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ title: `Chat ${conversations.length + 1}` })
		});

		const conv = (await resp.json()) as Conversation;
		if (!conv?.id) return;

		await loadConversations();
		await selectConversation(conv.id);
	}

	async function selectConversation(id: string) {
		activeConversation = id;
		const resp = await fetch(`${getRuntimeApiBase()}/api/agent/conversations/${id}/messages`);
		const data = (await resp.json()) as ChatMessage[] | { error?: string };
		messages = Array.isArray(data) ? data : [];
	}

	function appendAssistantChunk(chunk: string) {
		const assistantIndex = messages.findIndex((m) => m.id === 'temp-assistant');
		if (assistantIndex === -1) return;

		const existing = messages[assistantIndex];
		const next = {
			...existing,
			content: `${existing.content}${chunk}`
		};

		messages = messages.map((m, i) => (i === assistantIndex ? next : m));
	}

	function isToolChunk(content: string) {
		return content.includes('[tool:');
	}

	async function sendMessage() {
		if (!inputText.trim() || !activeConversation || streaming) return;

		const content = inputText.trim();
		inputText = '';
		streaming = true;

		const tempUser: ChatMessage = {
			id: 'temp-user',
			conversation_id: activeConversation,
			role: 'user',
			content,
			tool_calls: null,
			created_at: new Date().toISOString()
		};
		const tempAssistant: ChatMessage = {
			id: 'temp-assistant',
			conversation_id: activeConversation,
			role: 'assistant',
			content: '',
			tool_calls: null,
			created_at: new Date().toISOString()
		};
		messages = [...messages, tempUser, tempAssistant];

		try {
			const { api_key, base_url, model } = await loadSettings();
			const resp = await fetch(`${getRuntimeApiBase()}/api/agent/chat`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					conversation_id: activeConversation,
					content,
					api_key,
					base_url,
					model
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
					if (data === '[DONE]' || data.startsWith('Error:')) continue;
					appendAssistantChunk(data);
				}
			}
		} catch (error) {
			messages = messages.map((msg) =>
				msg.id === 'temp-assistant'
					? { ...msg, content: `Error: ${error instanceof Error ? error.message : String(error)}` }
					: msg
			);
		} finally {
			streaming = false;
			await loadConversations();
			if (activeConversation) {
				await selectConversation(activeConversation);
			}
		}
	}

	onMount(() => {
		loadConversations();
	});
</script>

<div class="flex h-screen">
	<aside class="w-64 border-r border-[var(--color-border)] p-4 flex flex-col gap-2">
		<Button variant="primary" size="sm" onclick={createConversation}>
			{#snippet icon()}<Plus class="h-4 w-4" />{/snippet}
			New Chat
		</Button>

		<div class="flex-1 overflow-y-auto space-y-1 mt-2">
			{#each conversations as conv (conv.id)}
				<button
					class="w-full text-left px-3 py-2 rounded-md text-sm transition-colors {activeConversation ===
					conv.id
						? 'bg-primary-50 text-primary-700'
						: 'hover:bg-[var(--color-bg)]'}"
					onclick={() => selectConversation(conv.id)}
				>
					{conv.title}
				</button>
			{/each}
		</div>
	</aside>

	<main class="flex-1 flex flex-col">
		{#if !activeConversation}
			<div class="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">
				Select or create a conversation to start chatting
			</div>
		{:else}
			<div class="flex-1 overflow-y-auto p-4 space-y-4">
				{#each messages as msg, i (`${msg.id}-${i}`)}
					<div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'}">
						<Card
							class="max-w-[70%] p-3 {msg.role === 'user'
								? 'bg-primary-50'
								: ''} {isToolChunk(msg.content) ? 'border border-amber-300 bg-amber-50' : ''}"
						>
							{#if isToolChunk(msg.content)}
								<p class="mb-1 text-xs font-medium text-amber-700">Tool Result</p>
							{/if}
							<p class="text-sm whitespace-pre-wrap">{msg.content || (streaming ? '...' : '')}</p>
						</Card>
					</div>
				{/each}
			</div>

			<div class="border-t border-[var(--color-border)] p-4 flex gap-2">
				<Input bind:value={inputText} placeholder="Type a message..." class="flex-1" />
				<Button variant="primary" onclick={sendMessage} disabled={streaming || !inputText.trim()}>
					{#snippet icon()}<Send class="h-4 w-4" />{/snippet}
					Send
				</Button>
			</div>
		{/if}
	</main>
</div>
