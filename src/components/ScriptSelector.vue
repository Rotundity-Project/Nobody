<template>
  <div class="min-h-screen flex items-center justify-center p-4 sm:p-8">
    <div class="max-w-2xl w-full panel-surface rounded-2xl p-6 sm:p-8">
      <div class="mb-6 space-y-2">
        <p class="text-xs uppercase tracking-[0.35em] text-amber-200/70">Choose Your Path</p>
        <h2 class="text-2xl sm:text-3xl font-display text-amber-100">选择剧本类型</h2>
      </div>

      <div class="mb-6 rounded-lg border border-slate-700/80 bg-slate-900/60 p-4">
        <label class="text-sm text-slate-300">主角姓名</label>
        <input
          v-model="playerName"
          type="text"
          maxlength="20"
          placeholder="输入你的名字（默认：无名弟子）"
          class="mt-2 w-full rounded-md bg-slate-800/80 px-3 py-2 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-amber-400/70"
        />
        <p class="mt-2 text-xs text-slate-500">将覆盖随机/自定义剧本中的主角姓名。</p>
      </div>

      <div class="space-y-4">
        <div
          v-for="scriptType in scriptTypes"
          :key="scriptType.type"
          class="p-6 rounded-lg border-2 transition-all duration-200"
          :class="[
            scriptType.available
              ? 'border-amber-400/60 bg-slate-800/70 hover:bg-slate-700 cursor-pointer'
              : 'border-slate-700 bg-slate-900/60 opacity-60 cursor-not-allowed'
          ]"
          @click="scriptType.available && selectScriptType(scriptType.type)"
        >
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-xl font-semibold text-slate-100 mb-2">
                {{ scriptType.title }}
              </h3>
              <p class="text-slate-400">{{ scriptType.description }}</p>
            </div>
            <div v-if="!scriptType.available" class="text-slate-500 text-sm">
              即将推出
            </div>
          </div>
        </div>
      </div>

      <div v-if="showCharacterSelect" class="mt-6 p-4 border border-slate-600 rounded-lg bg-slate-900/60">
        <h3 class="text-xl font-semibold text-slate-100 mb-2">选择主角</h3>
        <p class="text-slate-400 mb-4">从小说中选择一个角色作为玩家。</p>

        <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
          <label
            v-for="character in novelCharacters"
            :key="character"
            class="flex items-center gap-2 px-3 py-2 rounded-md bg-slate-900/80 hover:bg-slate-700 cursor-pointer"
          >
            <input
              type="radio"
              name="novel-character"
              class="text-purple-500 focus:ring-purple-500"
              :value="character"
              v-model="selectedCharacter"
            />
            <span class="text-slate-100">{{ character }}</span>
          </label>
        </div>

        <div class="mt-4 flex flex-wrap gap-3">
          <button
            @click="confirmNovelSelection"
          class="px-4 py-2 bg-amber-500 hover:bg-amber-400 text-slate-900 rounded-lg transition-colors duration-200"
        >
          开始导入
        </button>
        <button
          @click="resetNovelSelection"
          class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
        >
          重新选择
        </button>
      </div>
      </div>

      <div v-if="isLoading" class="mt-6">
        <LoadingIndicator :message="loadingMessage" detail="请稍候，正在处理请求..." size="lg" />
      </div>

      <div v-if="error" class="mt-6 p-4 bg-red-900 bg-opacity-50 border border-red-500 rounded-lg">
        <p class="text-red-200">{{ error }}</p>
      </div>

      <button
        @click="handleBack"
        class="mt-6 px-6 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
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
import { invokeWithTimeout } from '../utils/tauriInvoke';
import { useGameStore } from '../stores/gameStore';
import LoadingIndicator from './LoadingIndicator.vue';
import { playClick } from '../utils/audioSystem';
import type { ScriptType, Script } from '../types/game';

const router = useRouter();
const gameStore = useGameStore();
const isLoading = ref(false);
const loadingMessage = ref('加载中...');
const error = ref<string | null>(null);
const playerName = ref('');

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
    available: true,
  },
]);

const selectScriptType = async (type: ScriptType) => {
  error.value = null;
  resetNovelSelection();
  playClick();

  if (type === 'custom') {
    await loadCustomScript();
    return;
  }

  if (type === 'random_generated') {
    await loadRandomScript();
    return;
  }

  if (type === 'existing_novel') {
    await prepareExistingNovel();
    return;
  }

  error.value = '此功能尚未实现';
};

const loadRandomScript = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在生成随机剧本...';
    error.value = null;

    await gameStore.initializeRandomGame(playerName.value);
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

    const script = await invokeWithTimeout<Script>(
      'load_script',
      { scriptPath: selected },
      10000,
      '加载剧本超时，请重试',
    );

    await gameStore.initializeGame(script, playerName.value);
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载剧本失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中...';
  }
};

const novelCharacters = ref<string[]>([]);
const selectedCharacter = ref<string | null>(null);
const selectedNovelPath = ref<string | null>(null);
const showCharacterSelect = ref(false);

const prepareExistingNovel = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在解析小说...';
    error.value = null;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Text',
          extensions: ['txt', 'md'],
        },
      ],
    });

    if (!selected) {
      return;
    }

    selectedNovelPath.value = selected;
    const characters = await invokeWithTimeout<string[]>(
      'parse_novel_characters',
      { novelPath: selected },
      15000,
      '解析小说超时，请检查文件或重试',
    );

    if (!characters.length) {
      throw new Error('未能从小说中解析出角色列表');
    }

    novelCharacters.value = characters;
    selectedCharacter.value = characters[0] ?? null;
    showCharacterSelect.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : '小说解析失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中...';
  }
};

const confirmNovelSelection = async () => {
  if (!selectedNovelPath.value) {
    error.value = '请先选择小说文件';
    return;
  }

  if (!selectedCharacter.value) {
    error.value = '请选择一个角色';
    return;
  }

  try {
    isLoading.value = true;
    loadingMessage.value = '正在导入小说剧本...';
    error.value = null;

    const script = await invokeWithTimeout<Script>(
      'load_existing_novel',
      {
        novelPath: selectedNovelPath.value,
        selectedCharacter: selectedCharacter.value,
      },
      20000,
      '导入小说超时，请重试',
    );

    await gameStore.initializeGame(script, playerName.value);
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '小说导入失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中...';
  }
};

const resetNovelSelection = () => {
  novelCharacters.value = [];
  selectedCharacter.value = null;
  selectedNovelPath.value = null;
  showCharacterSelect.value = false;
};

const handleBack = () => {
  playClick();
  resetNovelSelection();
  router.push('/');
};
</script>
