import { computed, type ComputedRef, type Ref } from 'vue';
import type { PlayerOption } from '../types/game';
import type { InputMode, InputValidation } from './useInputMode';

export const useRuntimeInteractionCardBridge = ({
  shouldShowInputPanel,
  userFacingError,
  isNoInputAdvanceState,
  availableOptions,
  inputMode,
  isLoading,
  freeTextInput,
  inputValidation,
  isGameInitialized,
  isWaitingForInput,
  loadingMessage,
  autoAdvanceRunning,
  autoAdvanceStopHint,
  shouldShowLlmSetupShortcut,
  setInputMode,
  handleOptionSelect,
  setFreeTextInput,
  handleFreeTextSubmit,
  handleContinue,
  requestStopAutoAdvance,
  openLlmDialogFromError,
}: {
  shouldShowInputPanel: ComputedRef<boolean>;
  userFacingError: ComputedRef<string | null>;
  isNoInputAdvanceState: ComputedRef<boolean>;
  availableOptions: ComputedRef<PlayerOption[]>;
  inputMode: Ref<InputMode>;
  isLoading: Ref<boolean>;
  freeTextInput: Ref<string>;
  inputValidation: ComputedRef<InputValidation>;
  isGameInitialized: ComputedRef<boolean>;
  isWaitingForInput: ComputedRef<boolean>;
  loadingMessage: Ref<string>;
  autoAdvanceRunning: Ref<boolean>;
  autoAdvanceStopHint: Ref<string>;
  shouldShowLlmSetupShortcut: ComputedRef<boolean>;
  setInputMode: (mode: InputMode) => void;
  handleOptionSelect: (option: PlayerOption) => void;
  setFreeTextInput: (value: string) => void;
  handleFreeTextSubmit: () => void;
  handleContinue: () => void;
  requestStopAutoAdvance: () => void;
  openLlmDialogFromError: () => void;
}) => {
  const interactionCardProps = computed(() => ({
    shouldShowInputPanel: shouldShowInputPanel.value,
    error: userFacingError.value,
    isNoInputAdvanceState: isNoInputAdvanceState.value,
    availableOptions: availableOptions.value,
    inputMode: inputMode.value,
    isLoading: isLoading.value,
    freeTextInput: freeTextInput.value,
    inputValidation: inputValidation.value,
    isGameInitialized: isGameInitialized.value,
    isWaitingForInput: isWaitingForInput.value,
    loadingMessage: loadingMessage.value,
    canStopAutoAdvance: autoAdvanceRunning.value,
    autoAdvanceStopHint: autoAdvanceStopHint.value,
    shouldShowLlmSetupShortcut: shouldShowLlmSetupShortcut.value,
  }));

  const interactionCardListeners = {
    'switch-mode': setInputMode,
    'select-option': handleOptionSelect,
    'update:free-text-input': setFreeTextInput,
    'submit-free-text': handleFreeTextSubmit,
    continue: handleContinue,
    'stop-auto-advance': requestStopAutoAdvance,
    'open-llm-settings': openLlmDialogFromError,
  };

  return {
    interactionCardProps,
    interactionCardListeners,
  };
};
