<template>
  <div class="min-h-screen text-white flex flex-col">
    <div class="flex-1 flex flex-col">
      <div class="bg-slate-900/80 border-b border-slate-700 px-6 py-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between backdrop-blur">
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
          <h1 class="text-xl font-display text-amber-200">Nobody</h1>
        </div>
        <div class="flex flex-wrap gap-2">
          <div class="relative">
            <button
              @click="toggleAudioPanel"
              class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
            >
              音量
            </button>
            <div
              v-if="showAudioPanel"
              class="absolute right-0 mt-2 w-64 panel-surface rounded-xl p-4"
            >
              <AudioControlPanel />
            </div>
          </div>
          <button
            @click="showLLMDialog = true"
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-slate-900 rounded-lg transition-colors duration-200"
          >
            LLM 设置
          </button>
          <button
            @click="showStorySettings = true"
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
          >
            剧情设置
          </button>
          <button
            @click="showCharacterInfo = true"
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
          >
            角色信息
          </button>
          <button
            @click="showSaveDialog = true"
            :disabled="!gameStore.isGameInitialized"
            class="px-4 py-2 rounded-lg transition-colors duration-200"
            :class="[
              gameStore.isGameInitialized
                ? 'bg-amber-500 hover:bg-amber-400 text-slate-900'
                : 'bg-gray-600 text-gray-400 cursor-not-allowed'
            ]"
          >
            保存
          </button>
          <button
            @click="showLoadDialog = true"
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
          >
            加载
          </button>
        </div>
      </div>

      <div ref="storyScrollRef" class="flex-1 overflow-y-auto p-8 relative">
        <div class="max-w-3xl mx-auto space-y-4">
          <div v-if="gameStore.plotState && gameStore.currentScene" class="prose prose-invert max-w-none">
            <h2 class="text-2xl font-display text-amber-200 mb-4">
              {{ currentChapterTitle }}
            </h2>
            <div v-if="shouldShowRecap" class="mb-6 rounded-lg border border-amber-500/30 bg-slate-950/70 p-4">
              <p class="text-xs uppercase tracking-[0.25em] text-amber-200/70">上一章摘要</p>
              <p class="mt-2 text-sm text-slate-300 font-story whitespace-pre-wrap">
                {{ lastChapterSummary }}
              </p>
            </div>
            <VirtualStoryList
              :paragraphs="currentChapterParagraphs"
              :scroll-element="storyScrollRef"
            />
            <p
              v-if="optionSourceLabel"
              class="mt-3 text-xs text-slate-500 font-mono"
            >
              选项来源：{{ optionSourceLabel }}
            </p>
          </div>

          <div v-if="!gameStore.isGameInitialized" class="text-center text-gray-400">
            <p>当前没有进行中的游戏，请先开始新游戏。</p>
          </div>
        </div>

        <div class="pointer-events-none absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-slate-950 to-transparent"></div>
      </div>

      <div class="border-t border-slate-700 bg-slate-900/80 p-6 backdrop-blur">
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
                    : 'border-amber-400/60 bg-slate-800/80 hover:bg-slate-700 cursor-pointer'
                ]"
              >
                <p class="text-slate-100">{{ option.description }}</p>
                <p v-if="option.requirements && option.requirements.length > 0" class="text-sm text-slate-400 mt-1">
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
                class="w-full bg-slate-800 border border-slate-700 rounded-lg p-3 text-white outline-none focus:border-amber-400"
              />
              <p v-if="inputValidation.message" class="text-sm" :class="inputValidation.valid ? 'text-gray-300' : 'text-amber-300'">
                {{ inputValidation.message }}
              </p>
              <button
                @click="handleFreeTextSubmit"
                :disabled="isLoading || !inputValidation.valid"
                class="px-4 py-2 rounded-lg transition-colors"
                :class="isLoading || !inputValidation.valid ? 'bg-gray-600 text-gray-400 cursor-not-allowed' : 'bg-amber-500 hover:bg-amber-400 text-slate-900'"
              >
                提交自由输入
              </button>
            </div>
          </div>

          <div v-else-if="isLoading" class="text-center">
            <LoadingIndicator :message="loadingMessage" detail="请稍候，剧情正在推进..." size="lg" />
          </div>

          <div v-else-if="gameStore.isGameInitialized && !gameStore.isWaitingForInput" class="text-center">
            <button
              @click="handleContinue"
              class="px-4 py-2 rounded-lg transition-colors bg-amber-500 hover:bg-amber-400 text-slate-900"
            >
              继续写
            </button>
          </div>
        </div>
      </div>

      <div class="border-t border-slate-800 bg-slate-950/70 p-6">
        <div class="max-w-3xl mx-auto">
          <NovelExporter
            :is-game-running="gameStore.isGameInitialized"
            :event-count="gameStore.gameState?.event_history?.length ?? 0"
          />
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
    <StorySettingsDialog
      :is-open="showStorySettings"
      :settings="storySettings"
      @close="showStorySettings = false"
      @save="applyStorySettings"
    />
    <div
      v-if="showCharacterInfo"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="showCharacterInfo = false"
    >
      <div class="w-full max-w-md">
        <CharacterPanel :character="gameStore.playerCharacter" />
        <div class="mt-3 text-right">
          <button
            class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
            @click="showCharacterInfo = false"
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import AudioControlPanel from './AudioControlPanel.vue';
import CharacterPanel from './CharacterPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import LoadingIndicator from './LoadingIndicator.vue';
import NovelExporter from './NovelExporter.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import StorySettingsDialog from './StorySettingsDialog.vue';
import type { PlayerOption } from '../types/game';
import {
  createFreeTextAction,
  createOptionAction,
  createContinueAction,
  validateFreeTextInput,
} from '../utils/playerInput';
import { playClick } from '../utils/audioSystem';
import { getStorySettings, saveStorySettings, type StorySettings } from '../utils/storySettings';
import { invokeWithTimeout } from '../utils/tauriInvoke';
import VirtualStoryList from './VirtualStoryList.vue';

