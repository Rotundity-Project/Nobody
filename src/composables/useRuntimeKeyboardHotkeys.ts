import { type ComputedRef, type Ref } from 'vue';
import { type useGameStore } from '../stores/gameStore';
import { useGameHotkeys } from './useGameHotkeys';

export const useRuntimeKeyboardHotkeys = ({
  gameStore,
  closeAllDialogs,
  closeRuntimeQuickPanels,
  hasBlockingOverlay,
  isNoInputAdvanceState,
  inputMode,
  freeTextInput,
  handleContinue,
  handleFreeTextSubmit,
  handleOptionSelect,
  showSaveDialog,
}: {
  gameStore: ReturnType<typeof useGameStore>;
  closeAllDialogs: () => void;
  closeRuntimeQuickPanels: () => void;
  hasBlockingOverlay: ComputedRef<boolean>;
  isNoInputAdvanceState: ComputedRef<boolean>;
  inputMode: Ref<string>;
  freeTextInput: Ref<string>;
  handleContinue: () => void;
  handleFreeTextSubmit: () => void;
  handleOptionSelect: (option: ReturnType<typeof useGameStore>['availableOptions'][number]) => void;
  showSaveDialog: Ref<boolean>;
}) => {
  const handleKeydown = (event: KeyboardEvent) => {
    if (!gameStore.isGameInitialized) {
      return;
    }

    if (event.key === 'Escape') {
      closeAllDialogs();
      closeRuntimeQuickPanels();
      return;
    }

    const target = event.target instanceof HTMLElement ? event.target : null;
    const inTextInput = Boolean(
      target
      && (
        target.tagName === 'TEXTAREA'
        || target.tagName === 'INPUT'
        || target.tagName === 'SELECT'
        || target.isContentEditable
      ),
    );
    if (inTextInput || hasBlockingOverlay.value) {
      return;
    }

    if (event.key === 'Enter' && isNoInputAdvanceState.value) {
      event.preventDefault();
      handleContinue();
      return;
    }

    if (event.key === 'Enter' && inputMode.value === 'freeText' && freeTextInput.value.trim()) {
      event.preventDefault();
      handleFreeTextSubmit();
    }

    if (inputMode.value === 'options' && gameStore.availableOptions.length > 0) {
      const num = parseInt(event.key);
      if (num >= 1 && num <= 5 && num <= gameStore.availableOptions.length) {
        event.preventDefault();
        const option = gameStore.availableOptions[num - 1];
        handleOptionSelect(option);
      }
    }

    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      if (gameStore.isGameInitialized) {
        showSaveDialog.value = true;
      }
    }
  };

  useGameHotkeys(handleKeydown);
};
