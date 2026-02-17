<template>
  <div class="min-h-screen text-white flex flex-col">
    <div class="flex-1 flex flex-col">
      <div class="bg-slate-900/80 border-b border-slate-700 px-6 py-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between backdrop-blur">
        <div class="flex items-center gap-4">
          <button
            class="text-gray-400 hover:text-white transition-colors"
            title="返回"
            @click="router.push('/')"
          >
            <svg
              class="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10 19l-7-7m0 0l7-7m-7 7h18"
              />
            </svg>
          </button>
          <h1 class="text-xl font-display text-amber-200">
            Nobody
          </h1>
        </div>
        <div class="flex flex-wrap gap-2">
          <div class="relative">
            <button
              class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
              @click="toggleAudioPanel"
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
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
            title="查看快捷键"
            @click="showShortcutsDialog = true"
          >
            ⌨️
          </button>
          <button
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-slate-900 rounded-lg transition-colors duration-200"
            @click="showLLMDialog = true"
          >
            LLM 设置
          </button>
          <button
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
            @click="showStorySettings = true"
          >
            剧情设置
          </button>
          <button
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
            @click="showCharacterInfo = true"
          >
            角色信息
          </button>
          <button
            :disabled="!gameStore.isGameInitialized"
            class="px-4 py-2 rounded-lg transition-colors duration-200"
            :class="[
              gameStore.isGameInitialized
                ? 'bg-amber-500 hover:bg-amber-400 text-slate-900'
                : 'bg-gray-600 text-gray-400 cursor-not-allowed'
            ]"
            @click="showSaveDialog = true"
          >
            保存
          </button>
          <button
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors duration-200"
            @click="showLoadDialog = true"
          >
            加载
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-hidden p-4 sm:p-6 lg:p-8">
        <div class="mx-auto grid h-full max-w-7xl gap-6 xl:grid-cols-[minmax(0,3fr)_minmax(280px,1fr)]">
          <div class="flex min-h-0 flex-col rounded-2xl border border-slate-800/90 bg-slate-950/45">
            <div
              ref="storyScrollRef"
              class="relative flex-1 overflow-y-auto p-6 sm:p-8"
            >
              <button
                v-if="gameStore.isGameInitialized"
                class="absolute right-4 bottom-24 z-10 p-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-white shadow-lg transition-colors"
                title="滚动到底部"
                @click="scrollToBottom"
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 14l-7 7m0 0l-7-7m7 7V3"
                  />
                </svg>
              </button>
              <div class="mx-auto max-w-3xl space-y-4">
                <div
                  v-if="gameStore.plotState && gameStore.currentScene"
                  class="prose prose-invert max-w-none"
                >
                  <h2 class="text-2xl font-display text-amber-200 mb-4">
                    {{ currentChapterTitle }}
                  </h2>
                  <div
                    v-if="shouldShowRecap"
                    class="mb-6 rounded-lg border border-amber-500/30 bg-slate-950/70 p-4"
                  >
                    <p class="text-xs uppercase tracking-[0.25em] text-amber-200/70">
                      上一章摘要
                    </p>
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

                <div
                  v-if="!gameStore.isGameInitialized"
                  class="text-center text-gray-400"
                >
                  <p>当前没有进行中的游戏，请先开始新游戏。</p>
                </div>
              </div>
              <div class="pointer-events-none absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-slate-950 to-transparent" />
            </div>

            <div class="border-t border-slate-700 bg-slate-900/80 p-6 backdrop-blur">
              <div class="mx-auto max-w-3xl">
                <div
                  v-if="shouldShowInputPanel"
                  class="space-y-4"
                >
                  <div v-if="gameStore.availableOptions.length > 0" class="flex items-center gap-2">
                    <button
                      class="px-3 py-1 rounded"
                      :class="inputMode === 'options' ? 'bg-purple-600 text-white' : 'bg-slate-700 text-gray-300'"
                      @click="inputMode = 'options'"
                    >
                      选项
                    </button>
                    <button
                      class="px-3 py-1 rounded"
                      :class="inputMode === 'freeText' ? 'bg-purple-600 text-white' : 'bg-slate-700 text-gray-300'"
                      @click="inputMode = 'freeText'"
                    >
                      自由输入
                    </button>
                  </div>

                  <div
                    v-if="inputMode === 'options' && gameStore.availableOptions.length > 0"
                    class="space-y-2"
                  >
                    <button
                      v-for="(option, index) in gameStore.availableOptions"
                      :key="index"
                      :disabled="isLoading"
                      class="w-full text-left p-4 rounded-lg border-2 transition-all duration-200"
                      :class="[
                        isLoading
                          ? 'border-gray-600 bg-slate-700 opacity-50 cursor-not-allowed'
                          : 'border-amber-400/60 bg-slate-800/80 hover:bg-slate-700 cursor-pointer'
                      ]"
                      @click="handleOptionSelect(option)"
                    >
                      <p class="text-slate-100">
                        {{ option.description }}
                      </p>
                      <p
                        v-if="option.requirements && option.requirements.length > 0"
                        class="text-sm text-slate-400 mt-1"
                      >
                        条件：{{ option.requirements.join('，') }}
                      </p>
                    </button>
                  </div>

                  <div
                    v-if="inputMode === 'freeText' || gameStore.availableOptions.length === 0"
                    class="space-y-2"
                  >
                    <textarea
                      v-model="freeTextInput"
                      :disabled="isLoading"
                      rows="3"
                      maxlength="200"
                      placeholder="输入你想执行的行为，例如：我去后山修炼。"
                      class="w-full bg-slate-800 border border-slate-700 rounded-lg p-3 text-white outline-none focus:border-amber-400"
                    />
                    <p
                      v-if="inputValidation.message"
                      class="text-sm"
                      :class="inputValidation.valid ? 'text-gray-300' : 'text-amber-300'"
                    >
                      {{ inputValidation.message }}
                    </p>
                    <button
                      :disabled="isLoading || !inputValidation.valid"
                      class="px-4 py-2 rounded-lg transition-colors"
                      :class="isLoading || !inputValidation.valid ? 'bg-gray-600 text-gray-400 cursor-not-allowed' : 'bg-amber-500 hover:bg-amber-400 text-slate-900'"
                      @click="handleFreeTextSubmit"
                    >
                      提交自由输入
                    </button>
                  </div>
                </div>

                <div
                  v-else-if="isNoInputAdvanceState && !isLoading"
                  class="text-center space-y-3"
                >
                  <p class="text-sm text-slate-400">当前无需输入，点击继续即可推进剧情。</p>
                  <button
                    class="px-4 py-2 rounded-lg transition-colors bg-amber-500 hover:bg-amber-400 text-slate-900"
                    @click="handleContinue"
                  >
                    继续推进剧情
                  </button>
                </div>

                <div
                  v-else-if="isLoading"
                  class="text-center"
                >
                  <LoadingIndicator
                    :message="loadingMessage"
                    detail="请稍候，剧情正在推进..."
                    size="lg"
                  />
                </div>

                <div
                  v-else-if="gameStore.isGameInitialized && !gameStore.isWaitingForInput"
                  class="text-center"
                >
                  <button
                    class="px-4 py-2 rounded-lg transition-colors bg-amber-500 hover:bg-amber-400 text-slate-900"
                    @click="handleContinue"
                  >
                    继续写
                  </button>
                </div>
              </div>
            </div>
          </div>

          <aside
            data-testid="game-sidebar"
            class="space-y-4 overflow-y-auto rounded-2xl border border-slate-800/90 bg-slate-950/40 p-4 sm:p-5"
          >
            <section class="rounded-xl border border-slate-700/70 bg-slate-900/60 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-slate-500">角色快照</p>
              <div class="mt-3 space-y-2 text-sm text-slate-200">
                <p>姓名：{{ gameStore.playerCharacter?.name || '无名弟子' }}</p>
                <p>境界：{{ playerRealmLabel }}</p>
                <p>战力：{{ playerCombatPowerLabel }}</p>
                <p>位置：{{ gameStore.playerCharacter?.location || gameStore.currentScene?.location || '未知' }}</p>
              </div>
            </section>

            <section class="rounded-xl border border-slate-700/70 bg-slate-900/60 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-slate-500">剧情进度</p>
              <div class="mt-3 space-y-2 text-sm text-slate-200">
                <p>章节：{{ chapterProgressLabel }}</p>
                <p>章节互动：{{ chapterInteractionLabel }}</p>
                <p>剧情段落：{{ gameStore.plotState?.segment_count ?? 0 }}</p>
                <p>当前状态：{{ gameStore.isWaitingForInput ? '等待玩家输入' : '自动推进中' }}</p>
              </div>
            </section>

            <section class="rounded-xl border border-slate-700/70 bg-slate-900/60 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-slate-500">经历导出</p>
              <div class="mt-3">
                <NovelExporter
                  :is-game-running="gameStore.isGameInitialized"
                  :event-count="gameStore.gameState?.event_history?.length ?? 0"
                />
              </div>
            </section>
          </aside>
        </div>
      </div>

      <div
        v-if="isDevMode && gameStore.plotState"
        class="border-t border-slate-800 bg-slate-950/80 p-4"
      >
        <div class="mx-auto max-w-3xl space-y-2 text-xs text-slate-300">
          <p class="uppercase tracking-[0.2em] text-slate-500">Debug Context</p>
          <p>章节：{{ gameStore.plotState.current_chapter.index }} / {{ gameStore.plotState.current_chapter.title }}</p>
          <p>选项来源：{{ optionSourceLabel || 'n/a' }}</p>
          <p>等待输入：{{ gameStore.isWaitingForInput ? 'yes' : 'no' }}</p>
          <p class="whitespace-pre-wrap text-slate-400">
            诊断：{{ gameStore.plotState.last_generation_diagnostics || '无' }}
          </p>
        </div>
      </div>

      <div
        v-if="gameStore.error"
        class="p-4 bg-red-900 bg-opacity-50 border-t border-red-500"
      >
        <div class="max-w-3xl mx-auto">
          <StatusBanner
            kind="error"
            title="系统提示"
            :message="gameStore.error ?? ''"
          />
          <button
            class="mt-2 px-4 py-1 bg-red-700 hover:bg-red-600 rounded text-sm transition-colors"
            @click="gameStore.clearError"
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

    <KeyboardShortcutsDialog
      :is-open="showShortcutsDialog"
      @close="showShortcutsDialog = false"
    />
    <LLMConfigDialog
      :is-open="showLLMDialog"
      @close="showLLMDialog = false"
    />
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
import { computed, ref, watchEffect, onMounted, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import AudioControlPanel from './AudioControlPanel.vue';
import CharacterPanel from './CharacterPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import KeyboardShortcutsDialog from './KeyboardShortcutsDialog.vue';
import LoadingIndicator from './LoadingIndicator.vue';
import NovelExporter from './NovelExporter.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import StatusBanner from './StatusBanner.vue';
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
const showShortcutsDialog = ref(false);
const storySettings = ref<StorySettings>(getStorySettings());
const inputMode = ref<'options' | 'freeText'>('options');
const freeTextInput = ref('');
const storyScrollRef = ref<HTMLElement | null>(null);
const previousChapterParagraphs = ref<string[]>([]);
const isDevMode = import.meta.env.DEV;
const MAX_AUTO_ADVANCE_STEPS = 12;

const inputValidation = computed(() => validateFreeTextInput(freeTextInput.value));
const currentChapterTitle = computed(
  () => gameStore.plotState?.current_chapter?.title || gameStore.currentScene?.name || '第一章'
);
const playerRealmLabel = computed(() => {
  const realm = gameStore.playerCharacter?.stats?.cultivation_realm;
  if (!realm) {
    return '凡人';
  }
  return `${realm.name} (${realm.level}-${realm.sub_level})`;
});
const playerCombatPowerLabel = computed(() => {
  const power = gameStore.playerCharacter?.stats?.combat_power;
  return typeof power === 'number' ? power.toLocaleString() : '未知';
});
const chapterProgressLabel = computed(() => {
  const chapter = gameStore.plotState?.current_chapter;
  if (!chapter) {
    return '0 / 无';
  }
  return `${chapter.index} / ${chapter.title || '未命名章节'}`;
});
const chapterInteractionLabel = computed(() => {
  const chapter = gameStore.plotState?.current_chapter;
  if (!chapter) {
    return '0 / 0-0';
  }
  const min = gameStore.plotState?.settings?.min_interactions_per_chapter ?? 0;
  const max = gameStore.plotState?.settings?.max_interactions_per_chapter ?? 0;
  return `${chapter.interaction_count} / ${min}-${max}`;
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
const isNoInputAdvanceState = computed(
  () =>
    gameStore.isGameInitialized &&
    gameStore.isWaitingForInput &&
    gameStore.availableOptions.length === 0 &&
    gameStore.plotState?.last_option_generation_source === 'not_waiting_for_input'
);
const shouldShowInputPanel = computed(
  () => gameStore.isWaitingForInput && gameStore.isGameInitialized && !isNoInputAdvanceState.value
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
    let step = 0;
    do {
      step += 1;
      if (step > 1) {
        loadingMessage.value = `正在自动推进剧情（${step}）...`;
      }
      await gameStore.executePlayerAction(createContinueAction());
    } while (isNoInputAdvanceState.value && step < MAX_AUTO_ADVANCE_STEPS);

    if (isNoInputAdvanceState.value) {
      console.warn(`自动推进已达到上限（${MAX_AUTO_ADVANCE_STEPS} 步），请检查剧情生成状态。`);
    }
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

const scrollToBottom = () => {
  if (storyScrollRef.value && typeof storyScrollRef.value.scrollTo === 'function') {
    storyScrollRef.value.scrollTo({
      top: storyScrollRef.value.scrollHeight,
      behavior: 'smooth'
    });
  }
};

watchEffect(() => {
  if (gameStore.isPlotInitialized) {
    void applyStorySettings(storySettings.value);
  }
});

// 监听章节内容变化，自动滚动到底部
watch(currentChapterParagraphs, (newParagraphs) => {
  if (newParagraphs.length > previousChapterParagraphs.value.length && storyScrollRef.value) {
    // 只有当有新内容时才滚动到底部
    requestAnimationFrame(() => {
      if (!storyScrollRef.value || typeof storyScrollRef.value.scrollTo !== 'function') {
        return;
      }
      storyScrollRef.value.scrollTo({
        top: storyScrollRef.value.scrollHeight,
        behavior: 'smooth'
      });
    });
  }
  previousChapterParagraphs.value = [...newParagraphs];
}, { deep: true });

// 键盘快捷键支持
const handleKeydown = (event: KeyboardEvent) => {
  // 只有在游戏初始化且不在输入框时才响应快捷键
  if (!gameStore.isGameInitialized || (event.target instanceof HTMLElement && event.target.tagName === 'TEXTAREA')) {
    return;
  }

  // ESC 关闭所有弹窗
  if (event.key === 'Escape') {
    showSaveDialog.value = false;
    showLoadDialog.value = false;
    showLLMDialog.value = false;
    showStorySettings.value = false;
    showCharacterInfo.value = false;
    showAudioPanel.value = false;
  }

  // Enter 提交自由输入
  if (event.key === 'Enter' && isNoInputAdvanceState.value) {
    event.preventDefault();
    handleContinue();
    return;
  }

  // Enter 提交自由输入
  if (event.key === 'Enter' && inputMode.value === 'freeText' && freeTextInput.value.trim()) {
    event.preventDefault();
    handleFreeTextSubmit();
  }

  // 1-5 数字键快速选择选项
  if (inputMode.value === 'options' && gameStore.availableOptions.length > 0) {
    const num = parseInt(event.key);
    if (num >= 1 && num <= 5 && num <= gameStore.availableOptions.length) {
      event.preventDefault();
      const option = gameStore.availableOptions[num - 1];
      handleOptionSelect(option);
    }
  }

  // Ctrl+S / Cmd+S 快速保存
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    if (gameStore.isGameInitialized) {
      showSaveDialog.value = true;
    }
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>
