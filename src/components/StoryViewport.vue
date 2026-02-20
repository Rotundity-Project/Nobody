<template>
  <div
    ref="scrollElement"
    class="relative flex-1 overflow-y-auto px-4 pb-5 pt-2 sm:px-6 sm:pb-6 sm:pt-3 md:px-7 md:pb-7 md:pt-3 xl:px-8 xl:pb-8 xl:pt-4"
  >
    <div
      v-if="showReadingLocator"
      class="sticky top-2 z-10 ml-auto mb-2 w-fit rounded-lg border border-slate-700 bg-slate-900/85 px-3 py-2 text-xs text-slate-200 backdrop-blur"
    >
      <p>阅读定位：{{ readingProgressPercent }}%</p>
      <p>段落进度：{{ currentParagraphIndex }} / {{ paragraphs.length }}</p>
      <div class="mt-2 flex gap-2">
        <button
          class="rounded bg-slate-700 px-2 py-1 text-[11px] text-slate-100 hover:bg-slate-600"
          @click="scrollToTop"
        >
          顶部
        </button>
        <button
          class="rounded bg-slate-700 px-2 py-1 text-[11px] text-slate-100 hover:bg-slate-600"
          @click="scrollToBottom"
        >
          底部
        </button>
      </div>
    </div>

    <ScrollToBottomButton
      :visible="isGameInitialized"
      @scroll="scrollToBottom"
    />
    <StoryScenePanel
      :has-scene="hasScene"
      :chapter-title="chapterTitle"
      :show-recap="showRecap"
      :recap-summary="recapSummary"
      :paragraphs="paragraphs"
      :option-source-label="optionSourceLabel"
      :is-game-initialized="isGameInitialized"
      :scroll-element="scrollElement"
    />
    <div class="pointer-events-none absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-slate-950 to-transparent" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import ScrollToBottomButton from './ScrollToBottomButton.vue';
import StoryScenePanel from './StoryScenePanel.vue';

const props = defineProps<{
  hasScene: boolean;
  chapterTitle: string;
  showRecap: boolean;
  recapSummary: string;
  paragraphs: string[];
  optionSourceLabel: string;
  isGameInitialized: boolean;
}>();

const scrollElement = ref<HTMLElement | null>(null);
const readingProgress = ref(0);

const showReadingLocator = computed(
  () => props.isGameInitialized && props.hasScene && props.paragraphs.length > 0,
);
const readingProgressPercent = computed(() => Math.round(readingProgress.value * 100));
const currentParagraphIndex = computed(() => {
  if (props.paragraphs.length === 0) {
    return 0;
  }
  const estimated = Math.round(readingProgress.value * props.paragraphs.length);
  return Math.min(props.paragraphs.length, Math.max(1, estimated));
});

const updateReadingProgress = () => {
  const el = scrollElement.value;
  if (!el) {
    return;
  }
  const maxScrollable = Math.max(1, el.scrollHeight - el.clientHeight);
  const ratio = el.scrollTop / maxScrollable;
  readingProgress.value = Math.max(0, Math.min(1, Number.isFinite(ratio) ? ratio : 0));
};

const scrollToTop = () => {
  if (!scrollElement.value || typeof scrollElement.value.scrollTo !== 'function') {
    return;
  }
  scrollElement.value.scrollTo({
    top: 0,
    behavior: 'smooth',
  });
};

const scrollToBottom = () => {
  if (!scrollElement.value || typeof scrollElement.value.scrollTo !== 'function') {
    return;
  }
  scrollElement.value.scrollTo({
    top: scrollElement.value.scrollHeight,
    behavior: 'smooth',
  });
};

onMounted(() => {
  scrollElement.value?.addEventListener('scroll', updateReadingProgress, { passive: true });
  updateReadingProgress();
});

onUnmounted(() => {
  scrollElement.value?.removeEventListener('scroll', updateReadingProgress);
});

watch(
  () => props.paragraphs.length,
  () => {
    updateReadingProgress();
  },
);

defineExpose({
  scrollToBottom,
});
</script>
