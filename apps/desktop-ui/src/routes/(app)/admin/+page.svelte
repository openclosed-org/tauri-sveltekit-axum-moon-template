<script lang="ts">
import { Card, Badge } from '$lib/components';

const stats = [
	{ label: 'Total Users', value: '12,345', change: '+12%' },
	{ label: 'Active Sessions', value: '1,234', change: '+5%' },
	{ label: 'Revenue', value: '$45,678', change: '+23%' },
	{ label: 'Growth', value: '8.2%', change: '+1.2%' },
];
</script>

<div class="p-4 md:p-6 space-y-6">
	<!-- Page Header -->
	<div>
		<h1 class="text-2xl font-semibold text-[var(--color-text)]">Admin Dashboard</h1>
		<p class="text-sm text-[var(--color-text-muted)] mt-1">Overview of your application metrics</p>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
		{#each stats as stat}
			<Card class="p-5">
				<div class="flex items-center justify-between">
					<p class="text-sm text-[var(--color-text-muted)]">{stat.label}</p>
					<Badge class="text-xs bg-green-50 text-green-700 dark:bg-green-950 dark:text-green-300">{stat.change}</Badge>
				</div>
				<p class="text-2xl font-semibold text-[var(--color-text)] mt-2">{stat.value}</p>
			</Card>
		{/each}
	</div>

	<!-- Chart Placeholders -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
		<!-- Revenue Chart Placeholder -->
		<Card class="p-5">
			<h3 class="text-sm font-medium text-[var(--color-text)] mb-4">Revenue Over Time</h3>
			<div class="flex items-end gap-1.5 h-32">
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 40%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 55%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 35%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 70%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 50%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 85%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 60%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 75%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 45%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 90%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 65%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 95%"></div>
			</div>
		</Card>

		<!-- User Activity Chart Placeholder -->
		<Card class="p-5">
			<h3 class="text-sm font-medium text-[var(--color-text)] mb-4">User Activity</h3>
			<div class="flex items-end gap-1.5 h-32">
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 60%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 45%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 80%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 30%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 70%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 55%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 90%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 40%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 75%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 50%"></div>
				<div class="flex-1 rounded-t bg-gray-200 dark:bg-gray-700" style="height: 85%"></div>
				<div class="flex-1 rounded-t bg-gray-300 dark:bg-gray-600" style="height: 65%"></div>
			</div>
		</Card>
	</div>
</div>
