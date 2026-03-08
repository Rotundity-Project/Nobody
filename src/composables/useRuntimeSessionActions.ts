import { ref } from 'vue';
import { type useGameStore } from '../stores/gameStore';

export const useRuntimeSessionActions = ({
  gameStore,
  logRuntimeAction,
  notifyRuntimeError,
}: {
  gameStore: ReturnType<typeof useGameStore>;
  logRuntimeAction: (title: string, detail?: string) => void;
  notifyRuntimeError: (scope: string, error: unknown) => void;
}) => {
  const travelPending = ref(false);

  const handleSaved = (slotId: number) => {
    logRuntimeAction('保存完成', `槽位 ${slotId}`);
  };

  const handleLoaded = (slotId: number) => {
    logRuntimeAction('读档完成', `槽位 ${slotId}`);
  };

  const handleTravel = async (locationId: string) => {
    if (!locationId) return;
    try {
      travelPending.value = true;
      await gameStore.travelToLocation(locationId);
      logRuntimeAction('地点移动完成', `目标：${locationId}`);
    } catch (error) {
      notifyRuntimeError('地点移动', error);
    } finally {
      travelPending.value = false;
    }
  };

  return {
    travelPending,
    handleSaved,
    handleLoaded,
    handleTravel,
  };
};
