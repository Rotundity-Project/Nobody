<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
    <div class="panel-surface w-full max-w-lg rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display text-amber-100">系统设置</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200" @click="$emit('close')">
          关闭
        </button>
      </div>

      <div class="space-y-4">
        <label class="text-sm text-slate-300">
          界面主题
          <select
            v-model="localTheme"
            class="mt-2 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white"
          >
            <option value="theme-scroll">浅色古风</option>
            <option value="theme-night">深色风格</option>
          </select>
        </label>

        <label class="flex items-center justify-between gap-4 text-sm text-slate-300">
          <span>显示上一章回顾</span>
          <input v-model="localSettings.recap_enabled" type="checkbox" class="h-4 w-4 accent-amber-400" />
        </label>

        <label class="text-sm text-slate-300">
          小说风格
          <select
            v-model="localSettings.novel_style"
            class="mt-2 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white"
          >
            <option value="xianxia-third-person">修仙白话·第三人称</option>
            <option value="xianxia-first-person">修仙白话·第一人称</option>
            <option value="xianxia-elegant-third-person">修仙雅叙·第三人称</option>
            <option value="xianxia-classical-third-person">修仙文言·第三人称</option>
          </select>
        </label>

        <label class="flex items-center justify-between gap-4 text-sm text-slate-300">
          <span>LLM 优先剧情生成</span>
          <input v-model="localSettings.llm_priority_mode" type="checkbox" class="h-4 w-4 accent-amber-400" />
        </label>

        <label class="flex items-center justify-between gap-4 text-sm text-slate-300">
          <span>强制 LLM（失败不推进）</span>
          <input v-model="localSettings.llm_strict_mode" type="checkbox" class="h-4 w-4 accent-amber-400" />
        </label>

        <button
          class="w-full rounded bg-amber-500 px-4 py-2 font-medium text-slate-900"
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
