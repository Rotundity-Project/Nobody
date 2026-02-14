<template>
  <div class="min-h-screen text-white flex flex-col relative">
    <!-- 背景光效 -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <div class="absolute top-20 right-20 w-96 h-96 bg-amber-500/5 rounded-full blur-3xl animate-pulse"></div>
      <div class="absolute bottom-20 left-20 w-80 h-80 bg-emerald-500/5 rounded-full blur-3xl animate-pulse" style="animation-delay: 1s;"></div>
    </div>

    <div class="flex-1 flex flex-col relative z-10">
      <!-- 顶部导航栏 -->
      <div class="bg-gradient-to-b from-slate-900/95 to-slate-900/80 border-b border-slate-700/50 px-4 sm:px-6 py-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between backdrop-blur-sm">
        <div class="flex items-center gap-3 sm:gap-4">
          <button
            @click="router.push('/')"
            class="p-2 text-gray-400 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all duration-300 tooltip"
            data-tooltip="返回主菜单"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <div class="flex items-center gap-2">
            <h1 class="text-xl sm:text-2xl font-display text-glow gradient-text">Nobody</h1>
          </div>
        </div>

        <div class="flex flex-wrap gap-2 sm:gap-3">
          <div class="relative">
            <button
              @click="toggleAudioPanel"
              class="px-3 sm:px-4 py-2 btn-secondary text-white rounded-lg text-sm sm:text-base transition-all duration-300"
            >
              <span class="hidden sm:inline">🔊 音量</span>
              <span class="sm:hidden">🔊</span>
            </button>
            <Transition name="fade-slide">
              <div
                v-if="showAudioPanel"
                class="absolute right-0 mt-3 w-64 panel-surface rounded-xl p-4 z-50"
              >
                <AudioControlPanel />
              </div>
            </Transition>
          </div>

          <button
            @click="showLLMDialog = true"
            class="px-3 sm:px-4 py-2 btn-emerald text-slate-900 rounded-lg text-sm sm:text-base font-medium transition-all duration-300"
          >
            <span class="hidden sm:inline">LLM 设置</span>
            <span class="sm:hidden">设置</span>
          </button>

          <button
            @click="showStorySettings = true"
            class="px-3 sm:px-4 py-2 btn-secondary text-white rounded-lg text-sm sm:text-base transition-all duration-300"
          >
            剧情
          </button>

          <button
            @click="showCharacterInfo = true"
            class="px-3 sm:px-4 py-2 btn-secondary text-white rounded-lg text-sm sm:text-base transition-all duration-300"
          >
            角色
          </button>

          <button
            @click="showSaveDialog = true"
            :disabled="!gameStore.isGameInitialized"
            class="px-3 sm:px-4 py-2 rounded-lg text-sm sm:text-base transition-all duration-300"
            :class="[
              gameStore.isGameInitialized
                ? 'btn-primary text-slate-900 font-medium'
                : 'bg-gray-700 text-gray-400 cursor-not-allowed opacity-50'
            ]"
          >
            保存
          </button>

          <button
            @click="showLoadDialog = true"
            class="px-3 sm:px-4 py-2 btn-secondary text-white rounded-lg text-sm sm:text-base transition-all duration-300"
          >
            加载
          </button>
        </div>
      </div>

      <!-- 故事显示区域 -->
      <div ref="storyScrollRef" class="flex-1 overflow-y-auto p-4 sm:p-8 relative">
        <div class="max-w-3xl mx-auto space-y-4">
          <div v-if="gameStore.plotState && gameStore.currentScene" class="prose prose-invert max-w-none">
            <h2 class="text-2xl sm:text-3xl font-display text-glow mb-6 gradient-text">
              {{ currentChapterTitle }}
            </h2>

            <Transition name="fade-up">
              <div v-if="shouldShowRecap" class="mb-6 rounded-xl border border-amber-500/30 bg-gradient-to-br from-amber-950/70 to-amber-950/30 p-5 backdrop-blur-sm">
                <p class="text-xs uppercase tracking-[0.3em] text-amber-200/70 mb-2">上一章摘要</p>
                <p class="text-sm text-slate-200 font-story whitespace-pre-wrap leading-relaxed">
                  {{ lastChapterSummary }}
                </p>
              </div>
            </Transition>

            <VirtualStoryList
              :paragraphs="currentChapterParagraphs"
              :scroll-element="storyScrollRef"
            />

            <p
              v-if="optionSourceLabel"
              class="mt-4 text-xs text-slate-500 font-mono tracking-wide"
            >
              选项来源：{{ optionSourceLabel }}
            </p>
          </div>

          <div v-if="!gameStore.isGameInitialized" class="text-center text-gray-400 py-12">
            <p class="text-lg">当前没有进行中的游戏</p>
            <p class="text-sm mt-2">请返回主菜单开始新游戏</p>
          </div>
        </div>

        <!-- 底部渐变遮罩 -->
        <div class="pointer-events-none absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-slate-950 to-transparent"></div>
      </div>

      <!-- 底部输入区域 -->
      <div class="border-t border-slate-700/50 bg-gradient-to-t from-slate-900/95 to-slate-900/80 p-4 sm:p-6 backdrop-blur-sm">
        <div class="max-w-3xl mx-auto">
          <div v-if="gameStore.isWaitingForInput && gameStore.isGameInitialized" class="space-y-4">
            <!-- 输入模式切换 -->
            <div class="flex items-center gap-2">
              <button
                @click="inputMode = 'options'"
                class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-300"
                :class="inputMode === 'options' ? 'bg-gradient-to-r from-purple-600 to-purple-700 text-white shadow-lg shadow-purple-500/30' : 'bg-slate-700/50 text-gray-300 hover:bg-slate-600/50'"
              >
                选项
              </button>
              <button
                @click="inputMode = 'freeText'"
                class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-300"
                :class="inputMode === 'freeText' ? 'bg-gradient-to-r from-purple-600 to-purple-700 text-white shadow-lg shadow-purple-500/30' : 'bg-slate-700/50 text-gray-300 hover:bg-slate-600/50'"
              >
                自由输入
              </button>
            </div>

            <!-- 选项模式 -->
            <Transition name="fade-slide">
              <div v-if="inputMode === 'options' && gameStore.availableOptions.length > 0" class="space-y-3">
                <button
                  v-for="(option, index) in gameStore.availableOptions"
                  :key="index"
                  @click="handleOptionSelect(option)"
                  :disabled="isLoading"
                  class="w-full text-left p-4 sm:p-5 rounded-xl option-btn transition-all duration-300"
                  :class="isLoading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'"
                >
                  <p class="text-slate-100 font-medium leading-relaxed">{{ option.description }}</p>
                  <p v-if="option.requirements && option.requirements.length > 0" class="text-xs text-amber-300/80 mt-2 flex items-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    {{ option.requirements.join('，') }}
                  </p>
                </button>
              </div>
            </Transition>

            <!-- 自由输入模式 -->
            <Transition name="fade-slide">
              <div v-if="inputMode === 'freeText'" class="space-y-3">
                <textarea
                  v-model="freeTextInput"
                  :disabled="isLoading"
                  rows="4"
                  maxlength="200"
                  placeholder="输入你想执行的行为，例如：我去后山修炼..."
                  class="w-full input-field rounded-xl p-4 text-white placeholder:text-slate-500 resize-none transition-all duration-300"
                />
                <p v-if="inputValidation.message" class="text-sm" :class="inputValidation.valid ? 'text-gray-400' : 'text-amber-300'">
                  {{ inputValidation.message }}
                </p>
                <button
                  @click="handleFreeTextSubmit"
                  :disabled="isLoading || !inputValidation.valid"
                  class="w-full px-6 py-3 rounded-lg font-medium transition-all duration-300 text-base"
                  :class="isLoading || !inputValidation.valid ? 'bg-gray-700 text-gray-400 cursor-not-allowed' : 'btn-primary text-slate-900'"
                >
                  提交自由输入
                </button>
              </div>
            </Transition>
          </div>

          <!-- 加载状态 -->
          <div v-else-if="isLoading" class="text-center py-8">
            <LoadingIndicator :message="loadingMessage" detail="请稍候，剧情正在推进..." size="lg" />
          </div>

          <!-- 继续按钮 -->
          <div v-else-if="gameStore.isGameInitialized && !gameStore.isWaitingForInput" class="text-center py-4">
            <button
              @click="handleContinue"
              class="px-8 py-3 rounded-lg btn-primary text-slate-900 font-medium text-lg transition-all duration-300"
            >
              继续写
            </button>
          </div>
        </div>
      </div>

      <!-- 小说导出区域 -->
      <div class="border-t border-slate-800/50 bg-slate-950/70 p-4 sm:p-6">
        <div class="max-w-3xl mx-auto">
          <NovelExporter
            :is-game-running="gameStore.isGameInitialized"
            :event-count="gameStore.gameState?.event_history?.length ?? 0"
          />
        </div>
      </div>

      <!-- 错误提示 -->
      <Transition name="fade-up">
        <div v-if="gameStore.error" class="p-4 bg-gradient-to-r from-red-950/90 to-red-900/90 border-t border-red-500/50 backdrop-blur-sm">
          <div class="max-w-3xl mx-auto">
            <p class="text-red-200 font-medium">{{ gameStore.error }}</p>
            <button
              @click="gameStore.clearError"
              class="mt-3 px-4 py-2 rounded-lg bg-red-700/50 hover:bg-red-700 text-sm text-white transition-colors duration-300"
            >
              关闭
            </button>
          </div>
        </div>
      </Transition>
    </div>

    <!-- 对话框 -->
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

    <!-- 角色信息弹窗 -->
    <Transition name="fade">
      <div
        v-if="showCharacterInfo"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 modal-overlay"
        @click.self="showCharacterInfo = false"
      >
        <div class="w-full max-w-lg panel-surface rounded-2xl p-6 animate-fade-up">
          <CharacterPanel :character="gameStore.playerCharacter" />
          <div class="mt-6 flex justify-end">
            <button
              class="px-6 py-2 rounded-lg btn-secondary text-white transition-all duration-300"
              @click="showCharacterInfo = false"
            >
              关闭
            </button>
          </div>
        </div>
      </div>
    </Transition>
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

<style scoped>
/* 过渡动画 */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.fade-slide-enter-active, .fade-slide-leave-active {
  transition: all 0.3s ease;
}

.fade-slide-enter-from, .fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.fade-up-enter-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-up-leave-active {
  transition: all 0.3s ease;
}

.fade-up-enter-from, .fade-up-leave-to {
  opacity: 0;
  transform: translateY(20px);
}
</style>
