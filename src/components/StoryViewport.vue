<template>
  <div class="story-viewport-frame relative flex-1 overflow-hidden rounded-[12px]">
    <div
      ref="scrollElement"
      class="runtime-story-scroll relative h-full overflow-y-auto"
      tabindex="0"
      @keydown="handleViewportKeydown"
    >
      <ScrollToBottomButton
        :visible="showScrollToBottomButton"
        @scroll="scrollToBottom"
      />
      <StoryScenePanel
        :has-scene="hasScene"
        :chapter-title="chapterTitle"
        :show-recap="showRecap && currentPage === 0"
        :recap-summary="recapSummary"
        :paragraphs="currentPageParagraphs"
        :option-source-label="optionSourceLabel"
        :is-game-initialized="isGameInitialized"
        :scroll-element="scrollElement"
      />
      <div
        class="story-reading-fade pointer-events-none absolute inset-x-0 bottom-0 h-20 rounded-b-[12px]"
        :style="readingFadeStyle"
      />
    </div>
    <div
      v-if="totalPages > 1"
      class="story-page-nav"
    >
      <button
        type="button"
        class="story-page-btn"
        :disabled="currentPage <= 0"
        @click="goPrevPage"
      >
        上一页
      </button>
      <span class="story-page-text">第 {{ currentPage + 1 }} / {{ totalPages }} 页</span>
      <button
        type="button"
        class="story-page-btn"
        :disabled="currentPage >= totalPages - 1"
        @click="goNextPage"
      >
        下一页
      </button>
    </div>
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
const hasScrollableContent = ref(false);
const currentPage = ref(0);

const PAGE_TRIGGER_CHAR_COUNT = 1200;
const PAGE_CHAR_BUDGET = 850;

const splitLongParagraph = (text: string, budget: number): string[] => {
  const trimmed = text.trim();
  if (trimmed.length <= budget) return [trimmed];
  const out: string[] = [];
  let start = 0;
  while (start < trimmed.length) {
    const next = Math.min(trimmed.length, start + budget);
    out.push(trimmed.slice(start, next));
    start = next;
  }
  return out;
};

const pagedParagraphGroups = computed(() => {
  const nonEmpty = props.paragraphs
    .map((text) => text.trim())
    .filter((text) => text.length > 0);
  if (nonEmpty.length === 0) return [[]];

  const totalChars = nonEmpty.reduce((sum, text) => sum + text.length, 0);
  if (totalChars <= PAGE_TRIGGER_CHAR_COUNT) {
    return [nonEmpty];
  }

  const pages: string[][] = [];
  let current: string[] = [];
  let used = 0;

  for (const para of nonEmpty) {
    for (const chunk of splitLongParagraph(para, PAGE_CHAR_BUDGET)) {
      const nextSize = used + chunk.length;
      if (current.length > 0 && nextSize > PAGE_CHAR_BUDGET) {
        pages.push(current);
        current = [];
        used = 0;
      }
      current.push(chunk);
      used += chunk.length;
    }
  }

  if (current.length > 0) {
    pages.push(current);
  }
  return pages.length > 0 ? pages : [nonEmpty];
});

const totalPages = computed(() => pagedParagraphGroups.value.length);
const currentPageParagraphs = computed(() => pagedParagraphGroups.value[currentPage.value] ?? []);

const showScrollToBottomButton = computed(
  () =>
    props.isGameInitialized
    && props.hasScene
    && props.paragraphs.length > 0
    && totalPages.value <= 1
    && hasScrollableContent.value
    && readingProgress.value < 0.95,
);
const readingFadeStyle = computed(() => ({
  background: 'linear-gradient(to top, color-mix(in srgb, var(--ink-bg-base) 86%, transparent), transparent)',
}));

const updateReadingProgress = () => {
  const el = scrollElement.value;
  if (!el) {
    hasScrollableContent.value = false;
    return;
  }
  const rawScrollable = el.scrollHeight - el.clientHeight;
  hasScrollableContent.value = rawScrollable > 24;
  const maxScrollable = Math.max(1, rawScrollable);
  const ratio = el.scrollTop / maxScrollable;
  readingProgress.value = Math.max(0, Math.min(1, Number.isFinite(ratio) ? ratio : 0));
};

const resolveScrollBehavior = (): ScrollBehavior => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'smooth';
  }
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth';
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

const goPrevPage = () => {
  if (currentPage.value <= 0) return;
  currentPage.value -= 1;
  void resetViewportScroll();
};

const goNextPage = () => {
  if (currentPage.value >= totalPages.value - 1) return;
  currentPage.value += 1;
  void resetViewportScroll();
};

const handleViewportKeydown = (event: KeyboardEvent) => {
  if (totalPages.value <= 1) return;
  if (event.key === 'ArrowLeft' || event.key === 'PageUp') {
    event.preventDefault();
    goPrevPage();
    return;
  }
  if (event.key === 'ArrowRight' || event.key === 'PageDown') {
    event.preventDefault();
    goNextPage();
  }
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
    currentPage.value = Math.max(0, totalPages.value - 1);
    updateReadingProgress();
  },
);

watch(
  () => [props.chapterTitle, props.paragraphs[0] ?? ''],
  () => {
    currentPage.value = Math.max(0, totalPages.value - 1);
    void resetViewportScroll();
  },
);

watch(
  totalPages,
  (next) => {
    if (currentPage.value > next - 1) {
      currentPage.value = Math.max(0, next - 1);
    }
  },
);

defineExpose({
  scrollToBottom,
});
</script>

<style scoped>
.story-page-nav {
  position: absolute;
  left: 50%;
  bottom: 12px;
  transform: translateX(-50%);
  z-index: 8;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  border: 1px solid var(--ink-border-accent, #b7a88c);
  border-radius: 999px;
  background: color-mix(in srgb, var(--ink-card-bg, #f5f0e8) 92%, transparent);
  padding: 6px 10px;
  backdrop-filter: blur(2px);
}

.story-page-btn {
  border: 1px solid var(--ink-border-strong, #d9c0b0);
  border-radius: 999px;
  background: var(--ink-paper, #faf7f2);
  color: var(--ink-text-primary, #2d2a24);
  padding: 3px 10px;
  font-size: 12px;
  transition: border-color 180ms ease, background-color 180ms ease, opacity 180ms ease;
}

.story-page-btn:hover:not(:disabled) {
  border-color: var(--ink-title-color, #b78c4a);
  background: #fff;
}

.story-page-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.story-page-text {
  color: var(--ink-text-muted, #5e5a54);
  font-size: 12px;
  min-width: 90px;
  text-align: center;
}

.story-reading-fade {
  z-index: 2;
}
</style>
