export type UiTheme = 'theme-scroll' | 'theme-night';

const UI_THEME_STORAGE_KEY = 'nobody_ui_theme';
const DEFAULT_UI_THEME: UiTheme = 'theme-scroll';

const isUiTheme = (value: string): value is UiTheme =>
  value === 'theme-scroll' || value === 'theme-night';

export const getUiTheme = (): UiTheme => {
  if (typeof window === 'undefined') {
    return DEFAULT_UI_THEME;
  }
  try {
    const stored = window.localStorage.getItem(UI_THEME_STORAGE_KEY) ?? '';
    return isUiTheme(stored) ? stored : DEFAULT_UI_THEME;
  } catch {
    return DEFAULT_UI_THEME;
  }
};

export const saveUiTheme = (theme: UiTheme): void => {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(UI_THEME_STORAGE_KEY, theme);
};
