<template>
  <div
    v-if="isOpen"
    class="settings-overlay fixed inset-0 z-50 flex items-center justify-center p-4"
  >
    <div class="panel-surface settings-panel w-full max-w-lg rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display settings-title">
          系统设置
        </h3>
        <button
          class="settings-close-btn rounded px-3 py-1 text-sm"
          @click="$emit('close')"
        >
          关闭
        </button>
      </div>

      <div class="space-y-4">
        <label class="text-sm settings-label">
          界面主题
          <select
            v-model="localTheme"
            class="settings-select mt-2 w-full rounded px-3 py-2"
          >
            <option value="theme-scroll">浅色古风</option>
            <option value="theme-night">深色风格</option>
          </select>
        </label>

        <label class="settings-label flex items-center justify-between gap-4 text-sm">
          <span>显示上一章回顾</span>
          <input
            v-model="localSettings.recap_enabled"
            type="checkbox"
            class="h-4 w-4 settings-checkbox"
          >
        </label>

        <label class="text-sm settings-label">
          小说风格
          <select
            v-model="localSettings.novel_style"
            class="settings-select mt-2 w-full rounded px-3 py-2"
          >
            <option value="xianxia-third-person">修仙白话·第三人称</option>
            <option value="xianxia-first-person">修仙白话·第一人称</option>
            <option value="xianxia-elegant-third-person">修仙雅叙·第三人称</option>
            <option value="xianxia-classical-third-person">修仙文言·第三人称</option>
          </select>
        </label>

        <label class="settings-label flex items-center justify-between gap-4 text-sm">
          <span>LLM 优先剧情生成</span>
          <input
            v-model="localSettings.llm_priority_mode"
            type="checkbox"
            class="h-4 w-4 settings-checkbox"
          >
        </label>

        <label class="settings-label flex items-center justify-between gap-4 text-sm">
          <span>强制 LLM（失败不推进）</span>
          <input
            v-model="localSettings.llm_strict_mode"
            type="checkbox"
            class="h-4 w-4 settings-checkbox"
          >
        </label>

        <button
          class="settings-save-btn w-full rounded px-4 py-2 font-medium"
          @click="handleSave"
        >
          保存设置
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
import type { StorySettings } from '../utils/storySettings';
import type { UiTheme } from '../utils/uiTheme';

const props = defineProps<{
  isOpen: boolean;
  settings: StorySettings;
  uiTheme: UiTheme;
}>();

const emit = defineEmits<{
  close: [];
  save: [settings: StorySettings];
  'update-theme': [theme: UiTheme];
}>();

const localSettings = reactive<StorySettings>({
  recap_enabled: props.settings.recap_enabled,
  novel_style: props.settings.novel_style,
  llm_priority_mode: props.settings.llm_priority_mode ?? true,
  llm_strict_mode: props.settings.llm_strict_mode ?? true,
  min_interactions_per_chapter: props.settings.min_interactions_per_chapter,
  max_interactions_per_chapter: props.settings.max_interactions_per_chapter,
  target_chapter_words_min: props.settings.target_chapter_words_min,
  target_chapter_words_max: props.settings.target_chapter_words_max,
});
const localTheme = ref<UiTheme>(props.uiTheme);

watch(
  () => props.settings,
  (next) => {
    localSettings.recap_enabled = next.recap_enabled;
    localSettings.novel_style = next.novel_style;
    localSettings.llm_priority_mode = next.llm_priority_mode ?? true;
    localSettings.llm_strict_mode = next.llm_strict_mode ?? true;
    localSettings.min_interactions_per_chapter = next.min_interactions_per_chapter;
    localSettings.max_interactions_per_chapter = next.max_interactions_per_chapter;
    localSettings.target_chapter_words_min = next.target_chapter_words_min;
    localSettings.target_chapter_words_max = next.target_chapter_words_max;
  },
  { deep: true },
);

watch(
  () => props.uiTheme,
  (nextTheme) => {
    localTheme.value = nextTheme;
  },
);

const handleSave = () => {
  emit('save', { ...localSettings });
  emit('update-theme', localTheme.value);
  emit('close');
};
</script>

<style scoped>
.settings-overlay {
  background: var(--settings-overlay-bg);
}

.settings-panel {
  border: 1px solid var(--ink-border-soft);
}

.settings-title {
  color: var(--ink-title-color);
}

.settings-label {
  color: var(--ink-text-primary);
}

.settings-close-btn {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-btn-muted-bg);
  color: var(--ink-text-primary);
}

.settings-select {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-input-bg);
  color: var(--ink-text-primary);
}

.settings-checkbox {
  accent-color: var(--ink-title-color);
}

.settings-save-btn {
  border: 1px solid var(--ink-border-accent);
  background: var(--settings-btn-accent-strong-bg);
  color: var(--ink-text-primary);
}
</style>
