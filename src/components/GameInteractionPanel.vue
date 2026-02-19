<template>
  <div class="mx-auto max-w-3xl">
    <div
      v-if="shouldShowInputPanel"
      class="space-y-4"
    >
      <InputStatusNotice
        :error="error"
        :show-auto-advance-hint="isNoInputAdvanceState"
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
    />

    <div
      v-else-if="isGameInitialized && !isWaitingForInput"
      class="text-center"
    >
      <ContinueActionPanel button-text="继续写" @continue="$emit('continue')" />
    </div>
  </div>
</template>

<script setup lang="ts">
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
}>();

defineEmits<{
  (event: 'switch-mode', mode: 'options' | 'freeText'): void;
  (event: 'select-option', option: PlayerOption): void;
  (event: 'update:free-text-input', value: string): void;
  (event: 'submit-free-text'): void;
  (event: 'continue'): void;
}>();
</script>
