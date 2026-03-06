export type UiTheme = 'theme-scroll' | 'theme-night';

const UI_THEME_STORAGE_KEY = 'nobody_ui_theme';
const DEFAULT_UI_THEME: UiTheme = 'theme-scroll';
const UI_THEME_CHANGE_EVENT = 'nobody:ui-theme-changed';

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
  window.dispatchEvent(
    new CustomEvent<UiTheme>(UI_THEME_CHANGE_EVENT, { detail: theme }),
  );
};

export const addUiThemeListener = (listener: (theme: UiTheme) => void): (() => void) => {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const handleCustomChange = (event: Event) => {
    const detail = (event as CustomEvent<unknown>).detail;
    if (detail === 'theme-scroll' || detail === 'theme-night') {
      listener(detail);
    }
  };

  const handleStorageChange = (event: StorageEvent) => {
    if (event.key !== UI_THEME_STORAGE_KEY || !event.newValue) {
      return;
    }
    if (event.newValue === 'theme-scroll' || event.newValue === 'theme-night') {
      listener(event.newValue);
    }
  };

  window.addEventListener(UI_THEME_CHANGE_EVENT, handleCustomChange as EventListener);
  window.addEventListener('storage', handleStorageChange);

  return () => {
    window.removeEventListener(UI_THEME_CHANGE_EVENT, handleCustomChange as EventListener);
    window.removeEventListener('storage', handleStorageChange);
  };
};
