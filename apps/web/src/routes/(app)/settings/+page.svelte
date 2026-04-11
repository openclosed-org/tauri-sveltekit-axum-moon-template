<script lang="ts">
import { Button, Card, Input } from '$lib/components';
import { signOut } from '$lib/stores/auth.svelte';

let apiKey = $state('');
let baseUrl = $state('https://api.openai.com/v1');
let model = $state('gpt-4o-mini');
let saving = $state(false);
let saved = $state(false);
let testingConnection = $state(false);

type CheckStatus = 'pass' | 'fail';
type ConnectionResult = {
  label: 'API key' | 'Base URL' | 'Model';
  status: CheckStatus;
  nextStep: string;
};

let connectionResults = $state<ConnectionResult[] | null>(null);

function pass(label: ConnectionResult['label'], nextStep: string): ConnectionResult {
  return { label, status: 'pass', nextStep };
}

function fail(label: ConnectionResult['label'], nextStep: string): ConnectionResult {
  return { label, status: 'fail', nextStep };
}

function formatBaseUrlModelsEndpoint(url: string): string {
  return `${url.replace(/\/$/, '')}/models`;
}

function sanitizeError(error: unknown, currentApiKey: string): string {
  const raw = error instanceof Error ? error.message : String(error);
  if (!currentApiKey.trim()) return raw;
  return raw.split(currentApiKey.trim()).join('[redacted-api-key]');
}

async function loadSettings() {
  try {
    if (typeof window !== 'undefined' && (window as { __TAURI__?: unknown }).__TAURI__) {
      const { Store } = await import('@tauri-apps/plugin-store');
      const store = await Store.load('settings.json');

      apiKey = ((await store.get('api_key')) as string | null) ?? '';
      baseUrl = ((await store.get('base_url')) as string | null) ?? 'https://api.openai.com/v1';
      model = ((await store.get('model')) as string | null) ?? 'gpt-4o-mini';
    } else {
      // Web mode: localStorage fallback
      apiKey = localStorage.getItem('settings_api_key') ?? '';
      baseUrl = localStorage.getItem('settings_base_url') ?? 'https://api.openai.com/v1';
      model = localStorage.getItem('settings_model') ?? 'gpt-4o-mini';
    }
  } catch {
    // ignore load failures
  }
}

async function saveSettings() {
  saving = true;
  saved = false;

  try {
    if (typeof window !== 'undefined' && (window as { __TAURI__?: unknown }).__TAURI__) {
      const { Store } = await import('@tauri-apps/plugin-store');
      const store = await Store.load('settings.json');

      await store.set('api_key', apiKey);
      await store.set('base_url', baseUrl);
      await store.set('model', model);
      await store.save();
    } else {
      // Web mode: localStorage fallback
      localStorage.setItem('settings_api_key', apiKey);
      localStorage.setItem('settings_base_url', baseUrl);
      localStorage.setItem('settings_model', model);
    }

    saved = true;
    setTimeout(() => {
      saved = false;
    }, 2000);
  } catch {
    // ignore save failures
  } finally {
    saving = false;
  }
}

