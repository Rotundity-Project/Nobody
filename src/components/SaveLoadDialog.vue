<template>
  <div
    v-if="isOpen"
    class="save-load-overlay fixed inset-0 flex items-center justify-center"
    style="z-index: 50;"
    @click.self="handleClose"
  >
    <div
      class="panel-surface save-load-panel relative max-h-[80vh] w-full max-w-2xl overflow-y-auto rounded-2xl p-8"
      style="z-index: 51;"
    >
      <div class="mb-6 flex items-center justify-between">
        <h2 class="save-load-title text-2xl font-display">
          {{ mode === 'save' ? '保存游戏' : '加载游戏' }}
        </h2>
        <button
          class="save-load-close-btn transition-colors"
          @click="handleClose"
        >
          <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-3">
        <div
          v-for="slot in saveSlots"
          :key="slot.id"
          class="save-slot-item cursor-pointer rounded-lg border-2 p-4 transition-all duration-200"
          :class="[
            selectedSlot === slot.id
              ? 'save-slot-item-active'
              : 'save-slot-item-idle'
          ]"
          @click="selectSlot(slot.id)"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <h3 class="save-slot-title mb-1 text-lg font-semibold">
                存档槽 {{ slot.id }}
              </h3>

              <div v-if="slot.data" class="save-slot-meta space-y-1 text-sm">
                <p>角色：{{ slot.data.characterName }}</p>
                <p>境界：{{ slot.data.realm }}</p>
                <p>位置：{{ slot.data.locationLabel }}</p>
                <p class="save-slot-sub-meta text-xs">游戏时间：{{ slot.data.gameTime }}</p>
                <p class="save-slot-sub-meta text-xs">保存时间：{{ formatDate(slot.data.timestamp) }}</p>
              </div>

              <div v-else class="save-slot-empty text-sm">
                空存档
              </div>
            </div>

            <div v-if="selectedSlot === slot.id" class="ml-4">
              <svg class="save-slot-check h-6 w-6" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>
      </div>

      <div v-if="error" class="save-load-error-wrap mt-4 rounded-lg border p-3">
        <p class="save-load-error-text text-sm">{{ error }}</p>
      </div>

      <div v-if="isLoading" class="mt-4">
        <LoadingIndicator :message="loadingMessage" detail="请保持窗口开启..." size="sm" />
      </div>

      <div class="mt-6 flex gap-3">
        <button
          class="save-load-confirm-btn flex-1 rounded-lg px-6 py-3 font-medium transition-colors duration-200"
          :class="[
            canConfirm && !isLoading
              ? 'save-load-confirm-btn-enabled'
              : 'save-load-confirm-btn-disabled cursor-not-allowed'
          ]"
          :disabled="!canConfirm || isLoading"
          @click="handleConfirm"
        >
          {{ mode === 'save' ? '保存' : '加载' }}
        </button>

        <button
          class="save-load-cancel-btn rounded-lg px-6 py-3 font-medium transition-colors duration-200"
          :disabled="isLoading"
          @click="handleClose"
        >
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useGameStore } from '../stores/gameStore';
import LoadingIndicator from './LoadingIndicator.vue';
import { playClick } from '../utils/audioSystem';
import { formatLocationLabel } from '../shared/locationLabel';

interface Props {
  isOpen: boolean;
  mode: 'save' | 'load';
}

interface SaveSlotData {
  characterName: string;
  realm: string;
  locationLabel: string;
  gameTime: string;
  timestamp: number;
}

interface SaveSlot {
  id: number;
  data: SaveSlotData | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  close: [];
  saved: [slotId: number];
  loaded: [slotId: number];
}>();

const gameStore = useGameStore();
const selectedSlot = ref<number | null>(null);
const isLoading = ref(false);
const error = ref<string | null>(null);
const loadingMessage = ref('处理中...');

const saveSlots = ref<SaveSlot[]>([
  { id: 1, data: null },
  { id: 2, data: null },
  { id: 3, data: null },
  { id: 4, data: null },
  { id: 5, data: null },
]);

const selectedSlotInfo = computed(
  () => saveSlots.value.find((slot) => slot.id === selectedSlot.value) ?? null,
);