const router = useRouter();
const gameStore = useGameStore();

const isLoading = ref(false);
const loadingMessage = ref('处理中...');
const showSaveDialog = ref(false);
const showLoadDialog = ref(false);
const showLLMDialog = ref(false);
const showAudioPanel = ref(false);
const showStorySettings = ref(false);
const showCharacterInfo = ref(false);
const storySettings = ref<StorySettings>(getStorySettings());
const inputMode = ref<'options' | 'freeText'>('options');
const freeTextInput = ref('');
const storyScrollRef = ref<HTMLElement | null>(null);

const inputValidation = computed(() => validateFreeTextInput(freeTextInput.value));
const currentChapterTitle = computed(
  () => gameStore.plotState?.current_chapter?.title || gameStore.currentScene?.name || '第一章'
);
const currentChapterContent = computed(() => {
  const content = gameStore.plotState?.current_chapter?.content ?? [];
  if (content.length > 0) {
    return content.join('\n\n');
  }
  return gameStore.currentScene?.description ?? '';
});
const currentChapterParagraphs = computed(() => {
  const content = gameStore.plotState?.current_chapter?.content ?? [];
  const combined = content.length > 0 ? content.join('\n\n') : gameStore.currentScene?.description ?? '';
  return combined
    .split(/\n{2,}/)
    .map((text) => text.trim())
    .filter((text) => text.length > 0);
});
const lastChapterSummary = computed(() => {
  const chapters = gameStore.plotState?.chapters ?? [];
  return chapters.length > 0 ? chapters[chapters.length - 1]?.summary ?? '' : '';
});
const shouldShowRecap = computed(
  () => storySettings.value.recap_enabled && lastChapterSummary.value.length > 0
);
const optionSourceLabel = computed(() => {
  const source = gameStore.plotState?.last_option_generation_source;
  if (!source) {
    return '';
  }
  const labels: Record<string, string> = {
    llm_structured: 'LLM-结构化',
    llm_regenerated: 'LLM-再生成',
    rule_fallback: '规则回退',
    previous_reused: '复用上一组选项',
    not_waiting_for_input: '当前无需输入',
  };
  return labels[source] ?? source;
});

const handleOptionSelect = async (option: PlayerOption) => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在执行选项...';
    playClick();
    await gameStore.executePlayerAction(createOptionAction(option));
  } catch (error) {
    console.error('执行行动失败：', error);
  } finally {
    isLoading.value = false;
    loadingMessage.value = '处理中...';
  }
};

const handleFreeTextSubmit = async () => {
  const check = validateFreeTextInput(freeTextInput.value);
  if (!check.valid) {
    return;
  }

  try {
    isLoading.value = true;
    loadingMessage.value = '正在解析输入...';
    playClick();
    await gameStore.executePlayerAction(createFreeTextAction(freeTextInput.value));
    freeTextInput.value = '';
  } catch (error) {
    console.error('提交自由输入失败：', error);
  } finally {
    isLoading.value = false;
    loadingMessage.value = '处理中...';
  }
};

const handleContinue = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在续写剧情...';
    playClick();
    await gameStore.executePlayerAction(createContinueAction());
  } catch (error) {
    console.error('继续写失败：', error);
  } finally {
    isLoading.value = false;
    loadingMessage.value = '处理中...';
  }
};

const handleSaved = (slotId: number) => {
  console.log(`游戏已保存到槽位 ${slotId}`);
};

const handleLoaded = (slotId: number) => {
  console.log(`已从槽位 ${slotId} 加载游戏`);
};

const toggleAudioPanel = () => {
  playClick();
  showAudioPanel.value = !showAudioPanel.value;
};

const applyStorySettings = async (settings: StorySettings) => {
  storySettings.value = settings;
  saveStorySettings(settings);
  try {
    await invokeWithTimeout(
      'update_plot_settings',
      {
        settings,
      },
      8000,
      '更新剧情设置超时，请稍后重试',
    );
  } catch (error) {
    console.error('更新剧情设置失败：', error);
  }
};

watchEffect(() => {
  if (gameStore.isPlotInitialized) {
    void applyStorySettings(storySettings.value);
  }
});
</script>
