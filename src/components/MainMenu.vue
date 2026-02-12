<template>
  <div class="min-h-screen flex items-center justify-center px-4 py-8">
    <div class="panel-surface text-center space-y-8 p-6 sm:p-10 rounded-2xl max-w-xl w-full">
      <div class="space-y-3">
        <p class="text-xs uppercase tracking-[0.3em] text-amber-200/70">Immortal Chronicle</p>
        <h1 class="text-4xl sm:text-6xl font-display text-glow text-amber-200">
          Nobody
        </h1>
        <p class="text-lg sm:text-xl text-slate-300 font-story">修仙模拟器</p>
      </div>

      <div class="space-y-4 mt-10 sm:mt-12">
        <button
          @click="handleNewGame"
          class="w-full sm:w-64 px-6 py-3 rounded-lg transition-colors duration-200 font-medium bg-amber-600 hover:bg-amber-500 text-slate-900"
        >
          新游戏
        </button>

        <button
          @click="handleLoadGame"
          class="w-full sm:w-64 px-6 py-3 rounded-lg transition-colors duration-200 font-medium bg-slate-700 hover:bg-slate-600 text-slate-100"
        >
          加载游戏
        </button>

        <button
          @click="handleSettings"
          class="w-full sm:w-64 px-6 py-3 rounded-lg transition-colors duration-200 font-medium bg-emerald-600 hover:bg-emerald-500 text-slate-900"
        >
          LLM 设置
        </button>
      </div>

      <div class="mt-8 text-left">
        <h2 class="text-sm font-semibold text-slate-300 mb-3">音频设置</h2>
        <AudioControlPanel />
      </div>
    </div>

    <LLMConfigDialog :is-open="showLLMDialog" @close="showLLMDialog = false" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import AudioControlPanel from './AudioControlPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import { playClick } from '../utils/audioSystem';

const router = useRouter();
const showLLMDialog = ref(false);

const handleNewGame = () => {
  playClick();
  router.push('/script-select');
};

const handleLoadGame = () => {
  playClick();
  // TODO: Implement load game functionality
  console.log('点击了加载游戏');
};

const handleSettings = () => {
  playClick();
  showLLMDialog.value = true;
};
</script>