const canConfirm = computed(() => {
  if (selectedSlot.value === null) {
    return false;
  }
  if (props.mode === 'save') {
    return true;
  }
  return selectedSlotInfo.value?.data !== null;
});

watch(
  () => props.isOpen,
  (newValue) => {
    if (newValue) {
      selectedSlot.value = null;
      error.value = null;
      void loadSaveSlots();
    }
  },
);

const loadSaveSlots = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在读取存档列表...';
    const saveInfos = await gameStore.listSaveSlots();
    const saveMap = new Map(
      saveInfos.map((info) => [
        info.slot_id,
        {
          characterName: info.player_name,
          realm: info.realm,
          locationLabel: formatLocationLabel(info.location),
          gameTime: info.game_time,
          timestamp: info.timestamp * 1000,
        } as SaveSlotData,
      ]),
    );

    saveSlots.value = [1, 2, 3, 4, 5].map((id) => ({
      id,
      data: saveMap.get(id) ?? null,
    }));
  } catch (err) {
    error.value = err instanceof Error ? err.message : '读取存档列表失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '处理中...';
  }
};

const selectSlot = (slotId: number) => {
  selectedSlot.value = slotId;
  error.value = null;
  playClick();
};

const handleConfirm = async () => {
  if (selectedSlot.value === null) {
    return;
  }

  playClick();

  if (props.mode === 'load' && selectedSlotInfo.value?.data === null) {
    error.value = '该槽位为空，无法加载。';
    return;
  }

  try {
    isLoading.value = true;
    error.value = null;
    loadingMessage.value = props.mode === 'save'
      ? '正在保存到选定槽位...'
      : '正在从槽位加载...';

    if (props.mode === 'save') {
      await gameStore.saveGame(selectedSlot.value);
      emit('saved', selectedSlot.value);
    } else {
      await gameStore.loadGame(selectedSlot.value);
      emit('loaded', selectedSlot.value);
    }

    handleClose();
  } catch (err) {
    error.value = err instanceof Error
      ? err.message
      : `${props.mode === 'save' ? '保存' : '加载'}游戏失败`;
  } finally {
    isLoading.value = false;
    loadingMessage.value = '处理中...';
  }
};

const handleClose = () => {
  if (!isLoading.value) {
    emit('close');
  }
};

const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleString();
};
</script>

<style scoped>
.save-load-overlay {
  background-color: var(--save-load-overlay-bg);
}

.save-load-panel {
  border-color: var(--save-load-panel-border);
  background: var(--save-load-panel-bg);
  box-shadow: var(--save-load-panel-shadow);
}

.save-load-title {
  color: var(--save-load-title-text);
}

.save-load-close-btn {
  color: var(--save-load-close-text);
}

.save-load-close-btn:hover {
  color: var(--save-load-close-hover-text);
}

.save-slot-item {
  border-color: var(--save-load-slot-border);
  background: var(--save-load-slot-bg);
}

.save-slot-item-idle:hover {
  border-color: var(--save-load-slot-hover-border);
}

.save-slot-item-active {
  border-color: var(--save-load-slot-active-border);
  background: var(--save-load-slot-active-bg);
}

.save-slot-title {
  color: var(--save-load-slot-title-text);
}

.save-slot-meta {
  color: var(--save-load-slot-meta-text);
}

.save-slot-sub-meta,
.save-slot-empty {
  color: var(--save-load-slot-muted-text);
}

.save-slot-check {
  color: var(--save-load-slot-check);
}

.save-load-error-wrap {
  border-color: var(--save-load-error-border);
  background: var(--save-load-error-bg);
}

.save-load-error-text {
  color: var(--save-load-error-text);
}

.save-load-confirm-btn-enabled {
  background: var(--save-load-confirm-bg);
  color: var(--save-load-confirm-text);
}

.save-load-confirm-btn-enabled:hover {
  background: var(--save-load-confirm-hover-bg);
}

.save-load-confirm-btn-disabled {
  background: var(--save-load-confirm-disabled-bg);
  color: var(--save-load-confirm-disabled-text);
}

.save-load-cancel-btn {
  background: var(--save-load-cancel-bg);
  color: var(--save-load-cancel-text);
}

.save-load-cancel-btn:hover {
  background: var(--save-load-cancel-hover-bg);
}
</style>
