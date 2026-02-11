<template>
  <div 
    v-if="isOpen"
    class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50"
    @click.self="handleClose"
  >
    <div class="bg-slate-800 rounded-lg shadow-2xl p-8 max-w-2xl w-full max-h-[80vh] overflow-y-auto">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-2xl font-bold text-white">
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
              <h3 class="text-lg font-semibold text-white mb-1">
                存档槽 {{ slot.id }}
              </h3>
              
              <div v-if="slot.data" class="text-sm text-gray-300 space-y-1">
                <p>角色: {{ slot.data.characterName }}</p>
                <p>境界: {{ slot.data.realm }}</p>
                <p>位置: {{ slot.data.location }}</p>
                <p class="text-gray-400 text-xs">
                  保存时间: {{ formatDate(slot.data.timestamp) }}
                </p>
              </div>
              
              <div v-else class="text-sm text-gray-500">
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

      <div v-if="isLoading" class="mt-4 text-center">
        <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-purple-500"></div>
        <p class="text-gray-300 text-sm mt-2">处理中...</p>
      </div>

      <div class="flex gap-3 mt-6">
        <button
          @click="handleConfirm"
          :disabled="selectedSlot === null || isLoading"
          class="flex-1 px-6 py-3 rounded-lg font-medium transition-colors duration-200"
          :class="[
            selectedSlot !== null && !isLoading
              ? 'bg-purple-600 hover:bg-purple-700 text-white'
              : 'bg-gray-600 text-gray-400 cursor-not-allowed'
          ]"
        >
          {{ mode === 'save' ? '保存' : '加载' }}
        </button>
        
        <button
          @click="handleClose"
          :disabled="isLoading"
          class="px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white rounded-lg font-medium transition-colors duration-200"
        >
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useGameStore } from '../stores/gameStore';

interface Props {
  isOpen: boolean;
  mode: 'save' | 'load';
}

interface SaveSlotData {
  characterName: string;
  realm: string;
  location: string;
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

const saveSlots = ref<SaveSlot[]>([
  { id: 1, data: null },
  { id: 2, data: null },
  { id: 3, data: null },
  { id: 4, data: null },
  { id: 5, data: null },
]);

watch(() => props.isOpen, (newValue) => {
  if (newValue) {
    selectedSlot.value = null;
    error.value = null;
    loadSaveSlots();
  }
});

const loadSaveSlots = async () => {
  // TODO: Load actual save slot data from backend
  // For now, using mock data
  console.log('Loading save slots...');
};

const selectSlot = (slotId: number) => {
  selectedSlot.value = slotId;
  error.value = null;
};

const handleConfirm = async () => {
  if (selectedSlot.value === null) return;

  try {
    isLoading.value = true;
    error.value = null;

    if (props.mode === 'save') {
      await gameStore.saveGame(selectedSlot.value);
      emit('saved', selectedSlot.value);
    } else {
      await gameStore.loadGame(selectedSlot.value);
      emit('loaded', selectedSlot.value);
    }

    handleClose();
  } catch (err) {
    error.value = err instanceof Error ? err.message : `${props.mode === 'save' ? '保存' : '加载'}游戏失败`;
  } finally {
    isLoading.value = false;
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
