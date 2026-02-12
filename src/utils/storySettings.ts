export interface StorySettings {
  recap_enabled: boolean;
  novel_style: string;
  min_interactions_per_chapter: number;
  max_interactions_per_chapter: number;
  target_chapter_words_min: number;
  target_chapter_words_max: number;
}

const STORAGE_KEY = 'nobody_story_settings';

const defaultSettings: StorySettings = {
  recap_enabled: true,
  novel_style: '修仙白话·第三人称',
  min_interactions_per_chapter: 2,
  max_interactions_per_chapter: 3,
  target_chapter_words_min: 5000,
  target_chapter_words_max: 7000,
};

export const getStorySettings = (): StorySettings => {
  if (typeof window === 'undefined') {
    return { ...defaultSettings };
  }
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return { ...defaultSettings };
  try {
    const parsed = JSON.parse(raw) as Partial<StorySettings>;
    return {
      recap_enabled:
        typeof parsed.recap_enabled === 'boolean'
          ? parsed.recap_enabled
          : defaultSettings.recap_enabled,
      novel_style:
        typeof parsed.novel_style === 'string'
          ? parsed.novel_style
          : defaultSettings.novel_style,
      min_interactions_per_chapter:
        typeof parsed.min_interactions_per_chapter === 'number'
          ? parsed.min_interactions_per_chapter
          : defaultSettings.min_interactions_per_chapter,
      max_interactions_per_chapter:
        typeof parsed.max_interactions_per_chapter === 'number'
          ? parsed.max_interactions_per_chapter
          : defaultSettings.max_interactions_per_chapter,
      target_chapter_words_min:
        typeof parsed.target_chapter_words_min === 'number'
          ? parsed.target_chapter_words_min
          : defaultSettings.target_chapter_words_min,
      target_chapter_words_max:
        typeof parsed.target_chapter_words_max === 'number'
          ? parsed.target_chapter_words_max
          : defaultSettings.target_chapter_words_max,
    };
  } catch {
    return { ...defaultSettings };
  }
};

export const saveStorySettings = (settings: StorySettings) => {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
};
