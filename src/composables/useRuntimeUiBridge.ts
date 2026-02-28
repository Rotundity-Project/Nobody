import { computed, type ComputedRef, type Ref } from 'vue';
import type { RuntimeQuickTab } from '../components/RuntimeQuickPanelsDialog.vue';
import type { ConsistencyPolicy } from '../types/game';
import type { StorySettings } from '../utils/storySettings';
import type { UiTheme } from '../utils/uiTheme';

export const useRuntimeUiBridge = ({
  isGameInitialized,
  activeThemeLabel,
  openCharacterDialog,
  openInfoDialog,
  openQuickPanel,
  openSaveDialog,
  openLoadDialog,
  toggleTheme,
  openStorySettingsDialog,
  showSaveDialog,
  showLoadDialog,
  showShortcutsDialog,
  showLLMDialog,
  showStorySettings,
  showConsistencySettings,
  storySettings,
  activeTheme,
  consistencyPolicy,
  handleSaved,
  handleLoaded,
  applyStorySettings,
  updateUiThemeFromSettings,
  applyConsistencyPolicy,
  resetConsistencyPolicy,
  showQuickPanel,
  activeQuickPanelTab,
  quickPanels,
}: {
  isGameInitialized: ComputedRef<boolean>;
  activeThemeLabel: ComputedRef<string>;
  openCharacterDialog: () => Promise<void>;
  openInfoDialog: () => Promise<void>;
  openQuickPanel: (tab: RuntimeQuickTab) => Promise<void>;
  openSaveDialog: () => void;
  openLoadDialog: () => void;
  toggleTheme: () => void;
  openStorySettingsDialog: () => void;
  showSaveDialog: Ref<boolean>;
  showLoadDialog: Ref<boolean>;
  showShortcutsDialog: Ref<boolean>;
  showLLMDialog: Ref<boolean>;
  showStorySettings: Ref<boolean>;
  showConsistencySettings: Ref<boolean>;
  storySettings: Ref<StorySettings>;
  activeTheme: Ref<UiTheme>;
  consistencyPolicy: Ref<ConsistencyPolicy>;
  handleSaved: (slotId: number) => void;
  handleLoaded: (slotId: number) => void;
  applyStorySettings: (settings: StorySettings) => Promise<void>;
  updateUiThemeFromSettings: (theme: UiTheme) => void;
  applyConsistencyPolicy: (policy: ConsistencyPolicy) => Promise<void>;
  resetConsistencyPolicy: () => void;
  showQuickPanel: Ref<boolean>;
  activeQuickPanelTab: Ref<RuntimeQuickTab>;
  quickPanels: ComputedRef<Array<{
    id: RuntimeQuickTab;
    label: string;
    title: string;
    subtitle?: string;
    emptyText?: string;
    items: Array<{
      id: string;
      title: string;
      description?: string;
      meta?: string;
      badge?: string;
      featured?: boolean;
    }>;
  }>>;
}) => {
  const bottomBarProps = computed(() => ({
    isGameInitialized: isGameInitialized.value,
    activeThemeLabel: activeThemeLabel.value,
  }));
  const bottomBarListeners = {
    'open-character': openCharacterDialog,
    'open-info': openInfoDialog,
    'open-backpack': () => openQuickPanel('backpack'),
    'open-techniques': () => openQuickPanel('techniques'),
    'open-factions': () => openQuickPanel('factions'),
    'open-world': () => openQuickPanel('world'),
    'open-save': openSaveDialog,
    'open-load': openLoadDialog,
    'toggle-theme': toggleTheme,
    'open-settings': openStorySettingsDialog,
  };

  const systemDialogsProps = computed(() => ({
    showSaveDialog: showSaveDialog.value,
    showLoadDialog: showLoadDialog.value,
    showShortcutsDialog: showShortcutsDialog.value,
    showLLMDialog: showLLMDialog.value,
    showStorySettings: showStorySettings.value,
    showConsistencySettings: showConsistencySettings.value,
    storySettings: storySettings.value,
    uiTheme: activeTheme.value,
    consistencyPolicy: consistencyPolicy.value,
  }));
  const systemDialogsListeners = {
    'close-save': () => {
      showSaveDialog.value = false;
    },
    saved: handleSaved,
    'close-load': () => {
      showLoadDialog.value = false;
    },
    loaded: handleLoaded,
    'close-shortcuts': () => {
      showShortcutsDialog.value = false;
    },
    'close-llm': () => {
      showLLMDialog.value = false;
    },
    'close-story-settings': () => {
      showStorySettings.value = false;
    },
    'save-story-settings': applyStorySettings,
    'update-ui-theme': updateUiThemeFromSettings,
    'close-consistency': () => {
      showConsistencySettings.value = false;
    },
    'save-consistency': applyConsistencyPolicy,
    'reset-consistency': resetConsistencyPolicy,
  };

  const quickPanelsDialogProps = computed(() => ({
    isOpen: showQuickPanel.value,
    activeTab: activeQuickPanelTab.value,
    panels: quickPanels.value,
  }));
  const quickPanelsDialogListeners = {
    'update:active-tab': (value: RuntimeQuickTab) => {
      activeQuickPanelTab.value = value;
    },
    close: () => {
      showQuickPanel.value = false;
    },
  };

  return {
    bottomBarProps,
    bottomBarListeners,
    systemDialogsProps,
    systemDialogsListeners,
    quickPanelsDialogProps,
    quickPanelsDialogListeners,
  };
};
