<template>
  <div
    ref="scrollElement"
    class="relative flex-1 overflow-y-auto px-4 pb-5 pt-0 sm:px-6 sm:pb-6 sm:pt-1 md:px-7 md:pb-7 md:pt-1 xl:px-8 xl:pb-8 xl:pt-2"
  >
    <div
      v-if="showReadingLocator"
      class="pointer-events-none absolute right-3 top-2 z-20 sm:right-4"
    >
      <div
        data-testid="reading-locator"
        class="pointer-events-auto w-fit max-w-[min(84vw,22rem)] rounded-lg border border-slate-700/90 bg-slate-900/90 px-2 py-1.5 text-[11px] text-slate-200 shadow-sm backdrop-blur sm:px-3 sm:py-2 sm:text-xs"
      >
        <div class="flex items-center justify-between gap-2 sm:gap-3">
          <p
            data-testid="reading-locator-summary"
            aria-live="polite"
            aria-atomic="true"
          >
            阅读定位：{{ readingProgressPercent }}% · {{ readingProgressCompact }}
          </p>
          <button
            data-testid="toggle-reading-locator"
            class="rounded bg-slate-700 px-1.5 py-0.5 text-[10px] text-slate-100 hover:bg-slate-600 sm:px-2 sm:text-[11px]"
            :aria-expanded="showReadingLocatorDetails ? 'true' : 'false'"
            :aria-controls="READING_LOCATOR_DETAILS_ID"
            :aria-label="showReadingLocatorDetails ? '收起阅读定位详情' : '展开阅读定位详情'"
            @click="toggleReadingLocatorDetails"
          >
            {{ showReadingLocatorDetails ? '收起' : '展开' }}
          </button>
        </div>
        <div
          v-if="showReadingLocatorDetails"
          :id="READING_LOCATOR_DETAILS_ID"
          class="mt-2 space-y-2"
        >
          <p>段落进度：{{ currentParagraphIndex }} / {{ paragraphs.length }}</p>
          <div class="flex gap-2">
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
      </div>
    </div>

    <ScrollToBottomButton
      :visible="showScrollToBottomButton"
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
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
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
const showReadingLocatorDetails = ref(false);
const READING_LOCATOR_STORAGE_KEY = 'nobody_reading_locator_expanded';
const READING_LOCATOR_DETAILS_ID = 'reading-locator-details';

const showReadingLocator = computed(
  () => props.isGameInitialized && props.hasScene && props.paragraphs.length > 0,
);
const showScrollToBottomButton = computed(
  () => showReadingLocator.value && readingProgress.value < 0.95,
);
const readingProgressPercent = computed(() => Math.round(readingProgress.value * 100));
const currentParagraphIndex = computed(() => {
  if (props.paragraphs.length === 0) {
    return 0;
  }
  const estimated = Math.round(readingProgress.value * props.paragraphs.length);
  return Math.min(props.paragraphs.length, Math.max(1, estimated));
});
const readingProgressCompact = computed(() => `${currentParagraphIndex.value}/${props.paragraphs.length}`);

const updateReadingProgress = () => {
  const el = scrollElement.value;
  if (!el) {
    return;
  }
  const maxScrollable = Math.max(1, el.scrollHeight - el.clientHeight);
  const ratio = el.scrollTop / maxScrollable;
  readingProgress.value = Math.max(0, Math.min(1, Number.isFinite(ratio) ? ratio : 0));
};

const getStoredLocatorExpanded = (): boolean | null => {
  if (typeof window === 'undefined') {
    return null;
  }
  const raw = window.localStorage.getItem(READING_LOCATOR_STORAGE_KEY);
  if (raw == null) {
    return null;
  }
  return raw === '1';
};

const persistLocatorExpanded = (expanded: boolean) => {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(READING_LOCATOR_STORAGE_KEY, expanded ? '1' : '0');
};

const toggleReadingLocatorDetails = () => {
  showReadingLocatorDetails.value = !showReadingLocatorDetails.value;
  persistLocatorExpanded(showReadingLocatorDetails.value);
};

const resolveScrollBehavior = (): ScrollBehavior => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'smooth';
  }
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth';
};

const scrollToTop = () => {
  if (!scrollElement.value || typeof scrollElement.value.scrollTo !== 'function') {
    return;
  }
  scrollElement.value.scrollTo({
    top: 0,
    behavior: resolveScrollBehavior(),
  });
};

const scrollToBottom = () => {
  if (!scrollElement.value || typeof scrollElement.value.scrollTo !== 'function') {
    return;
  }
  scrollElement.value.scrollTo({
    top: scrollElement.value.scrollHeight,
    behavior: resolveScrollBehavior(),
  });
};

const resetViewportScroll = async () => {
  await nextTick();
  const el = scrollElement.value;
  if (!el) {
    return;
  }
  if (typeof el.scrollTo === 'function') {
    el.scrollTo({ top: 0, behavior: 'auto' });
  }
  el.scrollTop = 0;
  updateReadingProgress();
};

onMounted(() => {
  scrollElement.value?.addEventListener('scroll', updateReadingProgress, { passive: true });
  const stored = getStoredLocatorExpanded();
  if (stored != null) {
    showReadingLocatorDetails.value = stored;
  }
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

watch(
  () => [props.chapterTitle, props.paragraphs[0] ?? ''],
  () => {
    void resetViewportScroll();
  },
);

defineExpose({
  scrollToBottom,
});
</script>
