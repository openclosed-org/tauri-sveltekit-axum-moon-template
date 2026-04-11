import { newId } from '$lib/utils/id';
import type { HandleServerError } from '@sveltejs/kit';

/**
 * Server-side error handler for unexpected errors during SSR.
 *
 * Note: SSR is currently disabled (ssr = false in +layout.ts),
 * but this file is kept for completeness if SSR is enabled later.
 *
 * This handler should NEVER throw. Return a safe user-facing message + tracking ID.
 */
export const handleError: HandleServerError = async ({ error, event, status, message }) => {
  const errorId = newId();

  console.error('[server-error]', {
    errorId,
    status,
    message,
    url: event.url.pathname,
    error: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : undefined,
  });

  return {
    message: 'Internal server error',
    errorId,
  };
};
