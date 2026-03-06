import { ref, type Ref } from 'vue';
import type { PlayerAction, PlayerOption } from '../types/game';
import { buildStoryLoadingPhases } from '../utils/loadingPhases';

const MAX_AUTO_ADVANCE_STEPS = 48;

type PlotStateLike = {
  current_chapter?: {
    index?: number;
    content?: string[];
  } | null;
  segment_count?: number;
  last_option_generation_source?: string | null;
};

type StoryFlowStore = {
  executePlayerAction: (action: PlayerAction) => Promise<void>;
  plotState?: PlotStateLike | null;
  worldRegistry?: {
    llm_model?: string | null;
  } | null;
  isWaitingForInput: boolean;
  availableOptions: PlayerOption[];
};

type StoryFlowDeps = {
  gameStore: StoryFlowStore;
  shouldAutoAdvance: Ref<boolean>;
  freeTextInput: Ref<string>;
  validateFreeTextInput: (text: string) => { valid: boolean };
  createOptionAction: (option: PlayerOption) => PlayerAction;
  createFreeTextAction: (text: string) => PlayerAction;
  createContinueAction: () => PlayerAction;
  playClick: () => void;
};

export const useStoryFlow = ({
  gameStore,
  shouldAutoAdvance,
  freeTextInput,
  validateFreeTextInput,
  createOptionAction,
  createFreeTextAction,
  createContinueAction,
  playClick,
}: StoryFlowDeps) => {
  const isLoading = ref(false);
  const loadingMessage = ref('处理中...');
  const loadingProgress = ref<number | null>(null);
  const loadingProgressText = ref('');
  const loadingStage = ref('');
  const loadingElapsedMs = ref<number | null>(null);
  const autoAdvanceRunning = ref(false);
  const autoAdvanceStopRequested = ref(false);
  const autoAdvanceStopHint = ref('');
  let loadingPhaseTimer: ReturnType<typeof setInterval> | null = null;
  let loadingElapsedTimer: ReturnType<typeof setInterval> | null = null;
  let loadingStartedAt = 0;

  const stopLoadingPhaseTicker = () => {
    if (loadingPhaseTimer) {
      clearInterval(loadingPhaseTimer);
      loadingPhaseTimer = null;
    }
  };

  const stopLoadingElapsedTicker = () => {
    if (loadingElapsedTimer) {
      clearInterval(loadingElapsedTimer);
      loadingElapsedTimer = null;
    }
  };

  const startLoadingPhaseTicker = () => {
    stopLoadingPhaseTicker();
    stopLoadingElapsedTicker();
    loadingStartedAt = Date.now();
    loadingElapsedMs.value = 0;
    const loadingPhases = buildStoryLoadingPhases(gameStore.worldRegistry?.llm_model);
    let phaseIndex = 0;
    loadingStage.value = loadingPhases[phaseIndex].stage;
    loadingProgress.value = loadingPhases[phaseIndex].progress;
    loadingProgressText.value = loadingPhases[phaseIndex].text;
    loadingElapsedTimer = setInterval(() => {
      loadingElapsedMs.value = Date.now() - loadingStartedAt;
    }, 200);
    loadingPhaseTimer = setInterval(() => {
      phaseIndex = Math.min(phaseIndex + 1, loadingPhases.length - 1);
      loadingStage.value = loadingPhases[phaseIndex].stage;
      loadingProgress.value = loadingPhases[phaseIndex].progress;
      loadingProgressText.value = loadingPhases[phaseIndex].text;
    }, 1300);
  };

  const resetLoadingState = () => {
    stopLoadingPhaseTicker();
    stopLoadingElapsedTicker();
    loadingMessage.value = '处理中...';
    loadingStage.value = '';
    loadingProgress.value = null;
    loadingProgressText.value = '';
    loadingElapsedMs.value = null;
  };
  const safePlayClick = () => {
    try {
      playClick();
    } catch (error) {
      console.warn('播放点击音效失败，已忽略：', error);
    }
  };

  const handleOptionSelect = async (option: PlayerOption) => {
    try {
      autoAdvanceStopHint.value = '';
      isLoading.value = true;
      loadingMessage.value = '正在执行选项...';
      startLoadingPhaseTicker();
      safePlayClick();
      await gameStore.executePlayerAction(createOptionAction(option));
      loadingStage.value = '状态落盘';
      loadingProgress.value = 100;
      loadingProgressText.value = '已完成，正在刷新界面...';
      loadingElapsedMs.value = Date.now() - loadingStartedAt;
    } catch (error) {
      console.error('执行行动失败：', error);
    } finally {
      isLoading.value = false;
      resetLoadingState();
    }
  };

  const handleFreeTextSubmit = async () => {
    const check = validateFreeTextInput(freeTextInput.value);
    if (!check.valid) {
      return;
    }

    try {
      autoAdvanceStopHint.value = '';
      isLoading.value = true;
      loadingMessage.value = '正在解析输入...';
      startLoadingPhaseTicker();
      safePlayClick();
      await gameStore.executePlayerAction(createFreeTextAction(freeTextInput.value));
      freeTextInput.value = '';
      loadingStage.value = '状态落盘';
      loadingProgress.value = 100;
      loadingProgressText.value = '已完成，正在刷新界面...';
      loadingElapsedMs.value = Date.now() - loadingStartedAt;
    } catch (error) {
      console.error('提交自由输入失败：', error);
    } finally {
      isLoading.value = false;
      resetLoadingState();
    }
  };

  const handleContinue = async () => {
    try {
      autoAdvanceStopHint.value = '';
      autoAdvanceStopRequested.value = false;
      isLoading.value = true;
      loadingMessage.value = '正在续写剧情...';
      safePlayClick();
      autoAdvanceRunning.value = true;
      let step = 0;
      let stoppedByStagnation = false;
      let stoppedByUser = false;
      let previousSignature = [
        gameStore.plotState?.current_chapter?.index ?? -1,
        gameStore.plotState?.current_chapter?.content?.length ?? 0,
        gameStore.plotState?.segment_count ?? 0,
        gameStore.plotState?.last_option_generation_source ?? '',
        gameStore.isWaitingForInput ? 'w' : 'nw',
        gameStore.availableOptions.length,
      ].join('|');
      do {
        if (autoAdvanceStopRequested.value && step > 0) {
          stoppedByUser = true;
          break;
        }
        step += 1;
        if (step > 1) {
          loadingMessage.value = `正在自动推进剧情（${step}）...`;
        }
        startLoadingPhaseTicker();
        await gameStore.executePlayerAction(createContinueAction());
        loadingStage.value = '状态落盘';
        loadingProgress.value = 100;
        loadingProgressText.value = '单步完成，继续评估中...';
        loadingElapsedMs.value = Date.now() - loadingStartedAt;
        stopLoadingPhaseTicker();
        stopLoadingElapsedTicker();

        const currentSignature = [
          gameStore.plotState?.current_chapter?.index ?? -1,
          gameStore.plotState?.current_chapter?.content?.length ?? 0,
          gameStore.plotState?.segment_count ?? 0,
          gameStore.plotState?.last_option_generation_source ?? '',
          gameStore.isWaitingForInput ? 'w' : 'nw',
          gameStore.availableOptions.length,
        ].join('|');

        const stagnated = currentSignature === previousSignature;
        previousSignature = currentSignature;
        if (stagnated) {
          stoppedByStagnation = true;
          break;
        }
      } while (shouldAutoAdvance.value && step < MAX_AUTO_ADVANCE_STEPS);

      if (shouldAutoAdvance.value && !stoppedByStagnation && step >= MAX_AUTO_ADVANCE_STEPS) {
        console.warn(`自动推进达到保护上限（${MAX_AUTO_ADVANCE_STEPS} 步），已停止以避免卡死。`);
      }
      if (stoppedByUser) {
        autoAdvanceStopHint.value = '自动推进已中断，可点击继续恢复。';
      } else if (stoppedByStagnation) {
        autoAdvanceStopHint.value = '自动推进已暂停：检测到剧情未继续变化。';
      } else if (shouldAutoAdvance.value && step >= MAX_AUTO_ADVANCE_STEPS) {
        autoAdvanceStopHint.value = '自动推进达到保护上限，已自动停止。';
      }
    } catch (error) {
      console.error('继续写失败：', error);
    } finally {
      autoAdvanceRunning.value = false;
      isLoading.value = false;
      resetLoadingState();
    }
  };

  const requestStopAutoAdvance = () => {
    if (autoAdvanceRunning.value) {
      autoAdvanceStopRequested.value = true;
    }
  };

  return {
    isLoading,
    loadingMessage,
    loadingStage,
    loadingProgress,
    loadingProgressText,
    loadingElapsedMs,
    autoAdvanceRunning,
    autoAdvanceStopHint,
    handleOptionSelect,
    handleFreeTextSubmit,
    handleContinue,
    requestStopAutoAdvance,
  };
};
