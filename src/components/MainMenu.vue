<template>
  <div class="min-h-screen flex items-center justify-center px-4 py-8 sm:py-12 relative overflow-hidden">
    <!-- 背景光效 -->
    <div class="absolute inset-0 pointer-events-none overflow-hidden">
      <div class="absolute top-10 left-10 w-64 h-64 bg-amber-500/10 rounded-full blur-3xl animate-pulse"></div>
      <div class="absolute bottom-10 right-10 w-80 h-80 bg-emerald-500/10 rounded-full blur-3xl animate-pulse" style="animation-delay: 1.5s;"></div>
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-purple-500/5 rounded-full blur-3xl animate-pulse" style="animation-delay: 0.75s;"></div>
    </div>

    <!-- 主面板 -->
    <div class="panel-surface text-center space-y-8 sm:space-y-10 p-6 sm:p-10 rounded-2xl max-w-xl w-full relative z-10 animate-fade-up">
      <!-- 标题区域 -->
      <div class="space-y-4">
        <p class="text-xs sm:text-sm uppercase tracking-[0.4em] text-amber-200/70 font-medium">Immortal Chronicle</p>
        <h1 class="text-4xl sm:text-6xl lg:text-7xl font-display text-glow gradient-text mb-2">
          Nobody
        </h1>
        <div class="inline-block">
          <span class="badge badge-amber px-6 py-2">修仙模拟器</span>
        </div>
      </div>

      <!-- 按钮组 -->
      <div class="space-y-4 mt-10 sm:mt-12">
        <button
          @click="handleNewGame"
          class="w-full sm:w-auto min-w-[200px] px-8 py-4 rounded-xl btn-primary text-slate-900 font-semibold text-lg transition-all duration-300 relative overflow-hidden group"
        >
          <span class="relative z-10 flex items-center justify-center gap-3">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            新游戏
          </span>
          <div class="absolute inset-0 bg-gradient-to-r from-amber-400 to-amber-600 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        </button>

        <button
          @click="handleLoadGame"
          class="w-full sm:w-auto min-w-[200px] px-8 py-4 rounded-xl btn-secondary text-white font-semibold text-lg transition-all duration-300 relative overflow-hidden group"
        >
          <span class="relative z-10 flex items-center justify-center gap-3">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            加载游戏
          </span>
          <div class="absolute inset-0 bg-gradient-to-r from-slate-600 to-slate-700 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        </button>

        <button
          @click="handleSettings"
          class="w-full sm:w-auto min-w-[200px] px-8 py-4 rounded-xl btn-emerald text-slate-900 font-semibold text-lg transition-all duration-300 relative overflow-hidden group"
        >
          <span class="relative z-10 flex items-center justify-center gap-3">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            LLM 设置
          </span>
          <div class="absolute inset-0 bg-gradient-to-r from-emerald-400 to-emerald-600 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        </button>
      </div>

      <!-- 音频设置 -->
      <div class="mt-8 sm:mt-10 text-left">
        <div class="flex items-center gap-2 mb-4">
          <svg class="w-5 h-5 text-amber-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
          </svg>
          <h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider">音频设置</h2>
        </div>
        <div class="card p-4">
          <AudioControlPanel />
        </div>
      </div>

      <!-- 底部信息 -->
      <div class="mt-8 pt-6 border-t border-slate-700/50">
        <p class="text-xs text-slate-500">
          Made with <span class="text-red-400">♥</span> for Cultivation lovers
        </p>
      </div>
    </div>

    <!-- LLM配置对话框 -->
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

<style scoped>
/* 动画延迟 */
@keyframes pulse-slow {
  0%, 100% {
    opacity: 0.5;
    transform: scale(1);
  }
  50% {
    opacity: 0.8;
    transform: scale(1.05);
  }
}

.animate-pulse {
  animation: pulse-slow 4s ease-in-out infinite;
}

/* 按钮悬停效果 */
.group:hover .relative {
  z-index: 20;
}

/* 响应式调整 */
@media (max-width: 640px) {
  .panel-surface {
    padding: 1.5rem;
  }

  .space-y-4 > * + * {
    margin-top: 1rem;
  }
}
</style>