async function testConnection() {
  testingConnection = true;
  connectionResults = null;

  const results: ConnectionResult[] = [];
  const trimmedApiKey = apiKey.trim();
  const trimmedBaseUrl = baseUrl.trim();
  const trimmedModel = model.trim();

  if (!trimmedApiKey) {
    results.push(fail('API key', 'Enter your API key before retrying.'));
  } else if (!trimmedApiKey.startsWith('sk-')) {
    results.push(fail('API key', 'API key usually starts with "sk-". Confirm provider format.'));
  } else {
    results.push(pass('API key', 'API key format looks valid.'));
  }

  let modelsPayload: unknown = null;
  let baseUrlIsReachable = false;

  try {
    new URL(trimmedBaseUrl);

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    try {
      const response = await fetch(formatBaseUrlModelsEndpoint(trimmedBaseUrl), {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${trimmedApiKey}`,
          'Content-Type': 'application/json',
        },
        signal: controller.signal,
      });

      if (!response.ok) {
        results.push(
          fail(
            'Base URL',
            `Request failed with status ${response.status}. Verify Base URL and API key.`,
          ),
        );
      } else {
        baseUrlIsReachable = true;
        modelsPayload = await response.json();
        results.push(pass('Base URL', 'Base URL is reachable and responded successfully.'));
      }
    } finally {
      clearTimeout(timeout);
    }
  } catch (error) {
    results.push(
      fail(
        'Base URL',
        `Cannot reach Base URL. ${sanitizeError(error, trimmedApiKey)}. Check URL format and network.`,
      ),
    );
  }

  if (!trimmedModel) {
    results.push(fail('Model', 'Enter a model name before retrying.'));
  } else if (!baseUrlIsReachable) {
    results.push(fail('Model', 'Fix API key/Base URL first, then retry model check.'));
  } else {
    const modelIds = Array.isArray((modelsPayload as { data?: unknown[] } | null)?.data)
      ? ((modelsPayload as { data: Array<{ id?: string }> }).data ?? [])
          .map((item) => item?.id)
          .filter((id): id is string => typeof id === 'string')
      : [];

    if (modelIds.includes(trimmedModel)) {
      results.push(pass('Model', 'Model is available on the target endpoint.'));
    } else {
      results.push(
        fail('Model', `Model "${trimmedModel}" was not found. Pick one from /models and retry.`),
      );
    }
  }

  connectionResults = results;
  testingConnection = false;
}

void loadSettings();
</script>

<div class="p-4 md:p-6 max-w-lg mx-auto space-y-6">
	<div>
		<h1 class="text-2xl font-semibold text-[var(--color-text)]">Settings</h1>
		<p class="text-sm text-[var(--color-text-muted)] mt-1">Configure your Agent Chat API connection</p>
	</div>

	<Card class="p-5 space-y-4">
		<div>
			<label class="text-sm font-medium text-[var(--color-text)]" for="api_key">API Key</label>
			<Input id="api_key" type="password" bind:value={apiKey} placeholder="sk-..." class="mt-1" />
		</div>

		<div>
			<label class="text-sm font-medium text-[var(--color-text)]" for="base_url">Base URL</label>
			<Input
				id="base_url"
				bind:value={baseUrl}
				placeholder="https://api.openai.com/v1"
				class="mt-1"
			/>
		</div>

		<div>
			<label class="text-sm font-medium text-[var(--color-text)]" for="model">Model</label>
			<Input id="model" bind:value={model} placeholder="gpt-4o-mini" class="mt-1" />
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<Button variant="primary" onclick={saveSettings} disabled={saving}>
				{saving ? 'Saving...' : saved ? 'Saved!' : 'Save Settings'}
			</Button>
			<Button variant="secondary" onclick={testConnection} disabled={testingConnection}>
				{testingConnection ? 'Testing...' : 'Test Connection'}
			</Button>
			<Button variant="secondary" onclick={signOut}>Logout</Button>
		</div>

		{#if connectionResults}
			<div class="space-y-2 pt-2">
				{#each connectionResults as result (result.label)}
					<div class="rounded-md border border-[var(--color-border)] p-3" data-testid={`connection-${result.label}`}>
						<div class="flex items-center justify-between gap-2">
							<p class="text-sm font-medium text-[var(--color-text)]">{result.label}</p>
							<span
								class={`text-xs font-medium uppercase ${result.status === 'pass' ? 'text-green-600' : 'text-red-600'}`}
							>
								{result.status}
							</span>
						</div>
						<p class="mt-1 text-xs text-[var(--color-text-muted)]">{result.nextStep}</p>
					</div>
				{/each}
			</div>
		{/if}
	</Card>
</div>
