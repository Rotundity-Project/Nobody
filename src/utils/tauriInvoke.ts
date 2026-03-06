import { isTauriRuntime } from '../platform/runtimeEnv';
import { invokeWebRuntime } from '../platform/webRuntime';

export async function invokeRuntime<T>(
  command: string,
  args: Record<string, unknown> | undefined,
): Promise<T> {
  if (isTauriRuntime()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args ?? {});
  }
  return invokeWebRuntime<T>(command, args ?? {});
}

export async function invokeWithTimeout<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  timeoutMs: number,
  timeoutMessage: string,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const timeout = new Promise<never>((_, reject) => {
      timer = setTimeout(() => reject(new Error(timeoutMessage)), timeoutMs);
    });
    return await Promise.race([invokeRuntime<T>(command, args), timeout]);
  } finally {
    if (timer) {
      clearTimeout(timer);
    }
  }
}
