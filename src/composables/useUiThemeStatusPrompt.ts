import { getCurrentInstance, onBeforeUnmount, ref } from 'vue';
import type { UiTheme } from '../utils/uiTheme';
import { buildUiThemeStatusText, type UiThemeStatusSource } from '../utils/uiThemeStatus';

const DEFAULT_AUTO_HIDE_MS = 2600;

export const useUiThemeStatusPrompt = (autoHideMs = DEFAULT_AUTO_HIDE_MS) => {
  const themeStatusTitle = ref('');
  const themeStatusMessage = ref('');
  const themeStatusVisible = ref(false);
  let hideTimer: number | null = null;

  const clearUiThemeStatus = () => {
    themeStatusVisible.value = false;
    themeStatusTitle.value = '';
    themeStatusMessage.value = '';
    if (hideTimer != null) {
      window.clearTimeout(hideTimer);
      hideTimer = null;
    }
  };

  const showUiThemeStatus = (theme: UiTheme, source: UiThemeStatusSource = 'manual') => {
    const status = buildUiThemeStatusText(theme, source);
    themeStatusTitle.value = status.title;
    themeStatusMessage.value = status.message;
    themeStatusVisible.value = true;
    if (hideTimer != null) {
      window.clearTimeout(hideTimer);
    }
    hideTimer = window.setTimeout(() => {
      hideTimer = null;
      themeStatusVisible.value = false;
      themeStatusTitle.value = '';
      themeStatusMessage.value = '';
    }, autoHideMs);
  };

  if (getCurrentInstance()) {
    onBeforeUnmount(() => {
      clearUiThemeStatus();
    });
  }

  return {
    themeStatusTitle,
    themeStatusMessage,
    themeStatusVisible,
    showUiThemeStatus,
    clearUiThemeStatus,
  };
};
