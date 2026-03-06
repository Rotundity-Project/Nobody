import type { UiTheme } from './uiTheme';

export type UiThemeStatusSource = 'manual' | 'sync';

export const getUiThemeDisplayLabel = (theme: UiTheme): string =>
  (theme === 'theme-night' ? '深色风格' : '浅色古风');

export const buildUiThemeStatusText = (
  theme: UiTheme,
  source: UiThemeStatusSource = 'manual',
): { title: string; message: string } => {
  const label = getUiThemeDisplayLabel(theme);
  return {
    title: '界面主题已切换',
    message: source === 'sync' ? `当前为${label}（已自动同步）` : `当前为${label}`,
  };
};
