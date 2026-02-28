import { describe, expect, it } from 'vitest';
import { getUiTheme, saveUiTheme } from './uiTheme';

describe('uiTheme', () => {
  it('returns default theme when storage is empty', () => {
    window.localStorage.removeItem('nobody_ui_theme');
    expect(getUiTheme()).toBe('theme-scroll');
  });

  it('reads stored theme when valid', () => {
    window.localStorage.setItem('nobody_ui_theme', 'theme-night');
    expect(getUiTheme()).toBe('theme-night');
  });

  it('falls back to default when stored value is invalid', () => {
    window.localStorage.setItem('nobody_ui_theme', 'invalid');
    expect(getUiTheme()).toBe('theme-scroll');
  });

  it('persists selected theme', () => {
    saveUiTheme('theme-night');
    expect(window.localStorage.getItem('nobody_ui_theme')).toBe('theme-night');
  });
});
