<script lang="ts">
import { Card } from '$lib/components';
import type { AdminDashboardStats } from '$lib/generated/api/AdminDashboardStats';
import { onMount } from 'svelte';

let stats = $state<AdminDashboardStats>({
  tenant_count: 0,
  counter_value: 0,
  last_login: null,
  app_version: '0.0.0',
});
let loading = $state(true);

interface TauriWindow {
  __TAURI__?: { core: { invoke(cmd: string): Promise<unknown> } };
}
const tauriApi = typeof window !== 'undefined' ? (window as TauriWindow).__TAURI__ : undefined;
const isTauri = !!tauriApi;

async function fetchStats() {
  loading = true;
  try {
    if (tauriApi) {
      stats = (await tauriApi.core.invoke('admin_get_dashboard_stats')) as AdminDashboardStats;
    } else {
      const resp = await fetch('http://localhost:3001/api/admin/stats');
      stats = await resp.json();
    }
  } catch (e) {
    console.error('Failed to load stats:', e);
  }
  loading = false;
}

onMount(() => {
  fetchStats();
});

const statCards = $derived([
  { label: 'Tenants', value: String(stats.tenant_count), icon: '🏢' },
  { label: 'Counter', value: String(stats.counter_value), icon: '🔢' },
  {
    label: 'Last Login',
    value: stats.last_login ? new Date(stats.last_login).toLocaleDateString() : 'N/A',
    icon: '👤',
  },
  { label: 'Version', value: stats.app_version, icon: '📦' },
]);
</script>

<div class="p-4 md:p-6 space-y-6">
	<div>
		<h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-50">Admin Dashboard</h1>
		<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Real-time application metrics</p>
	</div>
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
		{#each statCards as stat}
			<Card class="p-5">
				<div class="flex items-center justify-between">
				<p class="text-sm text-gray-500 dark:text-gray-400">{stat.label}</p>
					<span class="text-lg">{stat.icon}</span>
				</div>
			<p class="text-2xl font-semibold text-gray-900 dark:text-gray-50 mt-2">{loading ? '...' : stat.value}</p>
			</Card>
		{/each}
	</div>
</div>
