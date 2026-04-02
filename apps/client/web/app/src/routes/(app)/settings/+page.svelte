<script lang="ts">
import { Card, Switch } from '$lib/components';
import { getTheme, toggleTheme } from '$lib/stores/theme';

const sections = [
	{ label: 'Notifications', description: 'Manage how you receive alerts and updates' },
	{ label: 'Privacy', description: 'Control your data and visibility settings' },
	{ label: 'Language', description: 'Choose your preferred display language' },
];
</script>

<div class="p-4 md:p-6 space-y-6">
	<!-- Page Header -->
	<div>
		<h1 class="text-2xl font-semibold text-[var(--color-text)]">Settings</h1>
		<p class="text-sm text-[var(--color-text-muted)] mt-1">Manage your application preferences</p>
	</div>

	<!-- Appearance -->
	<Card class="p-5 space-y-4">
		<h2 class="text-base font-medium text-[var(--color-text)]">Appearance</h2>
		<div class="flex items-center justify-between">
			<div>
				<p class="text-sm text-[var(--color-text)]">Dark Mode</p>
				<p class="text-xs text-[var(--color-text-muted)]">Toggle between light and dark theme</p>
			</div>
			<Switch checked={getTheme() === 'dark'} onCheckedChange={toggleTheme} />
		</div>
	</Card>

	<!-- Placeholder Sections -->
	{#each sections as section}
		<Card class="p-5">
			<div class="flex items-center justify-between">
				<div>
					<h2 class="text-base font-medium text-[var(--color-text)]">{section.label}</h2>
					<p class="text-xs text-[var(--color-text-muted)] mt-0.5">{section.description}</p>
				</div>
				<span class="text-xs text-[var(--color-text-muted)] bg-[var(--color-bg)] px-2 py-1 rounded">Coming soon</span>
			</div>
		</Card>
	{/each}

	<!-- Account -->
	<Card class="p-5 space-y-3">
		<h2 class="text-base font-medium text-[var(--color-text)]">Account</h2>
		<p class="text-xs text-[var(--color-text-muted)]">Account management will be available after authentication is implemented.</p>
		<div class="flex gap-2">
			<span class="text-xs bg-[var(--color-bg)] text-[var(--color-text-muted)] px-2 py-1 rounded">Profile</span>
			<span class="text-xs bg-[var(--color-bg)] text-[var(--color-text-muted)] px-2 py-1 rounded">Security</span>
			<span class="text-xs bg-[var(--color-bg)] text-[var(--color-text-muted)] px-2 py-1 rounded">Export Data</span>
		</div>
	</Card>
</div>
