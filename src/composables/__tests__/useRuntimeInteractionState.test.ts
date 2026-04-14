import { ref } from 'vue';
import { describe, expect, it } from 'vitest';
import { useRuntimeInteractionState } from '../useRuntimeInteractionState';

describe('useRuntimeInteractionState', () => {
  it('prefers noname option bias hint from diagnostics', () => {
    const state = useRuntimeInteractionState({
      gameStore: {
        isGameInitialized: true,
        isWaitingForInput: true,
        availableOptions: [],
        reachableLocationIds: [],
        mapOverview: [],
        error: null,
        plotState: {
          interaction_state: 'waiting_for_choice',
          last_option_generation_source: 'llm_structured',
          last_generation_diagnostics: '选项来源：llm_structured；NoName选项偏置：下轮优先围绕山门危机提供行动切入点',
        },
      } as never,
      isDevMode: true,
      showSaveDialog: ref(false),
      showLoadDialog: ref(false),
      showLLMDialog: ref(false),
      showStorySettings: ref(false),
      showConsistencySettings: ref(false),
      showInfoTabs: ref(false),
      showCharacterInfo: ref(false),
      showShortcutsDialog: ref(false),
      showQuickPanel: ref(false),
    });

    expect(state.optionSourceHint.value).toContain('NoName选项偏置');
    expect(state.optionSourceHint.value).toContain('山门危机');
  });
});
