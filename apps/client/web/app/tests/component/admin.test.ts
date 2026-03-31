import { render, cleanup } from '@testing-library/svelte';
import { describe, it, expect, vi, afterEach } from 'vitest';

// Mock bits-ui components that may not resolve in test env
vi.mock('bits-ui', () => ({
	Dialog: {},
	Select: {},
	DropdownMenu: {}
}));

describe('AdminPage', () => {
	afterEach(() => {
		cleanup();
	});

	it('renders the dashboard title', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getByText } = render(AdminPage);
		expect(getByText('Admin Dashboard')).toBeTruthy();
	});

	it('displays the dashboard subtitle', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getAllByText } = render(AdminPage);
		// Subtitle may appear twice due to Svelte hydration
		const matches = getAllByText(/overview of your application metrics/i);
		expect(matches.length).toBeGreaterThan(0);
	});

	it('renders all four stat cards', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getByText } = render(AdminPage);

		expect(getByText('Total Users')).toBeTruthy();
		expect(getByText('Active Sessions')).toBeTruthy();
		expect(getByText('Revenue')).toBeTruthy();
		expect(getByText('Growth')).toBeTruthy();
	});

	it('displays stat card values', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getByText } = render(AdminPage);

		expect(getByText('12,345')).toBeTruthy();
		expect(getByText('1,234')).toBeTruthy();
		expect(getByText('$45,678')).toBeTruthy();
		expect(getByText('8.2%')).toBeTruthy();
	});

	it('shows percentage change badges', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getAllByText } = render(AdminPage);

		expect(getAllByText('+12%').length).toBeGreaterThan(0);
		expect(getAllByText('+5%').length).toBeGreaterThan(0);
		expect(getAllByText('+23%').length).toBeGreaterThan(0);
		expect(getAllByText('+1.2%').length).toBeGreaterThan(0);
	});

	it('renders chart placeholder sections', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { getByText } = render(AdminPage);

		expect(getByText('Revenue Over Time')).toBeTruthy();
		expect(getByText('User Activity')).toBeTruthy();
	});

	it('has a responsive grid layout for stat cards', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { container } = render(AdminPage);

		const gridContainer = container.querySelector('.grid');
		expect(gridContainer).toBeTruthy();
	});

	it('renders chart bar elements', async () => {
		const AdminPage = (await import('../../src/routes/(app)/admin/+page.svelte')).default;
		const { container } = render(AdminPage);

		const bars = container.querySelectorAll('[style*="height"]');
		expect(bars.length).toBe(24); // 12 bars per chart, 2 charts
	});
});
