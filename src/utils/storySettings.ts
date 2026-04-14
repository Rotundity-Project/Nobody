export interface StorySettings {
  recap_enabled: boolean;
  novel_style: string;
  llm_priority_mode: boolean;
  llm_strict_mode: boolean;
  min_interactions_per_chapter: number;
  max_interactions_per_chapter: number;
  target_chapter_words_min: number;
  target_chapter_words_max: number;
}

const STORAGE_KEY = 'nobody_story_settings';
type SupportedNovelStyle =
  | 'xianxia-third-person'
  | 'xianxia-first-person'
  | 'xianxia-elegant-third-person'
  | 'xianxia-classical-third-person';
const NOVEL_STYLE_ALIAS_MAP: Record<string, SupportedNovelStyle> = {
  'xianxia-third-person': 'xianxia-third-person',
  'xianxia-first-person': 'xianxia-first-person',
  'xianxia-elegant-third-person': 'xianxia-elegant-third-person',
  'xianxia-classical-third-person': 'xianxia-classical-third-person',
  'xianxia-literary-third-person': 'xianxia-elegant-third-person',
  'xianxia-classic-third-person': 'xianxia-classical-third-person',
  '修仙白话·第三人称': 'xianxia-third-person',
  '修仙白话-第三人称': 'xianxia-third-person',
  '修仙白话·第一人称': 'xianxia-first-person',
  '修仙白话-第一人称': 'xianxia-first-person',
  '修仙雅叙·第三人称': 'xianxia-elegant-third-person',
  '修仙雅叙-第三人称': 'xianxia-elegant-third-person',
  '修仙文言·第三人称': 'xianxia-classical-third-person',
  '修仙文言-第三人称': 'xianxia-classical-third-person',
};

const defaultSettings: StorySettings = {
  recap_enabled: true,
  novel_style: 'xianxia-third-person',
  llm_priority_mode: true,
  llm_strict_mode: true,
  min_interactions_per_chapter: 2,
  max_interactions_per_chapter: 3,
  target_chapter_words_min: 5000,
  target_chapter_words_max: 7000,
};

const normalizeNovelStyle = (value: unknown): SupportedNovelStyle => {
  if (typeof value !== 'string') {
    return defaultSettings.novel_style as SupportedNovelStyle;
  }
  const trimmed = value.trim();
  if (!trimmed) {
    return defaultSettings.novel_style as SupportedNovelStyle;
  }
  return NOVEL_STYLE_ALIAS_MAP[trimmed] ?? (defaultSettings.novel_style as SupportedNovelStyle);
};

const sanitizeStorySettings = (input: Partial<StorySettings>): StorySettings => ({
  recap_enabled:
    typeof input.recap_enabled === 'boolean'
      ? input.recap_enabled
      : defaultSettings.recap_enabled,
  novel_style: normalizeNovelStyle(input.novel_style),
  llm_priority_mode:
    typeof input.llm_priority_mode === 'boolean'
      ? input.llm_priority_mode
      : defaultSettings.llm_priority_mode,
  llm_strict_mode:
    typeof input.llm_strict_mode === 'boolean'
      ? input.llm_strict_mode
      : defaultSettings.llm_strict_mode,
  min_interactions_per_chapter:
    typeof input.min_interactions_per_chapter === 'number'
      ? input.min_interactions_per_chapter
      : defaultSettings.min_interactions_per_chapter,
  max_interactions_per_chapter:
    typeof input.max_interactions_per_chapter === 'number'
      ? input.max_interactions_per_chapter
      : defaultSettings.max_interactions_per_chapter,
  target_chapter_words_min:
    typeof input.target_chapter_words_min === 'number'
      ? input.target_chapter_words_min
      : defaultSettings.target_chapter_words_min,
  target_chapter_words_max:
    typeof input.target_chapter_words_max === 'number'
      ? input.target_chapter_words_max
      : defaultSettings.target_chapter_words_max,
});

export const getStorySettings = (): StorySettings => {
  if (typeof window === 'undefined') {
    return { ...defaultSettings };
  }
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return { ...defaultSettings };
  try {
    const parsed = JSON.parse(raw) as Partial<StorySettings>;
    return sanitizeStorySettings(parsed);
  } catch {
    return { ...defaultSettings };
  }
};

export const saveStorySettings = (settings: StorySettings) => {
  if (typeof window === 'undefined') return;
  const sanitized = sanitizeStorySettings(settings);
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitized));
};
