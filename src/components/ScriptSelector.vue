<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 p-8">
    <div class="max-w-2xl w-full bg-slate-800 rounded-lg shadow-2xl p-8">
      <h2 class="text-3xl font-bold text-white mb-6">选择剧本类型</h2>

      <div class="space-y-4">
        <div
          v-for="scriptType in scriptTypes"
          :key="scriptType.type"
          class="p-6 rounded-lg border-2 transition-all duration-200"
          :class="[
            scriptType.available
              ? 'border-purple-500 bg-slate-700 hover:bg-slate-600 cursor-pointer'
              : 'border-gray-600 bg-slate-800 opacity-50 cursor-not-allowed'
          ]"
          @click="scriptType.available && selectScriptType(scriptType.type)"
        >
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-xl font-semibold text-white mb-2">
                {{ scriptType.title }}
              </h3>
              <p class="text-gray-300">{{ scriptType.description }}</p>
            </div>
            <div v-if="!scriptType.available" class="text-gray-500 text-sm">
              即将推出
            </div>
          </div>
        </div>
      </div>

      <div v-if="isLoading" class="mt-6 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
        <p class="text-gray-300 mt-2">{{ loadingMessage }}</p>
      </div>

      <div v-if="error" class="mt-6 p-4 bg-red-900 bg-opacity-50 border border-red-500 rounded-lg">
        <p class="text-red-200">{{ error }}</p>
      </div>

      <button
        @click="handleBack"
        class="mt-6 px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors duration-200"
      >
        返回
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useGameStore } from '../stores/gameStore';
import type { ScriptType, Script } from '../types/game';

const router = useRouter();
const gameStore = useGameStore();
const isLoading = ref(false);
const loadingMessage = ref('加载中...');
const error = ref<string | null>(null);

interface ScriptTypeOption {
  type: ScriptType;
  title: string;
  description: string;
  available: boolean;
}

const scriptTypes = ref<ScriptTypeOption[]>([
  {
    type: 'custom' as ScriptType,
    title: '自定义剧本',
    description: '加载自定义 JSON 剧本文件',
    available: true,
  },
  {
    type: 'random_generated' as ScriptType,
    title: '随机生成',
    description: '使用 AI 生成随机修仙世界',
    available: true,
  },
  {
    type: 'existing_novel' as ScriptType,
    title: '现有小说',
    description: '从现有修仙小说导入剧本',
    available: false,
  },
]);

const selectScriptType = async (type: ScriptType) => {
  error.value = null;

  if (type === 'custom') {
    await loadCustomScript();
    return;
  }

  if (type === 'random_generated') {
    await loadRandomScript();
    return;
  }

  error.value = '此功能尚未实现';
};

const loadRandomScript = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在生成随机剧本...';
    error.value = null;

    await gameStore.initializeRandomGame();
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '随机剧本生成失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中...';
  }
};

const loadCustomScript = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在加载自定义剧本...';
    error.value = null;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
    });

    if (!selected) {
      return;
    }

    const script = await invoke<Script>('load_script', {
      scriptPath: selected,
    });

    await gameStore.initializeGame(script);
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载剧本失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中...';
  }
};

const handleBack = () => {
  router.push('/');
};
</script>
