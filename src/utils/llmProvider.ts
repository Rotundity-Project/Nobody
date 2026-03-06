export type LlmProviderKey = 'deepseek' | 'kimi' | 'generic';

export const resolveLlmProviderKey = (model: string | null | undefined): LlmProviderKey => {
  const lower = (model ?? '').toLowerCase();
  if (lower.includes('deepseek')) return 'deepseek';
  if (lower.includes('kimi') || lower.includes('moonshot')) return 'kimi';
  return 'generic';
};

export const getLlmProviderLabel = (providerKey: LlmProviderKey): string => {
  if (providerKey === 'deepseek') return 'DeepSeek';
  if (providerKey === 'kimi') return 'Kimi';
  return '远程模型';
};

export const getLlmProviderAccentVar = (providerKey: LlmProviderKey): string => {
  if (providerKey === 'deepseek') return 'var(--ink-text-cool)';
  if (providerKey === 'kimi') return 'var(--ink-title-color)';
  return 'var(--ink-accent-note)';
};
