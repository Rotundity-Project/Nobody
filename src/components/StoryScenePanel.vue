<template>
  <div class="mx-auto max-w-3xl space-y-5">
    <div
      v-if="hasScene"
      class="prose prose-invert max-w-none"
    >
      <header class="mb-5 border-b border-slate-700/70 pb-4">
        <p class="mb-1 text-xs uppercase tracking-[0.28em] text-amber-200/75">章节阅读</p>
        <h2 class="mb-2 text-2xl font-display text-amber-200">
          {{ chapterTitle }}
        </h2>
        <p class="m-0 text-sm text-slate-200">
          本章共 {{ paragraphs.length }} 段，节奏：{{ rhythmLabel }}
        </p>
      </header>

      <ChapterRecapCard
        :visible="showRecap"
        :summary="recapSummary"
      />

      <section class="space-y-2">
        <h3 class="m-0 text-sm font-semibold tracking-wide text-slate-300">正文</h3>
        <VirtualStoryList
          v-if="paragraphs.length > 0"
          :paragraphs="paragraphs"
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

const rhythmLabel = computed(() => {
  if (props.paragraphs.length === 0) {
    return '未开始';
  }
  const avgLength = props.paragraphs.reduce((acc, p) => acc + p.length, 0) / props.paragraphs.length;
  if (avgLength < 36) {
    return '紧凑';
  }
  if (avgLength < 88) {
    return '均衡';
  }
  return '舒缓';
});
</script>
