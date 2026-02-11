<template>
  <div class="min-h-screen bg-slate-900 text-white flex">
    <!-- Character Panel (Left Sidebar) -->
    <CharacterPanel :character="gameStore.playerCharacter" />

    <!-- Main Game Area -->
    <div class="flex-1 flex flex-col">
      <!-- Top Menu Bar -->
      <div class="bg-slate-800 border-b border-slate-700 px-6 py-3 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <button
            @click="router.push('/')"
            class="text-gray-400 hover:text-white transition-colors"
            title="返回主菜单"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <h1 class="text-xl font-bold text-purple-400">Nobody</h1>
        </div>
        <div class="flex gap-2">
          <button
            @click="showSaveDialog = true"
            :disabled="!gameStore.isGameInitialized"
            class="px-4 py-2 rounded-lg transition-colors duration-200"
            :class="[
              gameStore.isGameInitialized
                ? 'bg-blue-600 hover:bg-blue-700 text-white'
                : 'bg-gray-600 text-gray-400 cursor-not-allowed'
            ]"
          >
            保存
          </button>
          <button
            @click="showLoadDialog = true"
            class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors duration-200"
          >
            加载
          </button>
        </div>
      </div>
      
      <!-- Plot Text Area -->
      <div class="flex-1 overflow-y-auto p-8">
        <div class="max-w-3xl mx-auto space-y-4">
          <div v-if="gameStore.plotState && gameStore.currentScene" class="prose prose-invert max-w-none">
            <h2 class="text-2xl font-bold text-purple-400 mb-4">{{ gameStore.currentScene.name }}</h2>
            <p class="text-gray-300 leading-relaxed whitespace-pre-wrap">{{ gameStore.currentScene.description }}</p>
          </div>
          
          <div v-if="!gameStore.isGameInitialized" class="text-center text-gray-400">
            <p>暂无游戏进行中，请开始新游戏</p>
          </div>
        </div>
      </div>

      <!-- Player Options Area -->
      <div class="border-t border-slate-700 bg-slate-800 p-6">
        <div class="max-w-3xl mx-auto">
          <div v-if="gameStore.isWaitingForInput && gameStore.availableOptions.length > 0">
            <h3 class="text-lg font-semibold mb-4 text-purple-400">选择你的行动：</h3>
            <div class="space-y-2">
              <button
                v-for="(option, index) in gameStore.availableOptions"
                :key="index"
                @click="handleOptionSelect(option)"
                :disabled="isLoading"
                class="w-full text-left p-4 rounded-lg border-2 transition-all duration-200"
                :class="[
                  isLoading 
                    ? 'border-gray-600 bg-slate-700 opacity-50 cursor-not-allowed' 
                    : 'border-purple-500 bg-slate-700 hover:bg-slate-600 cursor-pointer'
                ]"
              >
                <p class="text-white">{{ option.description }}</p>
                <p v-if="option.requirements" class="text-sm text-gray-400 mt-1">
                  需求: {{ option.requirements }}
                </p>
              </button>
            </div>
          </div>

          <div v-else-if="isLoading" class="text-center">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
            <p class="text-gray-300 mt-2">处理中...</p>
          </div>

          <div v-else-if="gameStore.isGameInitialized && !gameStore.isWaitingForInput" class="text-center text-gray-400">
            <p>剧情继续...</p>
          </div>
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="gameStore.error" class="p-4 bg-red-900 bg-opacity-50 border-t border-red-500">
        <div class="max-w-3xl mx-auto">
          <p class="text-red-200">{{ gameStore.error }}</p>
          <button
            @click="gameStore.clearError"
            class="mt-2 px-4 py-1 bg-red-700 hover:bg-red-600 rounded text-sm transition-colors"
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>

    <!-- Save/Load Dialogs -->
    <SaveLoadDialog
      :is-open="showSaveDialog"
      mode="save"
      @close="showSaveDialog = false"
      @saved="handleSaved"
    />
    
    <SaveLoadDialog
      :is-open="showLoadDialog"
      mode="load"
      @close="showLoadDialog = false"
      @loaded="handleLoaded"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import CharacterPanel from './CharacterPanel.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import type { PlayerOption } from '../types/game';
import { ActionType } from '../types/game';

const router = useRouter();

const gameStore = useGameStore();
const isLoading = ref(false);
const showSaveDialog = ref(false);
const showLoadDialog = ref(false);

const handleOptionSelect = async (option: PlayerOption) => {
  try {
    isLoading.value = true;
    await gameStore.executePlayerAction({
      action_type: ActionType.SelectedOption,
      content: option.description,
      selected_option_id: option.id,
    });
  } catch (error) {
    console.error('Failed to execute action:', error);
  } finally {
    isLoading.value = false;
  }
};

const handleSaved = (slotId: number) => {
  console.log(`Game saved to slot ${slotId}`);
};

const handleLoaded = (slotId: number) => {
  console.log(`Game loaded from slot ${slotId}`);
};

</script>
