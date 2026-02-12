<template>
  <div class="min-h-screen bg-slate-900 text-white flex">
    <CharacterPanel :character="gameStore.playerCharacter" />

    <div class="flex-1 flex flex-col">
      <div class="bg-slate-800 border-b border-slate-700 px-6 py-3 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <button
            @click="router.push('/')"
            class="text-gray-400 hover:text-white transition-colors"
            title="返回"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <h1 class="text-xl font-bold text-purple-400">Nobody</h1>
        </div>
        <div class="flex gap-2">
          <button
            @click="showLLMDialog = true"
            class="px-4 py-2 bg-violet-600 hover:bg-violet-700 text-white rounded-lg transition-colors duration-200"
          >
            LLM 设置
          </button>
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

      <div class="flex-1 overflow-y-auto p-8">
        <div class="max-w-3xl mx-auto space-y-4">
          <div v-if="gameStore.plotState && gameStore.currentScene" class="prose prose-invert max-w-none">
            <h2 class="text-2xl font-bold text-purple-400 mb-4">{{ gameStore.currentScene.name }}</h2>
            <p class="text-gray-300 leading-relaxed whitespace-pre-wrap">{{ gameStore.currentScene.description }}</p>
          </div>

          <div v-if="!gameStore.isGameInitialized" class="text-center text-gray-400">
            <p>当前没有进行中的游戏，请先开始新游戏。</p>
          </div>
        </div>
      </div>

      <div class="border-t border-slate-700 bg-slate-800 p-6">
        <div class="max-w-3xl mx-auto">
          <div v-if="gameStore.isWaitingForInput && gameStore.isGameInitialized" class="space-y-4">
            <div class="flex items-center gap-2">
              <button
                @click="inputMode = 'options'"
                class="px-3 py-1 rounded"
                :class="inputMode === 'options' ? 'bg-purple-600 text-white' : 'bg-slate-700 text-gray-300'"
              >
                选项
              </button>
              <button
                @click="inputMode = 'freeText'"
                class="px-3 py-1 rounded"
                :class="inputMode === 'freeText' ? 'bg-purple-600 text-white' : 'bg-slate-700 text-gray-300'"
              >
                自由输入
              </button>
            </div>

            <div v-if="inputMode === 'options' && gameStore.availableOptions.length > 0" class="space-y-2">
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
                <p v-if="option.requirements && option.requirements.length > 0" class="text-sm text-gray-400 mt-1">
                  条件：{{ option.requirements.join('，') }}
                </p>
              </button>
            </div>

            <div v-if="inputMode === 'freeText'" class="space-y-2">
              <textarea
                v-model="freeTextInput"
                :disabled="isLoading"
                rows="3"
                maxlength="200"
                placeholder="输入你想执行的行为，例如：我去后山修炼。"
                class="w-full bg-slate-700 border border-slate-600 rounded-lg p-3 text-white outline-none focus:border-purple-500"
              />
              <p v-if="inputValidation.message" class="text-sm" :class="inputValidation.valid ? 'text-gray-300' : 'text-amber-300'">
                {{ inputValidation.message }}
              </p>
              <button
                @click="handleFreeTextSubmit"
                :disabled="isLoading || !inputValidation.valid"
                class="px-4 py-2 rounded-lg transition-colors"
                :class="isLoading || !inputValidation.valid ? 'bg-gray-600 text-gray-400 cursor-not-allowed' : 'bg-purple-600 hover:bg-purple-700 text-white'"
              >
                提交自由输入
              </button>
            </div>
          </div>

          <div v-else-if="isLoading" class="text-center">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
            <p class="text-gray-300 mt-2">处理中...</p>
          </div>

          <div v-else-if="gameStore.isGameInitialized && !gameStore.isWaitingForInput" class="text-center text-gray-400">
            <p>剧情推进中...</p>
          </div>
        </div>
      </div>

      <div class="p-6 bg-slate-900/60">
        <div class="max-w-3xl mx-auto">
          <NovelExporter :is-game-running="gameStore.isGameInitialized" :event-count="eventCount" />
        </div>
      </div>

      <div v-if="gameStore.error" class="p-4 bg-red-900 bg-opacity-50 border-t border-red-500">
        <div class="max-w-3xl mx-auto">
          <p class="text-red-200">{{ gameStore.error }}</p>
          <button
            @click="gameStore.clearError"
            class="mt-2 px-4 py-1 bg-red-700 hover:bg-red-600 rounded text-sm transition-colors"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

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

    <LLMConfigDialog :is-open="showLLMDialog" @close="showLLMDialog = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import CharacterPanel from './CharacterPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import NovelExporter from './NovelExporter.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import type { PlayerOption } from '../types/game';
import {
  createFreeTextAction,
  createOptionAction,
  validateFreeTextInput,
} from '../utils/playerInput';

const router = useRouter();
const gameStore = useGameStore();

const isLoading = ref(false);
const showSaveDialog = ref(false);
const showLoadDialog = ref(false);
const showLLMDialog = ref(false);
const inputMode = ref<'options' | 'freeText'>('options');
const freeTextInput = ref('');

const inputValidation = computed(() => validateFreeTextInput(freeTextInput.value));
const eventCount = computed(() => gameStore.gameState?.event_history?.length ?? 0);

const handleOptionSelect = async (option: PlayerOption) => {
  try {
    isLoading.value = true;
    await gameStore.executePlayerAction(createOptionAction(option));
  } catch (error) {
    console.error('执行行动失败：', error);
  } finally {
    isLoading.value = false;
  }
};

const handleFreeTextSubmit = async () => {
  const check = validateFreeTextInput(freeTextInput.value);
  if (!check.valid) {
    return;
  }

  try {
    isLoading.value = true;
    await gameStore.executePlayerAction(createFreeTextAction(freeTextInput.value));
    freeTextInput.value = '';
  } catch (error) {
    console.error('提交自由输入失败：', error);
  } finally {
    isLoading.value = false;
  }
};

const handleSaved = (slotId: number) => {
  console.log(`游戏已保存到槽位 ${slotId}`);
};

const handleLoaded = (slotId: number) => {
  console.log(`已从槽位 ${slotId} 加载游戏`);
};
</script>
