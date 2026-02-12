<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 flex items-center justify-center"
    style="z-index: 50; background-color: rgba(0, 0, 0, 0.75);"
    @click.self="handleClose"
  >
    <div
      class="panel-surface rounded-2xl p-8 max-w-2xl w-full max-h-[80vh] overflow-y-auto relative"
      style="z-index: 51;"
    >
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-2xl font-display text-amber-100">
          {{ mode === 'save' ? '保存游戏' : '加载游戏' }}
        </h2>
        <button
          @click="handleClose"
          class="text-gray-400 hover:text-white transition-colors"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-3">
        <div
          v-for="slot in saveSlots"
          :key="slot.id"
          class="p-4 rounded-lg border-2 transition-all duration-200 cursor-pointer"
          :class="[
            selectedSlot === slot.id
              ? 'border-purple-500 bg-slate-700'
              : 'border-slate-600 bg-slate-750 hover:border-slate-500'
          ]"
          @click="selectSlot(slot.id)"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <h3 class="text-lg font-semibold text-slate-100 mb-1">
                存档槽 {{ slot.id }}
              </h3>

              <div v-if="slot.data" class="text-sm text-slate-300 space-y-1">
                <p>角色：{{ slot.data.characterName }}</p>
                <p>境界：{{ slot.data.realm }}</p>
                <p>位置：{{ slot.data.location }}</p>
                <p class="text-slate-400 text-xs">游戏时间：{{ slot.data.gameTime }}</p>
                <p class="text-slate-400 text-xs">保存时间：{{ formatDate(slot.data.timestamp) }}</p>
              </div>

              <div v-else class="text-sm text-slate-500">
                空存档
              </div>
            </div>

            <div v-if="selectedSlot === slot.id" class="ml-4">
              <svg class="w-6 h-6 text-purple-500" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>
      </div>

      <div v-if="error" class="mt-4 p-3 bg-red-900 bg-opacity-50 border border-red-500 rounded-lg">
        <p class="text-red-200 text-sm">{{ error }}</p>
      </div>

      <div v-if="isLoading" class="mt-4">
        <LoadingIndicator :message="loadingMessage" detail="请保持窗口开启..." size="sm" />
      </div>

      <div class="flex gap-3 mt-6">
        <button
          @click="handleConfirm"
          :disabled="!canConfirm || isLoading"
          class="flex-1 px-6 py-3 rounded-lg font-medium transition-colors duration-200"
          :class="[
            canConfirm && !isLoading
              ? 'bg-amber-500 hover:bg-amber-400 text-slate-900'
              : 'bg-gray-600 text-gray-400 cursor-not-allowed'
          ]"
        >
          {{ mode === 'save' ? '保存' : '加载' }}
        </button>

        <button
          @click="handleClose"
          :disabled="isLoading"
          class="px-6 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors duration-200"
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

interface Props {
  isOpen: boolean;
  mode: 'save' | 'load';
}

interface SaveSlotData {
  characterName: string;
  realm: string;
  location: string;
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
  () => saveSlots.value.find((slot) => slot.id === selectedSlot.value) ?? null
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
  }
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
          location: info.location,
          gameTime: info.game_time,
          timestamp: info.timestamp * 1000,
        } as SaveSlotData,
      ])
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
    loadingMessage.value =
      props.mode === 'save' ? '正在保存到选定槽位...' : '正在从槽位加载...';

    if (props.mode === 'save') {
      await gameStore.saveGame(selectedSlot.value);
      emit('saved', selectedSlot.value);
    } else {
      await gameStore.loadGame(selectedSlot.value);
      emit('loaded', selectedSlot.value);
    }

    handleClose();
  } catch (err) {
    error.value =
      err instanceof Error
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
