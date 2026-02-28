import { watch, watchEffect, type ComputedRef, type Ref } from 'vue';
import type { StorySettings } from '../utils/storySettings';

export const useRuntimeViewportEffects = ({
  isPlotInitialized,
  storySettings,
  applyStorySettings,
  currentChapterParagraphs,
  shouldAutoFollowNewParagraph,
  previousChapterParagraphs,
  scrollToBottom,
}: {
  isPlotInitialized: ComputedRef<boolean>;
  storySettings: Ref<StorySettings>;
  applyStorySettings: (settings: StorySettings) => Promise<void>;
  currentChapterParagraphs: ComputedRef<string[]>;
  shouldAutoFollowNewParagraph: ComputedRef<boolean>;
  previousChapterParagraphs: Ref<string[]>;
  scrollToBottom: () => void;
}) => {
  watchEffect(() => {
    if (isPlotInitialized.value) {
      void applyStorySettings(storySettings.value);
    }
  });

  watch(currentChapterParagraphs, (newParagraphs) => {
    if (
      shouldAutoFollowNewParagraph.value
      && newParagraphs.length > previousChapterParagraphs.value.length
    ) {
      requestAnimationFrame(() => {
        scrollToBottom();
      });
    }
    previousChapterParagraphs.value = [...newParagraphs];
  }, { deep: true });
};
