export function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  return typeof (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined';
}

