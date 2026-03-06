import { describe, expect, it } from 'vitest';
import {
  getLlmProviderAccentVar,
  getLlmProviderLabel,
  resolveLlmProviderKey,
} from './llmProvider';

describe('llmProvider', () => {
  it('detects deepseek model ids', () => {
    expect(resolveLlmProviderKey('deepseek-ai/DeepSeek-V3.2')).toBe('deepseek');
    expect(resolveLlmProviderKey('DEEPSEEK-chat')).toBe('deepseek');
  });

  it('detects kimi and moonshot model ids', () => {
    expect(resolveLlmProviderKey('xopkimik25')).toBe('kimi');
    expect(resolveLlmProviderKey('moonshot-v1-8k')).toBe('kimi');
  });

  it('falls back to generic for unknown or empty model ids', () => {
    expect(resolveLlmProviderKey('gpt-4o-mini')).toBe('generic');
    expect(resolveLlmProviderKey('')).toBe('generic');
    expect(resolveLlmProviderKey(null)).toBe('generic');
    expect(resolveLlmProviderKey(undefined)).toBe('generic');
  });

  it('maps provider key to display label', () => {
    expect(getLlmProviderLabel('deepseek')).toBe('DeepSeek');
    expect(getLlmProviderLabel('kimi')).toBe('Kimi');
    expect(getLlmProviderLabel('generic')).toBe('远程模型');
  });

  it('maps provider key to accent css variable', () => {
    expect(getLlmProviderAccentVar('deepseek')).toBe('var(--ink-text-cool)');
    expect(getLlmProviderAccentVar('kimi')).toBe('var(--ink-title-color)');
    expect(getLlmProviderAccentVar('generic')).toBe('var(--ink-accent-note)');
  });
});
