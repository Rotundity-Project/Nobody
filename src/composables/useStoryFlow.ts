import { ref, type Ref } from 'vue';
import type { PlayerOption } from '../types/game';

const MAX_AUTO_ADVANCE_STEPS = 48;

type PlotStateLike = {
  current_chapter?: {
    index?: number;
    content?: string[];
  };
  segment_count?: number;
  last_option_generation_source?: string;
};

type StoryFlowStore = {
  executePlayerAction: (action: unknown) => Promise<unknown>;
  plotState?: PlotStateLike | null;
  isWaitingForInput: boolean;
  availableOptions: PlayerOption[];
};

type StoryFlowDeps = {
  gameStore: StoryFlowStore;
  shouldAutoAdvance: Ref<boolean>;
  freeTextInput: Ref<string>;
  validateFreeTextInput: (text: string) => { valid: boolean };
  createOptionAction: (option: PlayerOption) => unknown;
  createFreeTextAction: (text: string) => unknown;
  createContinueAction: () => unknown;
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
  const autoAdvanceRunning = ref(false);
  const autoAdvanceStopRequested = ref(false);
  const autoAdvanceStopHint = ref('');

  const handleOptionSelect = async (option: PlayerOption) => {
    try {
      autoAdvanceStopHint.value = '';
      isLoading.value = true;
      loadingMessage.value = '正在执行选项...';
      playClick();
      await gameStore.executePlayerAction(createOptionAction(option));
    } catch (error) {
      console.error('执行行动失败：', error);
    } finally {
      isLoading.value = false;
      loadingMessage.value = '处理中...';
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
      playClick();
      await gameStore.executePlayerAction(createFreeTextAction(freeTextInput.value));
      freeTextInput.value = '';
    } catch (error) {
      console.error('提交自由输入失败：', error);
    } finally {
      isLoading.value = false;
      loadingMessage.value = '处理中...';
    }
  };

  const handleContinue = async () => {
    try {
      autoAdvanceStopHint.value = '';
      autoAdvanceStopRequested.value = false;
      isLoading.value = true;
      loadingMessage.value = '正在续写剧情...';
      playClick();
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
        await gameStore.executePlayerAction(createContinueAction());

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
      loadingMessage.value = '处理中...';
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
    autoAdvanceRunning,
    autoAdvanceStopHint,
    handleOptionSelect,
    handleFreeTextSubmit,
    handleContinue,
    requestStopAutoAdvance,
  };
};
