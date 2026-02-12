<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
    <div class="panel-surface rounded-2xl p-6 w-full max-w-lg">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-xl font-display text-amber-100">剧情设置</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200" @click="$emit('close')">
          关闭
        </button>
      </div>

      <div class="space-y-4">
        <label class="flex items-center justify-between gap-4 text-sm text-slate-300">
          <span>回顾上一章摘要</span>
          <input v-model="localSettings.recap_enabled" type="checkbox" class="accent-amber-400 h-4 w-4" />
        </label>

        <label class="text-sm text-slate-300">
          小说风格
          <select v-model="localSettings.novel_style" class="mt-2 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white">
            <option>修仙白话·第三人称</option>
            <option>修仙白话·第一人称</option>
            <option>修仙雅叙·第三人称</option>
            <option>修仙文言·第三人称</option>
          </select>
        </label>

        <button
          class="w-full rounded bg-amber-500 px-4 py-2 text-slate-900 font-medium"
          @click="handleSave"
        >
          保存设置
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import type { StorySettings } from '../utils/storySettings';

const props = defineProps<{
  isOpen: boolean;
  settings: StorySettings;
}>();

const emit = defineEmits<{
  close: [];
  save: [settings: StorySettings];
}>();

const localSettings = reactive<StorySettings>({
  recap_enabled: props.settings.recap_enabled,
  novel_style: props.settings.novel_style,
  min_interactions_per_chapter: props.settings.min_interactions_per_chapter,
  max_interactions_per_chapter: props.settings.max_interactions_per_chapter,
  target_chapter_words_min: props.settings.target_chapter_words_min,
  target_chapter_words_max: props.settings.target_chapter_words_max,
});

watch(
  () => props.settings,
  (next) => {
    localSettings.recap_enabled = next.recap_enabled;
    localSettings.novel_style = next.novel_style;
    localSettings.min_interactions_per_chapter = next.min_interactions_per_chapter;
    localSettings.max_interactions_per_chapter = next.max_interactions_per_chapter;
    localSettings.target_chapter_words_min = next.target_chapter_words_min;
    localSettings.target_chapter_words_max = next.target_chapter_words_max;
  },
  { deep: true },
);

const handleSave = () => {
  emit('save', { ...localSettings });
  emit('close');
};
</script>
