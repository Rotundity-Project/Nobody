<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
    <div class="text-center space-y-8 p-8">
      <h1 class="text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-600">
        Nobody
      </h1>
      <p class="text-xl text-gray-300">修仙模拟器</p>

      <div class="space-y-4 mt-12">
        <button
          @click="handleNewGame"
          class="w-64 px-6 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors duration-200 font-medium"
        >
          新游戏
        </button>

        <button
          @click="handleLoadGame"
          class="w-64 px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors duration-200 font-medium"
        >
          加载游戏
        </button>

        <button
          @click="handleSettings"
          class="w-64 px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors duration-200 font-medium"
        >
          LLM 设置
        </button>
      </div>
    </div>

    <LLMConfigDialog :is-open="showLLMDialog" @close="showLLMDialog = false" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import LLMConfigDialog from './LLMConfigDialog.vue';

const router = useRouter();
const showLLMDialog = ref(false);

const handleNewGame = () => {
  router.push('/script-select');
};

const handleLoadGame = () => {
  // TODO: Implement load game functionality
  console.log('点击了加载游戏');
};

const handleSettings = () => {
  showLLMDialog.value = true;
};
</script>
