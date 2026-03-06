import { ref } from 'vue';
import type { NotificationItem } from '../components/NotificationCenter.vue';

const normalizeErrorText = (value: unknown): string => {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value ?? '');
};

const isDegradedModeError = (text: string): boolean =>
  text.includes('快速模式')
  || text.includes('quick_mode')
  || text.includes('quick mode')
  || text.includes('rule_only');

const isLlmConfigError = (text: string): boolean =>
  (text.includes('LLM') || text.includes('模型'))
  && (
    text.includes('配置')
    || text.includes('API')
    || text.includes('密钥')
    || text.includes('连接')
    || text.includes('未获取')
    || text.includes('超时')
  );

export const buildRuntimeErrorNotification = (
  label: string,
  rawError: unknown,
  idSeed?: string,
): NotificationItem => {
  const details = normalizeErrorText(rawError);
  const suffix = idSeed ?? `${Date.now()}`;
  if (isDegradedModeError(details)) {
    return {
      id: `runtime-error-degraded-${suffix}`,
      kind: 'validation',
      title: '已自动降级为快速模式',
      message: `${label}超时后已自动降级，可继续推进；如需完整质量可稍后重试标准模式。`,
      priority: 'banner',
    };
  }
  if (isLlmConfigError(details)) {
    return {
      id: `runtime-error-llm-${suffix}`,
      kind: 'error',
      title: '需要检查 LLM 配置',
      message: `${label}失败。请打开 LLM 设置检查模型、密钥与网络连接。`,
      priority: 'banner',
    };
  }
  return {
    id: `runtime-error-retry-${suffix}`,
    kind: 'error',
    title: `${label}失败，可重试`,
    message: details || '本次操作未完成，请稍后重试。',
    priority: 'toast',
  };
};

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
    const details = normalizeErrorText(error);
    console.error(`[runtime-error] ${label}: ${details}`);
    actionNotification.value = buildRuntimeErrorNotification(label, details);
  };

  return {
    actionNotification,
    logRuntimeAction,
    notifyRuntimeError,
  };
};
