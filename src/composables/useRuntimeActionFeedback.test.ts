import { describe, expect, it } from 'vitest';
import { buildRuntimeErrorNotification } from './useRuntimeActionFeedback';

describe('buildRuntimeErrorNotification', () => {
  it('classifies degraded quick-mode errors', () => {
    const item = buildRuntimeErrorNotification('剧情推进', '主链路超时，已切换快速模式续写。', 'seed-a');
    expect(item.id).toBe('runtime-error-degraded-seed-a');
    expect(item.kind).toBe('validation');
    expect(item.priority).toBe('banner');
  });

  it('classifies llm setup errors', () => {
    const item = buildRuntimeErrorNotification('剧情推进', 'LLM 配置无效，请检查 API 密钥', 'seed-b');
    expect(item.id).toBe('runtime-error-llm-seed-b');
    expect(item.kind).toBe('error');
    expect(item.priority).toBe('banner');
  });

  it('falls back to retry guidance', () => {
    const item = buildRuntimeErrorNotification('存档', '磁盘写入失败', 'seed-c');
    expect(item.id).toBe('runtime-error-retry-seed-c');
    expect(item.kind).toBe('error');
    expect(item.priority).toBe('toast');
    expect(item.title).toContain('可重试');
  });
});
