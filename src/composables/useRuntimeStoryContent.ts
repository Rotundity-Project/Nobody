import { computed, type ComputedRef } from 'vue';
import { type useGameStore } from '../stores/gameStore';

const decodeEscapedText = (value: string): string =>
  value
    .replace(/\\\\/g, '\\')
    .replace(/\\"/g, '"')
    .replace(/\\n/g, '\n')
    .replace(/\\t/g, '\t');

const normalizeGeneratedBlock = (block: string): string => {
  const trimmed = block.trim();
  if (trimmed.length === 0) {
    return '';
  }
  if (!trimmed.includes('"text"')) {
    return trimmed;
  }
  const textField = trimmed.match(/"text"\s*:\s*"([\s\S]*?)"\s*(?:,|\})/);
  if (!textField || !textField[1]) {
    return trimmed;
  }
  return decodeEscapedText(textField[1]).trim();
};

export const useRuntimeStoryContent = ({
  gameStore,
  recapEnabled,
}: {
  gameStore: ReturnType<typeof useGameStore>;
  recapEnabled: ComputedRef<boolean>;
}) => {
  const currentChapterParagraphs = computed(() => {
    const content = gameStore.plotState?.current_chapter?.content ?? [];
    const combined = content.length > 0 ? content.join('\n\n') : gameStore.currentScene?.description ?? '';
    return combined
      .split(/\n{2,}/)
      .map((text) => normalizeGeneratedBlock(text))
      .flatMap((text) => text.split(/\n{2,}/))
      .map((text) => text.trim())
      .filter((text) => text.length > 0);
  });

  const lastChapterSummary = computed(() => {
    const chapters = gameStore.plotState?.chapters ?? [];
    return chapters.length > 0 ? chapters[chapters.length - 1]?.summary ?? '' : '';
  });

  const shouldShowRecap = computed(
    () => recapEnabled.value && lastChapterSummary.value.length > 0,
  );

  return {
    currentChapterParagraphs,
    lastChapterSummary,
    shouldShowRecap,
  };
};
