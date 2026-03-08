import { computed, ref, type Ref } from 'vue';
import type { NotificationItem } from '../components/NotificationCenter.vue';
import { buildRuntimeErrorNotification } from './useRuntimeActionFeedback';

type UseRuntimeNotificationsInput = {
  characterCreationDurationLabel: Ref<string>;
  autoAdvanceStopHint: Ref<string>;
  actionNotification: Ref<NotificationItem | null>;
  runtimeError: Ref<string | null>;
  backgroundNotice: Ref<{ id: string; message: string } | null>;
  clearBackgroundNotice: () => void;
};

export const useRuntimeNotifications = ({
  characterCreationDurationLabel,
  autoAdvanceStopHint,
  actionNotification,
  runtimeError,
  backgroundNotice,
  clearBackgroundNotice,
}: UseRuntimeNotificationsInput) => {
  const dismissedNotificationIds = ref<string[]>([]);

  const runtimeNotifications = computed<NotificationItem[]>(() => {
    const out: NotificationItem[] = [];

    if (characterCreationDurationLabel.value) {
      out.push({
        id: `character-init-${characterCreationDurationLabel.value}`,
        kind: 'info',
        title: '角色创建完成',
        message: `创建耗时 ${characterCreationDurationLabel.value}`,
        priority: 'toast',
      });
    }

    if (autoAdvanceStopHint.value) {
      out.push({
        id: 'auto-advance-stop',
        kind: 'validation',
        title: '自动推进已暂停',
        message: autoAdvanceStopHint.value,
        priority: 'toast',
      });
    }

    if (actionNotification.value) {
      out.push(actionNotification.value);
    }

    if (runtimeError.value) {
      const seed = runtimeError.value.slice(0, 80).replace(/\s+/g, '_');
      out.push(buildRuntimeErrorNotification('剧情推进', runtimeError.value, seed));
    }
    if (backgroundNotice.value) {
      out.push({
        id: backgroundNotice.value.id,
        kind: 'info',
        title: '剧情已补全',
        message: backgroundNotice.value.message,
        priority: 'toast',
      });
    }

    return out.filter((item) => !dismissedNotificationIds.value.includes(item.id));
  });

  const dismissRuntimeNotification = (id: string) => {
    if (!dismissedNotificationIds.value.includes(id)) {
      dismissedNotificationIds.value.push(id);
    }
    if (backgroundNotice.value?.id === id) {
      clearBackgroundNotice();
    }
  };

  return {
    runtimeNotifications,
    dismissRuntimeNotification,
  };
};
