<template>
  <div class="w-full max-w-none">
    <div
      v-if="hasScene"
      class="max-w-none"
    >
      <ChapterRecapCard
        :visible="showRecap"
        :summary="recapSummary"
      />

      <section class="space-y-1 sm:space-y-1.5">
        <VirtualStoryList
          v-if="displayParagraphs.length > 0"
          :paragraphs="displayParagraphs"
          :scroll-element="scrollElement"
        />
        <p
          v-else
          class="text-sm text-[var(--ink-text-muted)]"
        >
          当前章节暂无正文内容。
        </p>
      </section>

      <p
        v-if="optionSourceLabel"
        class="mt-3 text-xs text-[var(--ink-text-muted)]"
      >
        选项来源：{{ optionSourceLabel }}
      </p>
    </div>

    <div
      v-if="!isGameInitialized"
      class="text-center text-[var(--ink-text-muted)]"
    >
      <p>当前没有进行中的游戏，请先开始新游戏。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ChapterRecapCard from './ChapterRecapCard.vue';
import VirtualStoryList from './VirtualStoryList.vue';

const props = defineProps<{
  hasScene: boolean;
  chapterTitle: string;
  showRecap: boolean;
  recapSummary: string;
  paragraphs: string[];
  optionSourceLabel: string;
  isGameInitialized: boolean;
  scrollElement: HTMLElement | null;
}>();

const displayParagraphs = computed(() => {
  if (props.paragraphs.length === 0) {
    return [];
  }
  let start = 0;
  while (start < props.paragraphs.length && props.paragraphs[start].trim().length === 0) {
    start += 1;
  }
  if (start >= props.paragraphs.length) {
    return [];
  }
  return props.paragraphs.slice(start).map((p) => p.trimEnd());
});
</script>

