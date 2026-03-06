import { ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';
import { useRuntimeShellUi } from './useRuntimeShellUi';

const createShellUi = (overrides?: {
  closeAllDialogs?: () => void;
}) => {
  const logRuntimeAction = vi.fn();
  const notifyRuntimeError = vi.fn();

  const ui = useRuntimeShellUi({
    showQuickPanel: ref(false),
    showCharacterInfo: ref(false),
    showInfoTabs: ref(false),
    showSaveDialog: ref(false),
    showLoadDialog: ref(false),
    showLLMDialog: ref(false),
    showStorySettings: ref(false),
    closeAllDialogs: overrides?.closeAllDialogs ?? vi.fn(),
    safePlayClick: vi.fn(),
    logRuntimeAction,
    notifyRuntimeError,
  });

  return {
    ui,
    logRuntimeAction,
    notifyRuntimeError,
  };
};

describe('useRuntimeShellUi', () => {
  it('updates active theme when external theme event is dispatched', () => {
    window.localStorage.setItem('nobody_ui_theme', 'theme-scroll');
    const { ui } = createShellUi();
    expect(ui.activeThemeClass.value).toBe('theme-scroll');

    window.dispatchEvent(new CustomEvent('nobody:ui-theme-changed', { detail: 'theme-night' }));
    expect(ui.activeThemeClass.value).toBe('theme-night');
  });

  it('logs unified theme status when external theme event is dispatched', () => {
    window.localStorage.setItem('nobody_ui_theme', 'theme-scroll');
    const { logRuntimeAction } = createShellUi();
    logRuntimeAction.mockClear();

    window.dispatchEvent(new CustomEvent('nobody:ui-theme-changed', { detail: 'theme-night' }));

    expect(logRuntimeAction).toHaveBeenCalledWith(
      '界面主题已切换',
      '当前为深色风格（已自动同步）',
    );
  });

  it('logs readable label when opening quick panel', async () => {
    const { ui, logRuntimeAction } = createShellUi();

    await ui.openQuickPanel('world');

    expect(logRuntimeAction).toHaveBeenCalledWith('已打开世界面板', undefined);
  });

  it('reports runtime error when action fails', async () => {
    const boom = new Error('boom');
    const { ui, notifyRuntimeError, logRuntimeAction } = createShellUi({
      closeAllDialogs: () => {
        throw boom;
      },
    });

    ui.openSaveDialog();
    await Promise.resolve();

    expect(notifyRuntimeError).toHaveBeenCalledWith('已打开存档面板', boom);
    expect(logRuntimeAction).not.toHaveBeenCalled();
  });
});
