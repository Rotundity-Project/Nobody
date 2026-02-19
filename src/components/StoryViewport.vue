<template>
  <div
    ref="scrollElement"
    class="relative flex-1 overflow-y-auto p-6 sm:p-8"
  >
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
import { ref } from 'vue';
import ScrollToBottomButton from './ScrollToBottomButton.vue';
import StoryScenePanel from './StoryScenePanel.vue';

defineProps<{
  hasScene: boolean;
  chapterTitle: string;
  showRecap: boolean;
  recapSummary: string;
  paragraphs: string[];
  optionSourceLabel: string;
  isGameInitialized: boolean;
}>();

const scrollElement = ref<HTMLElement | null>(null);

const scrollToBottom = () => {
  if (!scrollElement.value || typeof scrollElement.value.scrollTo !== 'function') {
    return;
  }
  scrollElement.value.scrollTo({
    top: scrollElement.value.scrollHeight,
    behavior: 'smooth',
  });
};

defineExpose({
  scrollToBottom,
});
</script>
