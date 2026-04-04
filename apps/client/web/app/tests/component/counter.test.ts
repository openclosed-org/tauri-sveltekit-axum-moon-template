import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest';
import CounterPage from '../../src/routes/(app)/counter/+page.svelte';

describe('CounterPage', () => {
	let counterValue = 0;

	beforeEach(() => {
		counterValue = 0;
		vi.spyOn(globalThis, 'fetch').mockImplementation(async (input, init) => {
			const url = String(input);
			const method = init?.method ?? 'GET';

			if (url.endsWith('/api/counter/value') && method === 'GET') {
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/increment') && method === 'POST') {
				counterValue += 1;
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/decrement') && method === 'POST') {
				counterValue -= 1;
				return new Response(JSON.stringify({ value: counterValue }), { status: 200 });
			}

			if (url.endsWith('/api/counter/reset') && method === 'POST') {
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

	it('increments count when plus button is clicked', async () => {
		const { container } = render(CounterPage);
		// The counter display shows the count value
		const display = container.querySelector('.font-mono');
		expect(display?.textContent?.trim()).toBe('0');

		// Find and click the increment button (Plus icon button)
		const buttons = container.querySelectorAll('button');
		// Buttons: decrement (Minus), increment (Plus), reset
		// Increment is the second button
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
		// Decrement is the first button
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

		// Reset (third button)
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
