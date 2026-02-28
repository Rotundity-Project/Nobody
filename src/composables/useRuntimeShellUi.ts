import { computed, nextTick, ref, watch, type Ref } from 'vue';
import type { RuntimeQuickTab } from '../components/RuntimeQuickPanelsDialog.vue';
import { getUiTheme, saveUiTheme, type UiTheme } from '../utils/uiTheme';

const QUICK_PANEL_TAB_STORAGE_KEY = 'runtime.quick_panel.active_tab';
const ALLOWED_QUICK_TABS: RuntimeQuickTab[] = ['backpack', 'techniques', 'factions', 'world'];

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
}) => {
  const activeQuickPanelTab = ref<RuntimeQuickTab>(loadSavedQuickPanelTab());
  const activeTheme = ref<UiTheme>(getUiTheme());

  const closeRuntimeQuickPanels = () => {
    showQuickPanel.value = false;
  };

  const openCharacterDialog = async () => {
    safePlayClick();
    closeAllDialogs();
    closeRuntimeQuickPanels();
    showCharacterInfo.value = false;
    await nextTick();
    showCharacterInfo.value = true;
    logRuntimeAction('已打开人物面板');
  };

  const openInfoDialog = async () => {
    safePlayClick();
    closeAllDialogs();
    closeRuntimeQuickPanels();
    showInfoTabs.value = false;
    await nextTick();
    showInfoTabs.value = true;
    logRuntimeAction('已打开行旅信息');
  };

  const openSaveDialog = () => {
    safePlayClick();
    closeAllDialogs();
    closeRuntimeQuickPanels();
    showSaveDialog.value = true;
    logRuntimeAction('已打开存档面板');
  };

  const openLoadDialog = () => {
    safePlayClick();
    closeAllDialogs();
    closeRuntimeQuickPanels();
    showLoadDialog.value = true;
    logRuntimeAction('已打开读档面板');
  };

  const openLlmDialogFromError = () => {
    safePlayClick();
    closeAllDialogs();
    closeRuntimeQuickPanels();
    showLLMDialog.value = true;
    logRuntimeAction('已打开 LLM 设置');
  };

  const openQuickPanel = async (tab: RuntimeQuickTab) => {
    safePlayClick();
    closeAllDialogs();
    showQuickPanel.value = false;
    activeQuickPanelTab.value = tab;
    await nextTick();
    showQuickPanel.value = true;
    const panelLabels: Record<RuntimeQuickTab, string> = {
      backpack: '背包',
      techniques: '功法',
      factions: '势力',
      world: '世界',
    };
    logRuntimeAction(`已打开${panelLabels[tab]}面板`);
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
    activeTheme.value = theme;
    saveUiTheme(theme);
  };

  const toggleTheme = () => {
    safePlayClick();
    const nextTheme: UiTheme = activeTheme.value === 'theme-night' ? 'theme-scroll' : 'theme-night';
    applyUiTheme(nextTheme);
    logRuntimeAction('已切换主题', `当前为${activeThemeLabel.value}主题`);
  };

  const updateUiThemeFromSettings = (theme: UiTheme) => {
    if (theme === activeTheme.value) {
      return;
    }
    applyUiTheme(theme);
    logRuntimeAction('已切换主题', `当前为${activeThemeLabel.value}主题`);
  };

  const openStorySettingsDialog = () => {
    safePlayClick();
    showStorySettings.value = true;
    logRuntimeAction('已打开系统设置');
  };

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
