import { vi, type Mock } from 'vitest';

type Handler = (args?: unknown) => unknown | Promise<unknown>;

let handlers: Record<string, Handler> = {};
let runtimeInstalled = false;

// Mocked replacement for `@tauri-apps/api/core`'s `invoke`. Tests register
// per-command handlers with `setInvokeHandlers`; unmocked commands throw so
// missing fixtures are visible rather than silently returning undefined.
export const invokeMock: Mock = vi.fn(async (command: string, args?: unknown) => {
  const handler = handlers[command];
  if (!handler) {
    throw new Error(`Unmocked Tauri invoke: ${command}`);
  }
  return handler(args);
});

// The frontend uses `'__TAURI_INTERNALS__' in window` to decide whether to call
// invoke or fall back to preview data. jsdom has no such property, so tests
// that exercise the invoke path must call this once during setup.
export function installTauriRuntime(): void {
  if (typeof window === 'undefined') return;
  if ('__TAURI_INTERNALS__' in window) return;
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    value: {},
    configurable: true,
    writable: true,
  });
  runtimeInstalled = true;
}

export function uninstallTauriRuntime(): void {
  if (!runtimeInstalled || typeof window === 'undefined') return;
  delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  runtimeInstalled = false;
}

export type InvokeFixture =
  | unknown
  | ((args?: unknown) => unknown | Promise<unknown>);

// Register handlers keyed by command name. A non-function value becomes a
// constant responder; a function receives the invoke args. Returns a reset
// function to call from `afterEach` so handlers do not leak across tests.
export function setInvokeHandlers(
  map: Record<string, InvokeFixture>,
): () => void {
  handlers = {};
  for (const [command, fixture] of Object.entries(map)) {
    handlers[command] =
      typeof fixture === 'function'
        ? (fixture as Handler)
        : () => fixture;
  }
  return resetInvokeMock;
}

export function resetInvokeMock(): void {
  handlers = {};
  invokeMock.mockClear();
}
