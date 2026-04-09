/**
 * Web/Desktop runtime abstraction layer.
 *
 * All Tauri API access MUST go through this module so that the web build
 * never crashes on `window.__TAURI_INTERNALS__` — top-level
 * `@tauri-apps/api/*` imports are forbidden outside this file.
 */

/** Detect whether we are running inside a Tauri WebView. */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as { __TAURI__?: unknown }).__TAURI__;
}

/**
 * Invoke a Tauri command.  In web mode throws a clear error instead of
 * crashing with `__TAURI_INTERNALS__` reference.
 */
export async function safeInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  if (!isTauri()) {
    throw new Error(`Tauri IPC unavailable: cannot invoke "${cmd}" in browser`);
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke(cmd, args);
}

/**
 * Listen to a Tauri event.  Returns a cleanup function.
 * In web mode returns a no-op cleanup — no crash, no warning.
 */
export async function safeListen<T = unknown>(
  event: string,
  callback: (payload: T) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => {};
  }
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<T>(event, (e) => callback(e.payload));
  return () => unlisten();
}

/**
 * Create a Tauri streaming Channel.  Returns `null` in web mode.
 */
export async function safeChannel<T = string>(): Promise<{ onmessage: (payload: T) => void } | null> {
  if (!isTauri()) {
    return null;
  }
  const { Channel } = await import('@tauri-apps/api/core');
  return new Channel<T>();
}
