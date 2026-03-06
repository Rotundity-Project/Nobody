import { computed, getCurrentInstance, nextTick, onBeforeUnmount, ref, watch, type Ref } from 'vue';
import type { RuntimeQuickTab } from '../components/RuntimeQuickPanelsDialog.vue';
import { addUiThemeListener, getUiTheme, saveUiTheme, type UiTheme } from '../utils/uiTheme';
import { buildUiThemeStatusText } from '../utils/uiThemeStatus';

const QUICK_PANEL_TAB_STORAGE_KEY = 'runtime.quick_panel.active_tab';
const ALLOWED_QUICK_TABS: RuntimeQuickTab[] = ['backpack', 'techniques', 'factions', 'world'];

const QUICK_PANEL_LABELS: Record<RuntimeQuickTab, string> = {
  backpack: '背包',
  techniques: '功法',
  factions: '势力',
  world: '世界',
};

const loadSavedQuickPanelTab = (): RuntimeQuickTab => {
  try {
    const raw = localStorage.getItem(QUICK_PANEL_TAB_STORAGE_KEY) ?? '';
    if (ALLOWED_QUICK_TABS.includes(raw as RuntimeQuickTab)) {
      return raw as RuntimeQuickTab;
    }
  } catch {
    // Ignore storage errors in restricted contexts.
  }
  return 'backpack';
};

export const useRuntimeShellUi = ({
  showQuickPanel,
  showCharacterInfo,
  showInfoTabs,
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showStorySettings,
  closeAllDialogs,
  safePlayClick,
  logRuntimeAction,
  notifyRuntimeError,
}: {
  showQuickPanel: Ref<boolean>;
  showCharacterInfo: Ref<boolean>;
  showInfoTabs: Ref<boolean>;
  showSaveDialog: Ref<boolean>;
  showLoadDialog: Ref<boolean>;
  showLLMDialog: Ref<boolean>;
  showStorySettings: Ref<boolean>;
  closeAllDialogs: () => void;
  safePlayClick: () => void;
  logRuntimeAction: (title: string, detail?: string) => void;
  notifyRuntimeError: (scope: string, error: unknown) => void;
}) => {
  const activeQuickPanelTab = ref<RuntimeQuickTab>(loadSavedQuickPanelTab());
  const activeTheme = ref<UiTheme>(getUiTheme());
  let pendingThemeSync: UiTheme | null = null;
  const removeUiThemeListener = addUiThemeListener((theme) => {
    const isLocalUpdate = pendingThemeSync === theme;
    pendingThemeSync = null;
    activeTheme.value = theme;
    if (!isLocalUpdate) {
      const status = buildUiThemeStatusText(theme, 'sync');
      logRuntimeAction(status.title, status.message);
    }
  });

  const runUiAction = async (
    scope: string,
    action: () => void | Promise<void>,
    detail?: string,
  ) => {
    try {
      await action();
      logRuntimeAction(scope, detail);
    } catch (error) {
      notifyRuntimeError(scope, error);
    }
  };

  const closeRuntimeQuickPanels = () => {
    showQuickPanel.value = false;
  };

  const openCharacterDialog = async () => {
    await runUiAction('已打开人物面板', async () => {
      safePlayClick();
      closeAllDialogs();
      closeRuntimeQuickPanels();
      showCharacterInfo.value = false;
      await nextTick();
      showCharacterInfo.value = true;
    });
  };

  const openInfoDialog = async () => {
    await runUiAction('已打开行旅信息', async () => {
      safePlayClick();
      closeAllDialogs();
      closeRuntimeQuickPanels();
      showInfoTabs.value = false;
      await nextTick();
      showInfoTabs.value = true;
    });
  };

  const openSaveDialog = () => {
    void runUiAction('已打开存档面板', () => {
      safePlayClick();
      closeAllDialogs();
      closeRuntimeQuickPanels();
      showSaveDialog.value = true;
    });
  };

  const openLoadDialog = () => {
    void runUiAction('已打开读档面板', () => {
      safePlayClick();
      closeAllDialogs();
      closeRuntimeQuickPanels();
      showLoadDialog.value = true;
    });
  };

  const openLlmDialogFromError = () => {
    void runUiAction('已打开 LLM 设置', () => {
      safePlayClick();
      closeAllDialogs();
      closeRuntimeQuickPanels();
      showLLMDialog.value = true;
    });
  };

  const openQuickPanel = async (tab: RuntimeQuickTab) => {
    await runUiAction(`已打开${QUICK_PANEL_LABELS[tab]}面板`, async () => {
      safePlayClick();
      closeAllDialogs();
      showQuickPanel.value = false;
      activeQuickPanelTab.value = tab;
      await nextTick();
      showQuickPanel.value = true;
    });
  };

  watch(activeQuickPanelTab, (tab) => {
    try {
      localStorage.setItem(QUICK_PANEL_TAB_STORAGE_KEY, tab);
    } catch {
      // Ignore storage errors in restricted contexts.
    }
  });

  const activeThemeClass = computed(() => activeTheme.value);
  const activeThemeLabel = computed(() => (activeTheme.value === 'theme-night' ? '深色' : '浅色'));

  const applyUiTheme = (theme: UiTheme) => {
    pendingThemeSync = theme;
    activeTheme.value = theme;
    saveUiTheme(theme);
  };

  const toggleTheme = () => {
    const nextTheme: UiTheme = activeTheme.value === 'theme-night' ? 'theme-scroll' : 'theme-night';
    const status = buildUiThemeStatusText(nextTheme);
    void runUiAction(status.title, () => {
      safePlayClick();
      applyUiTheme(nextTheme);
    }, status.message);
  };

  const updateUiThemeFromSettings = (theme: UiTheme) => {
    if (theme === activeTheme.value) {
      return;
    }
    const status = buildUiThemeStatusText(theme);
    void runUiAction(status.title, () => {
      applyUiTheme(theme);
    }, status.message);
  };

  const openStorySettingsDialog = () => {
    void runUiAction('已打开系统设置', () => {
      safePlayClick();
      showStorySettings.value = true;
    });
  };

  if (getCurrentInstance()) {
    onBeforeUnmount(() => {
      removeUiThemeListener();
    });
  }

  return {
    activeQuickPanelTab,
    activeTheme,
    closeRuntimeQuickPanels,
    openCharacterDialog,
    openInfoDialog,
    openSaveDialog,
    openLoadDialog,
    openLlmDialogFromError,
    openQuickPanel,
    activeThemeClass,
    activeThemeLabel,
    toggleTheme,
    updateUiThemeFromSettings,
    openStorySettingsDialog,
  };
};
