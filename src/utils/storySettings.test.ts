import { describe, expect, it } from 'vitest';
import { getStorySettings, saveStorySettings } from './storySettings';

describe('storySettings', () => {
  it('uses non-strict llm opening by default', () => {
    window.localStorage.removeItem('nobody_story_settings');
    expect(getStorySettings().llm_strict_mode).toBe(false);
  });

  it('normalizes compatible novel style labels from storage', () => {
    window.localStorage.setItem('nobody_story_settings', JSON.stringify({
      novel_style: '修仙白话·第三人称',
    }));
    expect(getStorySettings().novel_style).toBe('xianxia-third-person');
  });

  it('falls back to default novel style for invalid value', () => {
    window.localStorage.setItem('nobody_story_settings', JSON.stringify({
      novel_style: 'unknown-style',
    }));
    expect(getStorySettings().novel_style).toBe('xianxia-third-person');
  });

  it('persists sanitized story settings', () => {
    saveStorySettings({
      recap_enabled: true,
      novel_style: '修仙雅叙·第三人称',
      llm_priority_mode: true,
      llm_strict_mode: false,
      min_interactions_per_chapter: 2,
      max_interactions_per_chapter: 3,
      target_chapter_words_min: 5000,
      target_chapter_words_max: 7000,
    });
    const raw = window.localStorage.getItem('nobody_story_settings');
    expect(raw).toBeTruthy();
    const parsed = JSON.parse(raw as string) as { novel_style: string };
    expect(parsed.novel_style).toBe('xianxia-elegant-third-person');
  });
});
