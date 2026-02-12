import { invoke } from '@tauri-apps/api/core';

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
    return await Promise.race([invoke<T>(command, args ?? {}), timeout]);
  } finally {
    if (timer) {
      clearTimeout(timer);
    }
  }
}
