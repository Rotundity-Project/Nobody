<template>
  <div class="mx-auto w-full min-w-0 max-w-3xl">
    <UiPanel
      padding="md"
      class="ink-interaction-panel"
    >
      <div
        v-if="shouldShowInputPanel"
        class="ink-interaction-stack"
      >
        <InputStatusNotice
          :error="error"
          :show-auto-advance-hint="isNoInputAdvanceState"
          :auto-advance-stop-hint="autoAdvanceStopHint"
        />
        <InputModeTabs
          :visible="availableOptions.length > 0"
          :mode="inputMode"
          @switch-mode="$emit('switch-mode', $event)"
        />

        <OptionListPanel
          :visible="inputMode === 'options' && availableOptions.length > 0"
          :options="availableOptions"
          :disabled="isLoading"
          @select="$emit('select-option', $event)"
        />

        <FreeTextInputPanel
          :visible="inputMode === 'freeText' || availableOptions.length === 0"
          :model-value="freeTextInput"
          :disabled="isLoading"
          :valid="inputValidation.valid"
          :validation-message="inputValidation.message"
          @update:model-value="$emit('update:free-text-input', $event)"
          @submit="$emit('submit-free-text')"
        />
      </div>

      <div
        v-else-if="isNoInputAdvanceState && !isLoading"
        class="text-center"
      >
        <ContinueActionPanel
          message="当前无需输入，点击继续即可推进剧情。"
          button-text="继续推进剧情"
          @continue="$emit('continue')"
        />
      </div>

      <LoadingStatePanel
        v-else-if="isLoading"
        :message="loadingMessage"
        :stage="loadingStage"
        :progress="loadingProgress"
        :progress-text="loadingProgressText"
        :elapsed-ms="loadingElapsedMs ?? null"
        :can-stop-auto-advance="canStopAutoAdvance"
        @stop-auto-advance="$emit('stop-auto-advance')"
      />

      <div
        v-else-if="isGameInitialized && !isWaitingForInput"
        class="text-center"
      >
        <ContinueActionPanel
          button-text="继续写"
          @continue="$emit('continue')"
        />
      </div>
    </UiPanel>
  </div>
</template>

<script setup lang="ts">
import UiPanel from '../shared/ui/UiPanel.vue';
import ContinueActionPanel from './ContinueActionPanel.vue';
import FreeTextInputPanel from './FreeTextInputPanel.vue';
import InputModeTabs from './InputModeTabs.vue';
import InputStatusNotice from './InputStatusNotice.vue';
import LoadingStatePanel from './LoadingStatePanel.vue';
import OptionListPanel from './OptionListPanel.vue';
import type { PlayerOption } from '../types/game';

defineProps<{
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
  isGameInitialized: boolean;
  isWaitingForInput: boolean;
  loadingMessage: string;
  loadingStage?: string;
  loadingProgress: number | null;
  loadingProgressText: string;
  loadingElapsedMs?: number | null;
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
</script>

<style scoped>
.ink-interaction-stack {
  display: grid;
  gap: 8px;
  min-width: 0;
}

.ink-interaction-panel {
  padding: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
}
</style>
