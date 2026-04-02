<script lang="ts">
import { onMount } from 'svelte';
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { Switch } from '$lib/components';
import { LayoutDashboard, Plus, Settings, PanelLeftClose, PanelLeftOpen } from '@jis3r/icons';
import { toggleTheme, getTheme } from '$lib/stores/theme';
import { auth, checkSession } from '$lib/stores/auth.svelte';
import type { Snippet } from 'svelte';

interface Props {
	children: Snippet;
}

const { children }: Props = $props();

let sidebarExpanded = $state(true);
let isDark = $state(getTheme() === 'dark');

// Auth guard: check session on mount, redirect if not authenticated
onMount(async () => {
	const hasSession = await checkSession();
	if (!hasSession) {
		goto('/login');
	}
});

	// Reactive guard: redirect if auth state changes (e.g., token expires)
	$effect(() => {
		if (!auth.isAuthenticated) {
			goto('/login');
		}
	});

const navItems = [
	{ href: '/counter', label: 'Counter', icon: Plus },
	{ href: '/admin', label: 'Admin', icon: LayoutDashboard },
	{ href: '/settings', label: 'Settings', icon: Settings },
];

function handleThemeToggle(checked: boolean) {
	isDark = checked;
	toggleTheme();
}
</script>

<div class="flex min-h-screen bg-[var(--color-bg)]">
	<!-- Desktop Sidebar (hidden on mobile) -->
	<aside
		class="hidden md:flex md:flex-col border-r border-[var(--color-border)] bg-[var(--color-bg-elevated)] transition-[width] duration-150 ease-in-out"
		style:width={sidebarExpanded ? '240px' : '64px'}
	>
		<!-- Logo + App Name -->
		<div class="flex h-14 items-center gap-2 border-b border-[var(--color-border)] px-4">
			<div class="h-8 w-8 rounded-lg bg-primary-600 flex items-center justify-center text-white font-bold text-sm shrink-0">
				T
			</div>
			{#if sidebarExpanded}
				<span class="text-sm font-semibold text-[var(--color-text)] truncate">Tauri App</span>
			{/if}
		</div>

		<!-- Nav Links -->
		<nav class="flex-1 p-2 space-y-1">
			{#each navItems as item}
				{@const active = page.url.pathname === item.href || page.url.pathname.startsWith(item.href + '/')}
				<a
					href={item.href}
					class="flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors {active
						? 'bg-primary-50 text-primary-700 dark:bg-primary-950 dark:text-primary-300'
						: 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg)] hover:text-[var(--color-text)]'}"
				>
					<item.icon class="h-5 w-5 shrink-0" />
					{#if sidebarExpanded}
						<span class="truncate">{item.label}</span>
					{/if}
				</a>
			{/each}
		</nav>

		<!-- Bottom: Settings + Theme Toggle -->
		<div class="border-t border-[var(--color-border)] p-3 space-y-2">
			<div class="flex items-center justify-between">
				{#if sidebarExpanded}
					<span class="text-xs text-[var(--color-text-muted)]">Dark mode</span>
				{/if}
				<Switch checked={isDark} onCheckedChange={handleThemeToggle} />
			</div>
			<button
				onclick={() => (sidebarExpanded = !sidebarExpanded)}
				class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs text-[var(--color-text-muted)] hover:bg-[var(--color-bg)] hover:text-[var(--color-text)] transition-colors"
			>
				{#if sidebarExpanded}
					<PanelLeftClose class="h-4 w-4" />
					<span>Collapse</span>
				{:else}
					<PanelLeftOpen class="h-4 w-4 mx-auto" />
				{/if}
			</button>
		</div>
	</aside>

	<!-- Main Content -->
	<main class="flex-1 overflow-auto pb-14 md:pb-0">
		{@render children()}
	</main>

	<!-- Mobile Bottom Tab Bar (hidden on desktop) -->
	<nav class="fixed bottom-0 left-0 right-0 z-50 flex h-14 items-center justify-around border-t border-[var(--color-border)] bg-[var(--color-bg-elevated)] md:hidden">
		{#each navItems as item}
			{@const active = page.url.pathname === item.href || page.url.pathname.startsWith(item.href + '/')}
			<a
				href={item.href}
				class="flex flex-col items-center gap-0.5 px-3 py-1 transition-colors {active
					? 'text-primary-600'
					: 'text-[var(--color-text-muted)]'}"
			>
				<item.icon class="h-5 w-5" />
				<span class="text-[10px]">{item.label}</span>
			</a>
		{/each}
	</nav>
</div>
