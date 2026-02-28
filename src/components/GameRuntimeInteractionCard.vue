<template>
  <section class="runtime-card runtime-interaction-card">
    <h3 class="runtime-card-title runtime-interaction-title">交互面板</h3>
    <p class="runtime-sub-text runtime-interaction-subtitle">选项或自由输入</p>
    <GameInteractionPanel
      :should-show-input-panel="shouldShowInputPanel"
      :error="error"
      :is-no-input-advance-state="isNoInputAdvanceState"
      :available-options="availableOptions"
      :input-mode="inputMode"
      :free-text-input="freeTextInput"
      :input-validation="inputValidation"
      :loading-message="loadingMessage"
      :auto-advance-stop-hint="autoAdvanceStopHint"
      :is-loading="isLoading"
      :is-game-initialized="isGameInitialized"
      :is-waiting-for-input="isWaitingForInput"
      :can-stop-auto-advance="canStopAutoAdvance"
      @switch-mode="$emit('switch-mode', $event)"
      @select-option="$emit('select-option', $event)"
      @update:free-text-input="$emit('update:free-text-input', $event)"
      @submit-free-text="$emit('submit-free-text')"
      @continue="$emit('continue')"
      @stop-auto-advance="$emit('stop-auto-advance')"
    />
    <div v-if="shouldShowLlmSetupShortcut" class="runtime-llm-shortcut">
      <p class="runtime-sub-text">检测到本轮选项续写未命中 LLM，可直接打开设置后重试。</p>
      <button type="button" class="runtime-bottom-btn px-3 py-1 text-xs" @click="$emit('open-llm-settings')">
        打开 LLM 设置
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { PlayerOption } from '../types/game';
import GameInteractionPanel from './GameInteractionPanel.vue';

type InputValidation = {
  valid: boolean;
  message: string;
};

defineProps<{
  shouldShowInputPanel: boolean;
  error: string | null;
  isNoInputAdvanceState: boolean;
  availableOptions: PlayerOption[];
  inputMode: 'options' | 'freeText';
  freeTextInput: string;
  inputValidation: InputValidation;
  loadingMessage: string;
  autoAdvanceStopHint: string;
  isLoading: boolean;
  isGameInitialized: boolean;
  isWaitingForInput: boolean;
  canStopAutoAdvance: boolean;
  shouldShowLlmSetupShortcut: boolean;
}>();

defineEmits<{
  'switch-mode': [mode: 'options' | 'freeText'];
  'select-option': [option: PlayerOption];
  'update:free-text-input': [value: string];
  'submit-free-text': [];
  continue: [];
  'stop-auto-advance': [];
  'open-llm-settings': [];
}>();
</script>

<style scoped>
.runtime-card {
  position: relative;
  border-radius: 14px;
  border: 1px solid var(--ink-border-strong);
  background: var(--panel-bg, var(--ink-card-bg));
  box-shadow: var(--ink-shadow-card);
  padding: 20px;
  background-image: var(--runtime-card-sheen);
}

.runtime-card-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: 1.35;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
  display: inline-block;
}

.runtime-sub-text {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.6;
  letter-spacing: 0.01em;
}

.runtime-interaction-card {
  flex: 1;
  min-height: 0;
  overflow: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.runtime-interaction-title,
.runtime-interaction-subtitle {
  margin: 0;
}

.runtime-interaction-title {
  font-size: 18px;
  line-height: 1.35;
}

.runtime-interaction-subtitle {
  font-size: 14px;
  line-height: 1.6;
}

.runtime-interaction-card :deep(.mx-auto.max-w-3xl) {
  max-width: none;
  margin: 0;
}

.runtime-interaction-card :deep(.ink-interaction-panel) {
  padding: 0;
}

.runtime-interaction-card :deep(.free-text-input),
.runtime-interaction-card :deep(.free-text-foot),
.runtime-interaction-card :deep(.free-text-validation) {
  max-width: 100%;
}

.runtime-llm-shortcut {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-top: 1px dashed var(--ink-border-accent);
  padding-top: 10px;
}

.runtime-bottom-btn {
  border-radius: 8px;
  border: 1px solid var(--runtime-btn-border);
  border-top-color: var(--runtime-btn-border-top);
  border-bottom-color: var(--runtime-btn-border-bottom);
  background: var(--runtime-btn-bg);
  color: var(--ink-text-primary);
  padding: 8px 18px;
  box-shadow: var(--runtime-btn-shadow);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-bottom-btn:hover {
  border-color: var(--runtime-btn-hover-border);
  background: var(--runtime-btn-hover-bg);
  box-shadow: var(--runtime-btn-hover-shadow);
}

.runtime-bottom-btn:active {
  transform: scale(0.98);
}

.runtime-interaction-card :deep(.option-btn) {
  border-radius: 9px;
  border: 1px solid var(--runtime-option-btn-border);
  border-top-color: var(--runtime-option-btn-border-top);
  border-bottom-color: var(--runtime-option-btn-border-bottom);
  background: var(--runtime-option-btn-bg);
  box-shadow: var(--runtime-option-btn-shadow);
}

.runtime-interaction-card :deep(.option-btn:hover) {
  border-color: var(--runtime-option-btn-hover-border);
  box-shadow: var(--runtime-option-btn-hover-shadow);
}

@media (max-width: 1180px) {
  .runtime-interaction-card {
    height: auto;
    padding: 16px;
    gap: 6px;
  }
}
</style>
