import { onBeforeUnmount, onMounted, type Ref } from 'vue';
import type { InputMode } from './useInputMode';
import { isTauriRuntime } from '../platform/runtimeEnv';

type GameStoreLike = {
  isGameInitialized: boolean;
  isPlotInitialized: boolean;
  isLoading: boolean;
  saveGame: (slotId: number) => Promise<void>;
  loadGame: (slotId: number) => Promise<unknown>;
};

type RuntimeSnapshot = {
  timestamp: number;
  hadActiveSession: boolean;
  inputMode: InputMode;
  freeTextInput: string;
  chapterTitle: string;
  chapterIndex: number;
  segmentCount: number;
};

const SNAPSHOT_STORAGE_KEY = 'nobody_runtime_lifecycle_snapshot_v1';
const RECOVERY_SLOT_ID = 99;
const SNAPSHOT_PERSIST_MIN_INTERVAL_MS = 15_000;

const safeNow = () => Date.now();

const readSnapshot = (): RuntimeSnapshot | null => {
  try {
    const raw = window.localStorage.getItem(SNAPSHOT_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<RuntimeSnapshot>;
    return {
      timestamp: Number(parsed.timestamp ?? 0),
      hadActiveSession: Boolean(parsed.hadActiveSession),
      inputMode: parsed.inputMode === 'freeText' ? 'freeText' : 'options',
      freeTextInput: String(parsed.freeTextInput ?? ''),
      chapterTitle: String(parsed.chapterTitle ?? ''),
      chapterIndex: Number(parsed.chapterIndex ?? 0),
      segmentCount: Number(parsed.segmentCount ?? 0),
    };
  } catch {
    return null;
  }
};

const writeSnapshot = (snapshot: RuntimeSnapshot) => {
  try {
    window.localStorage.setItem(SNAPSHOT_STORAGE_KEY, JSON.stringify(snapshot));
  } catch {
    // Ignore storage failures in restricted contexts.
  }
};

export const useRuntimeLifecycleRecovery = ({
  gameStore,
  inputMode,
  freeTextInput,
  setInputMode,
  currentChapterTitle,
  currentChapterIndex,
  currentSegmentCount,
  logRuntimeAction,
  notifyRuntimeError,
}: {
  gameStore: GameStoreLike;
  inputMode: Ref<InputMode>;
  freeTextInput: Ref<string>;
  setInputMode: (mode: InputMode) => void;
  currentChapterTitle: Ref<string>;
  currentChapterIndex: Ref<number>;
  currentSegmentCount: Ref<number>;
  logRuntimeAction: (title: string, detail?: string) => void;
  notifyRuntimeError: (scope: string, error: unknown) => void;
}) => {
  let lastPersistAt = 0;
  let recoveryAttempted = false;
  const tauriUnlistenFns: Array<() => void | Promise<void>> = [];

  const restoreUiDraft = (snapshot: RuntimeSnapshot | null) => {
    if (!snapshot) return;
    if (snapshot.freeTextInput.trim().length > 0) {
      setInputMode(snapshot.inputMode);
      freeTextInput.value = snapshot.freeTextInput;
    }
  };

  const persistLifecycleSnapshot = async (trigger: string) => {
    const now = safeNow();
    if (now - lastPersistAt < SNAPSHOT_PERSIST_MIN_INTERVAL_MS) {
      return;
    }
    lastPersistAt = now;

    writeSnapshot({
      timestamp: now,
      hadActiveSession: gameStore.isGameInitialized,
      inputMode: inputMode.value,
      freeTextInput: freeTextInput.value,
      chapterTitle: currentChapterTitle.value,
      chapterIndex: currentChapterIndex.value,
      segmentCount: currentSegmentCount.value,
    });

    if (!gameStore.isGameInitialized || !gameStore.isPlotInitialized || gameStore.isLoading) {
      return;
    }

    try {
      // Reserve slot 99 as automatic recovery checkpoint.
      await gameStore.saveGame(RECOVERY_SLOT_ID);
      void trigger;
    } catch (error) {
      notifyRuntimeError('写入恢复快照', error);
    }
  };

  const tryRecoverOnResume = async (source: 'init' | 'visible' | 'focus') => {
    if (gameStore.isLoading || recoveryAttempted) {
      return;
    }
    const hasCompleteRuntimeState = gameStore.isGameInitialized && gameStore.isPlotInitialized;
    if (hasCompleteRuntimeState) {
      return;
    }
    const snapshot = readSnapshot();
    if (!snapshot?.hadActiveSession) {
      return;
    }
    recoveryAttempted = true;
    restoreUiDraft(snapshot);
    try {
      await gameStore.loadGame(RECOVERY_SLOT_ID);
      logRuntimeAction(
        '已恢复到最近安全快照',
        `来源: ${source} | 章节: ${snapshot.chapterIndex}.${snapshot.chapterTitle || '未命名章节'}`,
      );
    } catch (error) {
      notifyRuntimeError('恢复最近安全快照', error);
    }
  };

  const onVisibilityChange = () => {
    if (document.visibilityState === 'hidden') {
      void persistLifecycleSnapshot('hidden');
      return;
    }
    void tryRecoverOnResume('visible');
  };

  const onFocus = () => {
    void tryRecoverOnResume('focus');
  };

  const onPageHide = () => {
    void persistLifecycleSnapshot('pagehide');
  };

  const setupTauriWindowLifecycle = async () => {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const appWindow = getCurrentWindow();

      const unlistenFocusChanged = await appWindow.onFocusChanged(async ({ payload: focused }) => {
        if (focused) {
          await tryRecoverOnResume('focus');
        } else {
          await persistLifecycleSnapshot('tauri-blur');
        }
      });
      tauriUnlistenFns.push(unlistenFocusChanged);

      const unlistenCloseRequested = await appWindow.onCloseRequested(async () => {
        await persistLifecycleSnapshot('tauri-close');
      });
      tauriUnlistenFns.push(unlistenCloseRequested);

      const unlistenResized = await appWindow.onResized(async () => {
        try {
          const minimized = await appWindow.isMinimized();
          if (minimized) {
            await persistLifecycleSnapshot('tauri-minimized');
          }
        } catch {
          // best-effort check
        }
      });
      tauriUnlistenFns.push(unlistenResized);
    } catch (error) {
      notifyRuntimeError('注册窗口生命周期监听', error);
    }
  };

  onMounted(() => {
    document.addEventListener('visibilitychange', onVisibilityChange);
    window.addEventListener('focus', onFocus);
    window.addEventListener('pagehide', onPageHide);
    void setupTauriWindowLifecycle();
    void tryRecoverOnResume('init');
  });

  onBeforeUnmount(() => {
    document.removeEventListener('visibilitychange', onVisibilityChange);
    window.removeEventListener('focus', onFocus);
    window.removeEventListener('pagehide', onPageHide);
    while (tauriUnlistenFns.length > 0) {
      const fn = tauriUnlistenFns.pop();
      try {
        void fn?.();
      } catch {
        // ignore unlisten failures during teardown
      }
    }
  });

  return {
    persistLifecycleSnapshot,
    tryRecoverOnResume,
  };
};
