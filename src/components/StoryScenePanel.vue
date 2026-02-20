<template>
  <div class="mx-auto max-w-3xl">
    <div
      v-if="hasScene"
      class="max-w-none"
    >
      <header class="mb-2.5 border-b border-slate-700/70 pb-2 sm:mb-3 sm:pb-3">
        <p class="mb-1 text-[11px] uppercase tracking-[0.24em] text-amber-200/75 sm:text-xs sm:tracking-[0.28em]">章节阅读</p>
        <h2 class="mb-1 text-xl font-display text-amber-200 sm:mb-2 sm:text-2xl">
          {{ chapterTitle }}
        </h2>
        <div class="flex flex-wrap items-center gap-1.5 text-xs text-slate-200 sm:gap-2 sm:text-sm">
          <span class="rounded border border-slate-600/70 bg-slate-800/50 px-2 py-0.5">
            本章 {{ displayParagraphs.length }} 段
          </span>
          <span
            data-testid="rhythm-badge"
            class="rounded border border-amber-400/40 bg-amber-400/10 px-2 py-0.5 text-amber-200"
          >
            节奏：{{ rhythmLabel }}
          </span>
        </div>
      </header>

      <ChapterRecapCard
        :visible="showRecap"
        :summary="recapSummary"
      />

      <section class="space-y-1.5 sm:space-y-2">
        <h3 class="m-0 text-sm font-semibold tracking-wide text-slate-300">正文</h3>
        <VirtualStoryList
          v-if="displayParagraphs.length > 0"
          :paragraphs="displayParagraphs"
          :scroll-element="scrollElement"
        />
        <p
          v-else
          class="text-sm text-slate-500"
        >
          当前章节暂无正文内容。
        </p>
      </section>

      <p
        v-if="optionSourceLabel"
        class="mt-3 font-mono text-xs text-slate-500"
      >
        选项来源：{{ optionSourceLabel }}
      </p>
    </div>

    <div
      v-if="!isGameInitialized"
      class="text-center text-gray-400"
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

const rhythmLabel = computed(() => {
  if (displayParagraphs.value.length === 0) {
    return '未开始';
  }
  const avgLength = displayParagraphs.value.reduce((acc, p) => acc + p.length, 0) / displayParagraphs.value.length;
  if (avgLength < 36) {
    return '紧凑';
  }
  if (avgLength < 88) {
    return '均衡';
  }
  return '舒缓';
});
</script>
