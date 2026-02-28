import { computed, ref, type Ref } from 'vue';
import type { NotificationItem } from '../components/NotificationCenter.vue';

type UseRuntimeNotificationsInput = {
  characterCreationDurationLabel: Ref<string>;
  autoAdvanceStopHint: Ref<string>;
  actionNotification: Ref<NotificationItem | null>;
};

export const useRuntimeNotifications = ({
  characterCreationDurationLabel,
  autoAdvanceStopHint,
  actionNotification,
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

    return out.filter((item) => !dismissedNotificationIds.value.includes(item.id));
  });

  const dismissRuntimeNotification = (id: string) => {
    if (!dismissedNotificationIds.value.includes(id)) {
      dismissedNotificationIds.value.push(id);
    }
  };

  return {
    runtimeNotifications,
    dismissRuntimeNotification,
  };
};
