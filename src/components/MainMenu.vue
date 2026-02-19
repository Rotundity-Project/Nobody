<template>
  <div class="min-h-screen flex items-center justify-center px-4 py-8">
    <div class="panel-surface w-full max-w-3xl rounded-2xl p-6 sm:p-10">
      <div class="space-y-3 text-center">
        <p class="text-xs uppercase tracking-[0.3em] text-amber-200/70">Immortal Chronicle</p>
        <h1 class="text-4xl sm:text-6xl font-display text-glow text-amber-200">
          Nobody
        </h1>
        <p class="text-lg sm:text-xl text-slate-300 font-story">修仙模拟器</p>
      </div>

      <div class="mt-8 grid gap-4 lg:grid-cols-[1.1fr_0.9fr]">
        <div class="space-y-3">
          <button
            @click="handleNewGame"
            class="w-full sm:w-64 rounded-lg bg-amber-600 px-6 py-3 font-medium text-slate-900 transition-colors duration-200 hover:bg-amber-500"
          >
            新游戏
          </button>
          <button
            @click="handleLoadGame"
            class="w-full sm:w-64 rounded-lg bg-slate-700 px-6 py-3 font-medium text-slate-100 transition-colors duration-200 hover:bg-slate-600"
          >
            继续游戏
          </button>
          <button
            @click="handleSettings"
            class="w-full sm:w-64 rounded-lg bg-emerald-600 px-6 py-3 font-medium text-slate-900 transition-colors duration-200 hover:bg-emerald-500"
          >
            LLM 设置
          </button>
          <p class="pt-1 text-xs text-slate-400">
            建议流程：新游戏 -> 选择剧本 -> 进入主界面推进剧情
          </p>
        </div>

        <div class="rounded-xl border border-slate-700 bg-slate-900/60 p-4">
          <h2 class="text-sm font-semibold text-slate-200">最近存档</h2>
          <p class="mt-2 text-xs leading-5 text-slate-400">
            暂未接入主菜单读取存档槽位。你可以先进入游戏界面后通过“加载”按钮恢复进度。
          </p>
          <button
            class="mt-3 rounded-md border border-slate-600 px-3 py-1.5 text-xs text-slate-200 transition-colors hover:bg-slate-800"
            @click="handleLoadGame"
          >
            进入并继续
          </button>
        </div>
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
  router.push('/game');
};

const handleSettings = () => {
  playClick();
  showLLMDialog.value = true;
};
</script>
