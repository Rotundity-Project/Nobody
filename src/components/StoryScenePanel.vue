<template>
  <div class="mx-auto max-w-3xl space-y-4">
    <div v-if="hasScene" class="prose prose-invert max-w-none">
      <h2 class="mb-4 text-2xl font-display text-amber-200">
        {{ chapterTitle }}
      </h2>
      <ChapterRecapCard :visible="showRecap" :summary="recapSummary" />
      <VirtualStoryList :paragraphs="paragraphs" :scroll-element="scrollElement" />
      <p v-if="optionSourceLabel" class="mt-3 font-mono text-xs text-slate-500">
        选项来源：{{ optionSourceLabel }}
      </p>
    </div>

    <div v-if="!isGameInitialized" class="text-center text-gray-400">
      <p>当前没有进行中的游戏，请先开始新游戏。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import ChapterRecapCard from './ChapterRecapCard.vue';
import VirtualStoryList from './VirtualStoryList.vue';

defineProps<{
  hasScene: boolean;
  chapterTitle: string;
  showRecap: boolean;
  recapSummary: string;
  paragraphs: string[];
  optionSourceLabel: string;
  isGameInitialized: boolean;
  scrollElement: HTMLElement | null;
}>();
</script>
