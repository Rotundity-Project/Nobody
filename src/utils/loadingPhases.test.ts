import { describe, expect, it } from 'vitest';
import { buildStoryLoadingPhases } from './loadingPhases';

describe('loadingPhases', () => {
  it('builds DeepSeek-specific phases', () => {
    const phases = buildStoryLoadingPhases('deepseek-ai/DeepSeek-V3.2');
    expect(phases).toHaveLength(5);
    expect(phases[1].text).toContain('DeepSeek');
    expect(phases[2].text).toContain('DeepSeek');
  });

  it('builds Kimi-specific phases', () => {
    const phases = buildStoryLoadingPhases('moonshot-v1-8k');
    expect(phases).toHaveLength(5);
    expect(phases[1].text).toContain('Kimi');
    expect(phases[2].text).toContain('Kimi');
  });

  it('falls back to generic provider phases', () => {
    const phases = buildStoryLoadingPhases('gpt-4o-mini');
    expect(phases).toHaveLength(5);
    expect(phases[1].text).toContain('远程模型');
    expect(phases[2].text).toContain('远程模型');
  });

  it('keeps phase progress sequence stable', () => {
    const phases = buildStoryLoadingPhases(null);
    expect(phases.map((phase) => phase.progress)).toEqual([14, 34, 58, 78, 92]);
  });
});
