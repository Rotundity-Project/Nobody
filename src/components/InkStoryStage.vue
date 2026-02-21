<template>
  <section class="ink-stage animate-fade-up">
    <div class="ink-stage-scroll">
      <StoryViewport
        ref="viewportRef"
        :has-scene="hasScene"
        :chapter-title="chapterTitle"
        :show-recap="showRecap"
        :recap-summary="recapSummary"
        :paragraphs="paragraphs"
        :option-source-label="optionSourceLabel"
        :is-game-initialized="isGameInitialized"
      />
    </div>

    <div class="ink-input-zone">
      <GameInteractionPanel
        :should-show-input-panel="shouldShowInputPanel"
        :error="error"
        :is-no-input-advance-state="isNoInputAdvanceState"
        :available-options="availableOptions"
        :input-mode="inputMode"
        :is-loading="isLoading"
        :free-text-input="freeTextInput"
        :input-validation="inputValidation"
        :is-game-initialized="isGameInitialized"
        :is-waiting-for-input="isWaitingForInput"
        :loading-message="loadingMessage"
        :can-stop-auto-advance="canStopAutoAdvance"
        :auto-advance-stop-hint="autoAdvanceStopHint"
        @switch-mode="$emit('switch-mode', $event)"
        @select-option="$emit('select-option', $event)"
        @update:free-text-input="$emit('update:free-text-input', $event)"
        @submit-free-text="$emit('submit-free-text')"
        @continue="$emit('continue')"
        @stop-auto-advance="$emit('stop-auto-advance')"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import StoryViewport from './StoryViewport.vue';
import GameInteractionPanel from './GameInteractionPanel.vue';
import type { PlayerOption } from '../types/game';

defineProps<{
  hasScene: boolean;
  chapterTitle: string;
  showRecap: boolean;
  recapSummary: string;
  paragraphs: string[];
  optionSourceLabel: string;
  isGameInitialized: boolean;
  shouldShowInputPanel: boolean;
  error: string | null;
  isNoInputAdvanceState: boolean;
  availableOptions: PlayerOption[];
  inputMode: 'options' | 'freeText';
  isLoading: boolean;
  freeTextInput: string;
  inputValidation: {
    valid: boolean;
    message: string;
  };
  isWaitingForInput: boolean;
  loadingMessage: string;
  canStopAutoAdvance: boolean;
  autoAdvanceStopHint: string;
}>();

defineEmits<{
  (event: 'switch-mode', mode: 'options' | 'freeText'): void;
  (event: 'select-option', option: PlayerOption): void;
  (event: 'update:free-text-input', value: string): void;
  (event: 'submit-free-text'): void;
  (event: 'continue'): void;
  (event: 'stop-auto-advance'): void;
}>();

const viewportRef = ref<{ scrollToBottom: () => void } | null>(null);

const scrollToBottom = () => {
  viewportRef.value?.scrollToBottom();
};

defineExpose({
  scrollToBottom,
});
</script>
