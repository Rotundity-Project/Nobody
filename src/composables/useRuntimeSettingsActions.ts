import type { Ref } from 'vue';
import type { ConsistencyPolicy } from '../types/game';
import type { StorySettings } from '../utils/storySettings';
import { saveStorySettings } from '../utils/storySettings';
import { invokeWithTimeout } from '../utils/tauriInvoke';

type UseRuntimeSettingsActionsInput = {
  storySettings: Ref<StorySettings>;
  consistencyPolicy: Ref<ConsistencyPolicy>;
  notifyRuntimeError: (label: string, error: unknown) => void;
};

export const useRuntimeSettingsActions = ({
  storySettings,
  consistencyPolicy,
  notifyRuntimeError,
}: UseRuntimeSettingsActionsInput) => {
  const applyStorySettings = async (settings: StorySettings) => {
    storySettings.value = settings;
    saveStorySettings(settings);
    try {
      await invokeWithTimeout(
        'update_plot_settings',
        {
          settings,
        },
        8000,
        '更新剧情设置超时，请稍后重试',
      );
    } catch (error) {
      notifyRuntimeError('更新剧情设置', error);
    }
  };

  const applyConsistencyPolicy = async (policy: ConsistencyPolicy) => {
    consistencyPolicy.value = policy;
    try {
      const updated = await invokeWithTimeout<ConsistencyPolicy>(
        'update_consistency_policy',
        { policy },
        8000,
        '保存一致性策略超时',
      );
      consistencyPolicy.value = updated;
    } catch (error) {
      notifyRuntimeError('保存一致性策略', error);
    }
  };

  const resetConsistencyPolicy = async () => {
    try {
      const reset = await invokeWithTimeout<ConsistencyPolicy>(
        'reset_consistency_policy',
        undefined,
        8000,
        '重置一致性策略超时',
      );
      consistencyPolicy.value = reset;
    } catch (error) {
      notifyRuntimeError('重置一致性策略', error);
    }
  };

  return {
    applyStorySettings,
    applyConsistencyPolicy,
    resetConsistencyPolicy,
  };
};
