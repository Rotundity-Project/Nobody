import { defineComponent, ref } from 'vue';
import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { useRuntimeLifecycleRecovery } from './useRuntimeLifecycleRecovery';

const mountHarness = (params: Parameters<typeof useRuntimeLifecycleRecovery>[0]) => {
  const Harness = defineComponent({
    setup() {
      useRuntimeLifecycleRecovery(params);
      return () => null;
    },
  });
  return mount(Harness);
};

describe('useRuntimeLifecycleRecovery', () => {
  it('tries to recover from reserved snapshot slot on init', async () => {
    window.localStorage.setItem(
      'nobody_runtime_lifecycle_snapshot_v1',
      JSON.stringify({
        timestamp: Date.now(),
        hadActiveSession: true,
        inputMode: 'freeText',
        freeTextInput: '继续推进',
        chapterTitle: '第一章',
        chapterIndex: 1,
        segmentCount: 3,
      }),
    );

    const store = {
      isGameInitialized: false,
      isPlotInitialized: false,
      isLoading: false,
      saveGame: vi.fn(async () => undefined),
      loadGame: vi.fn(async () => undefined),
    };
    const inputMode = ref<'options' | 'freeText'>('options');
    const freeTextInput = ref('');
    const setInputMode = vi.fn((mode: 'options' | 'freeText') => {
      inputMode.value = mode;
    });
    const logRuntimeAction = vi.fn();
    const notifyRuntimeError = vi.fn();

    mountHarness({
      gameStore: store,
      inputMode,
      freeTextInput,
      setInputMode,
      currentChapterTitle: ref('第一章'),
      currentChapterIndex: ref(1),
      currentSegmentCount: ref(3),
      logRuntimeAction,
      notifyRuntimeError,
    });

    await Promise.resolve();
    await Promise.resolve();

    expect(store.loadGame).toHaveBeenCalledWith(99);
    expect(setInputMode).toHaveBeenCalledWith('freeText');
    expect(freeTextInput.value).toBe('继续推进');
    expect(logRuntimeAction).toHaveBeenCalled();
    expect(notifyRuntimeError).not.toHaveBeenCalled();
  });

  it('persists snapshot and writes reserved recovery slot when hidden', async () => {
    window.localStorage.removeItem('nobody_runtime_lifecycle_snapshot_v1');

    const store = {
      isGameInitialized: true,
      isPlotInitialized: true,
      isLoading: false,
      saveGame: vi.fn(async () => undefined),
      loadGame: vi.fn(async () => undefined),
    };
    const logRuntimeAction = vi.fn();
    const notifyRuntimeError = vi.fn();

    mountHarness({
      gameStore: store,
      inputMode: ref<'options' | 'freeText'>('options'),
      freeTextInput: ref(''),
      setInputMode: vi.fn(),
      currentChapterTitle: ref('第一章'),
      currentChapterIndex: ref(1),
      currentSegmentCount: ref(2),
      logRuntimeAction,
      notifyRuntimeError,
    });

    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      value: 'hidden',
    });
    document.dispatchEvent(new Event('visibilitychange'));
    await Promise.resolve();
    await Promise.resolve();

    const raw = window.localStorage.getItem('nobody_runtime_lifecycle_snapshot_v1');
    expect(raw).toBeTruthy();
    expect(store.saveGame).toHaveBeenCalledWith(99);
    expect(notifyRuntimeError).not.toHaveBeenCalled();
  });

  it('recovers when runtime is partially initialized (game initialized but plot missing)', async () => {
    window.localStorage.setItem(
      'nobody_runtime_lifecycle_snapshot_v1',
      JSON.stringify({
        timestamp: Date.now(),
        hadActiveSession: true,
        inputMode: 'options',
        freeTextInput: '',
        chapterTitle: '第一章',
        chapterIndex: 1,
        segmentCount: 2,
      }),
    );

    const store = {
      isGameInitialized: true,
      isPlotInitialized: false,
      isLoading: false,
      saveGame: vi.fn(async () => undefined),
      loadGame: vi.fn(async () => undefined),
    };

    mountHarness({
      gameStore: store,
      inputMode: ref<'options' | 'freeText'>('options'),
      freeTextInput: ref(''),
      setInputMode: vi.fn(),
      currentChapterTitle: ref('第一章'),
      currentChapterIndex: ref(1),
      currentSegmentCount: ref(2),
      logRuntimeAction: vi.fn(),
      notifyRuntimeError: vi.fn(),
    });

    await Promise.resolve();
    await Promise.resolve();

    expect(store.loadGame).toHaveBeenCalledWith(99);
  });
});
