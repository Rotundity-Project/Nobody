import { ref } from 'vue';
import type { NotificationItem } from '../components/NotificationCenter.vue';

export const useRuntimeActionFeedback = () => {
  const actionNotification = ref<NotificationItem | null>(null);

  const pushActionNotification = (
    kind: NotificationItem['kind'],
    title: string,
    message?: string,
  ) => {
    actionNotification.value = {
      id: `runtime-action-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      kind,
      title,
      message,
      priority: 'toast',
    };
  };

  const logRuntimeAction = (label: string, message?: string) => {
    console.info(`[runtime-action] ${label}${message ? ` | ${message}` : ''}`);
    pushActionNotification('info', label, message);
  };

  const notifyRuntimeError = (label: string, error: unknown) => {
    const details = error instanceof Error ? error.message : String(error);
    console.error(`${label}：`, error);
    pushActionNotification('error', `${label}失败`, details);
  };

  return {
    actionNotification,
    logRuntimeAction,
    notifyRuntimeError,
  };
};
