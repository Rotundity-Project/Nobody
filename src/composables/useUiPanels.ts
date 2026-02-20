import { ref } from 'vue';

export const useUiPanels = () => {
  const showSaveDialog = ref(false);
  const showLoadDialog = ref(false);
  const showLLMDialog = ref(false);
  const showAudioPanel = ref(false);
  const showStorySettings = ref(false);
  const showInfoTabs = ref(false);
  const showConsistencySettings = ref(false);
  const showSystemMenu = ref(false);
  const showCharacterInfo = ref(false);
  const showShortcutsDialog = ref(false);

  const closeSystemMenu = () => {
    showSystemMenu.value = false;
    showAudioPanel.value = false;
  };

  const closeAllDialogs = () => {
    showSaveDialog.value = false;
    showLoadDialog.value = false;
    showLLMDialog.value = false;
    showStorySettings.value = false;
    showConsistencySettings.value = false;
    showSystemMenu.value = false;
    showCharacterInfo.value = false;
    showAudioPanel.value = false;
    showShortcutsDialog.value = false;
    showInfoTabs.value = false;
  };

  return {
    showSaveDialog,
    showLoadDialog,
    showLLMDialog,
    showAudioPanel,
    showStorySettings,
    showInfoTabs,
    showConsistencySettings,
    showSystemMenu,
    showCharacterInfo,
    showShortcutsDialog,
    closeSystemMenu,
    closeAllDialogs,
  };
};