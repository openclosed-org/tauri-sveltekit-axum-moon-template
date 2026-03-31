import type { HandleClientError } from '@sveltejs/kit';
import { newId } from '$lib/utils/id';

/**
 * Client-side error handler for unexpected errors during navigation and rendering.
 *
 * SvelteKit distinguishes:
 * - Expected errors: thrown via error() — go directly to +error.svelte
 * - Unexpected errors: uncaught exceptions — pass through handleError first
 *
 * This handler should NEVER throw. Return a safe user-facing message + tracking ID.
 */
export const handleError: HandleClientError = async ({ error, event, status, message }) => {
	const errorId = newId();

	console.error('[client-error]', {
		errorId,
		status,
		message,
		url: event.url.pathname,
		error: error instanceof Error ? error.message : String(error),
		stack: error instanceof Error ? error.stack : undefined
	});

	return {
		message: 'Something went wrong',
		errorId
	};
};
