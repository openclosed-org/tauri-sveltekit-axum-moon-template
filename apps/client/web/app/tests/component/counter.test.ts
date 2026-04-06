import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest';
import CounterPage from '../../src/routes/(app)/counter/+page.svelte';

describe('CounterPage', () => {
	let counterValue = 0;
	let loadError = false;
	let failCommands = new Set<string>();

	beforeEach(() => {
		counterValue = 0;
		loadError = false;
		failCommands = new Set<string>();

		vi.spyOn(globalThis, 'fetch').mockImplementation(async (input, init) => {
			const url = String(input);
			const method = init?.method ?? 'GET';

			if (url.endsWith('/api/counter/value') && method === 'GET') {
				if (loadError) {
					throw new Error('load failed');
				}
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/increment') && method === 'POST') {
				if (failCommands.has('counter_increment')) {
					throw new Error('increment failed');
				}
				counterValue += 1;
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/decrement') && method === 'POST') {
				if (failCommands.has('counter_decrement')) {
					throw new Error('decrement failed');
				}
				counterValue -= 1;
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/reset') && method === 'POST') {
				if (failCommands.has('counter_reset')) {
					throw new Error('reset failed');
				}
				counterValue = 0;
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			return new Response(JSON.stringify({ value: counterValue }), { status: 404 });
		});
	});

	afterEach(() => {
		vi.restoreAllMocks();
		cleanup();
	});

	it('renders counter display with initial value 0', () => {
		const { getByText } = render(CounterPage);
		expect(getByText('0')).toBeTruthy();
	});

	it('shows error banner and keeps count at 0 when load fails', async () => {
		loadError = true;
		const { container, getByText } = render(CounterPage);

		const display = container.querySelector('.font-mono');

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('0');
			expect(getByText('Failed to load persisted counter value')).toBeTruthy();
		});
	});

	it('updates count only from command return values on increment/decrement/reset success', async () => {
		counterValue = 5;
		const { container } = render(CounterPage);
		const display = container.querySelector('.font-mono');
		const buttons = container.querySelectorAll('button');

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('5');
		});

		counterValue = 41;
		await fireEvent.click(buttons[1]);
		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('42');
		});

		counterValue = 8;
		await fireEvent.click(buttons[0]);
		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('7');
		});

		counterValue = 0;
		await fireEvent.click(buttons[2]);
		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('0');
		});
	});

	it('keeps previous successful count and shows visible error when mutation fails', async () => {
		counterValue = 9;
		const { container, getByText } = render(CounterPage);
		const display = container.querySelector('.font-mono');
		const buttons = container.querySelectorAll('button');

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('9');
		});

		counterValue = 10;
		await fireEvent.click(buttons[1]);
		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('11');
		});

		failCommands.add('counter_increment');
		counterValue = 500;
		await fireEvent.click(buttons[1]);

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('11');
			expect(getByText('Failed to increment counter')).toBeTruthy();
		});
	});

	it('increments count when plus button is clicked', async () => {
		const { container } = render(CounterPage);
		const display = container.querySelector('.font-mono');
		expect(display?.textContent?.trim()).toBe('0');

		const buttons = container.querySelectorAll('button');
		const incrementBtn = buttons[1];
		await fireEvent.click(incrementBtn);

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('1');
		});
	});

	it('decrements count when minus button is clicked', async () => {
		const { container } = render(CounterPage);
		const display = container.querySelector('.font-mono');

		const buttons = container.querySelectorAll('button');
		const decrementBtn = buttons[0];
		await fireEvent.click(decrementBtn);

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('-1');
		});
	});

	it('resets count to 0 when reset button is clicked', async () => {
		const { container } = render(CounterPage);
		const display = container.querySelector('.font-mono');
		const buttons = container.querySelectorAll('button');

		// Increment twice
		await fireEvent.click(buttons[1]);
		await fireEvent.click(buttons[1]);

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('2');
		});

		const resetBtn = buttons[2];
		await fireEvent.click(resetBtn);

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('0');
		});
	});

	it('handles multiple increment and decrement operations', async () => {
		const { container } = render(CounterPage);
		const display = container.querySelector('.font-mono');
		const buttons = container.querySelectorAll('button');

		await fireEvent.click(buttons[1]); // +1
		await fireEvent.click(buttons[1]); // +2
		await fireEvent.click(buttons[1]); // +3
		await fireEvent.click(buttons[0]); // -1 => 2

		await waitFor(() => {
			expect(display?.textContent?.trim()).toBe('2');
		});
	});
});
