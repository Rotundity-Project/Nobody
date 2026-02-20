<template>
  <div class="min-h-screen text-white flex flex-col">
    <div class="flex-1 flex flex-col">
      <div class="bg-slate-900/80 border-b border-slate-700 px-3 py-2 sm:px-5 sm:py-2.5 md:px-6 xl:px-8 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between backdrop-blur">
        <GameTopBar
          :is-system-menu-open="showSystemMenu"
          :show-audio-panel="showAudioPanel"
          :is-game-initialized="gameStore.isGameInitialized"
          @back="handleBackToMenu"
          @toggle-menu="toggleSystemMenu"
          @close-menu="closeSystemMenu"
          @toggle-audio="toggleAudioPanel"
          @open-shortcuts="openShortcutsDialog"
          @open-llm="openLlmDialog"
          @open-story-settings="openStorySettingsDialog"
          @open-consistency="openConsistencySettingsFromMenu"
          @open-character="showCharacterInfo = true"
          @open-info="showInfoTabs = true"
          @open-save="showSaveDialog = true"
          @open-load="showLoadDialog = true"
        />
      </div>
      <div
        v-if="gameStore.isGameInitialized"
        class="border-b border-slate-800/80 bg-slate-900/60 px-3 py-1.5 text-[11px] text-slate-300 sm:hidden"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="min-w-0 flex-1 flex items-center gap-1.5">
            <p class="truncate">{{ mobileStatusSummary }}</p>
            <span
              v-if="optionSourceLabel"
              class="shrink-0 rounded border border-slate-700/70 bg-slate-900/70 px-1.5 py-0.5 text-[10px] text-slate-300"
            >
              {{ optionSourceLabel }}
            </span>
          </div>
          <button
            data-testid="toggle-mobile-status-card"
            class="shrink-0 rounded-md border border-slate-600 px-2 py-0.5 text-[10px] text-slate-200 transition-colors hover:bg-slate-800"
            :aria-expanded="showMobileStatusCard ? 'true' : 'false'"
            @click="toggleMobileStatusCard"
          >
            {{ showMobileStatusCard ? '收起' : '展开' }}
          </button>
        </div>
      </div>
      <ChapterStatusStrip
        :visible="gameStore.isGameInitialized"
        :chapter-progress="chapterProgressLabel"
        :chapter-interaction="chapterInteractionLabel"
        :interaction-state="interactionStateLabel"
        :option-source-label="optionSourceLabel"
        :option-source-hint="optionSourceHint || undefined"
        class="hidden sm:block"
      />
      <div
        v-if="gameStore.isGameInitialized && showMobileStatusCard"
        class="px-3 pb-1 pt-0.5 sm:hidden"
      >
        <ContextStatusCard
          :visible="true"
          :player-name="gameStore.playerCharacter?.name || '无名弟子'"
          :player-realm="playerRealmLabel"
          :chapter-progress="chapterProgressLabel"
          :chapter-interaction="chapterInteractionLabel"
          :location-label="currentLocationLabel"
          :interaction-state-label="interactionStateLabel"
        />
      </div>
      <div class="hidden px-3 pb-1 pt-1 sm:block sm:px-5 sm:pb-2 sm:pt-1.5 md:px-6 xl:px-8">
        <ContextStatusCard
          :visible="gameStore.isGameInitialized"
          :player-name="gameStore.playerCharacter?.name || '无名弟子'"
          :player-realm="playerRealmLabel"
          :chapter-progress="chapterProgressLabel"
          :chapter-interaction="chapterInteractionLabel"
          :location-label="currentLocationLabel"
          :interaction-state-label="interactionStateLabel"
        />
      </div>

      <div class="flex-1 overflow-hidden p-2 sm:p-4 md:p-5 xl:p-6">
        <div class="mx-auto h-full max-w-7xl">
          <div class="flex h-full min-h-0 flex-col rounded-2xl border border-slate-800/90 bg-slate-950/45">
            <StoryViewport
              ref="storyViewportRef"
              :has-scene="Boolean(gameStore.plotState && gameStore.currentScene)"
              :chapter-title="currentChapterTitle"
              :show-recap="shouldShowRecap"
              :recap-summary="lastChapterSummary"
              :paragraphs="currentChapterParagraphs"
              :option-source-label="optionSourceLabel"
              :is-game-initialized="gameStore.isGameInitialized"
            />

            <div class="border-t border-slate-700 bg-slate-900/80 p-4 sm:p-5 md:p-6 backdrop-blur">
              <GameInteractionPanel
                :should-show-input-panel="shouldShowInputPanel"
                :error="gameStore.error"
                :is-no-input-advance-state="isNoInputAdvanceState"
                :available-options="gameStore.availableOptions"
                :input-mode="inputMode"
                :is-loading="isLoading"
                :free-text-input="freeTextInput"
                :input-validation="inputValidation"
                :is-game-initialized="gameStore.isGameInitialized"
                :is-waiting-for-input="gameStore.isWaitingForInput"
                :loading-message="loadingMessage"
                :can-stop-auto-advance="autoAdvanceRunning"
                :auto-advance-stop-hint="autoAdvanceStopHint"
                @switch-mode="setInputMode"
                @select-option="handleOptionSelect"
                @update:free-text-input="freeTextInput = $event"
                @submit-free-text="handleFreeTextSubmit"
                @continue="handleContinue"
                @stop-auto-advance="requestStopAutoAdvance"
              />
            </div>
        </div>
      </div>
    </div>

    <GameSystemDialogs
      :show-save-dialog="showSaveDialog"
      :show-load-dialog="showLoadDialog"
      :show-shortcuts-dialog="showShortcutsDialog"
      :show-l-l-m-dialog="showLLMDialog"
      :show-story-settings="showStorySettings"
      :show-consistency-settings="showConsistencySettings"
      :story-settings="storySettings"
      :consistency-policy="consistencyPolicy"
      @close-save="showSaveDialog = false"
      @saved="handleSaved"
      @close-load="showLoadDialog = false"
      @loaded="handleLoaded"
      @close-shortcuts="showShortcutsDialog = false"
      @close-llm="showLLMDialog = false"
      @close-story-settings="showStorySettings = false"
      @save-story-settings="applyStorySettings"
      @close-consistency="showConsistencySettings = false"
      @save-consistency="applyConsistencyPolicy"
      @reset-consistency="resetConsistencyPolicy"
    />
    <GameInfoCenterDialog
      :is-open="showInfoTabs"
      :game-store="gameStore"
      :player-realm-label="playerRealmLabel"
      :player-combat-power-label="playerCombatPowerLabel"
      :chapter-progress-label="chapterProgressLabel"
      :chapter-interaction-label="chapterInteractionLabel"
      :world-location-list="worldLocationList"
      :recent-combat-review="recentCombatReview"
      :travel-pending="travelPending"
      :is-dev-mode="isDevMode"
      :option-source-label="optionSourceLabel"
      :option-source-hint="optionSourceHint || undefined"
      :consistency-risk-score="consistencyRiskScore"
      @close="showInfoTabs = false"
      @clear-error="gameStore.clearError()"
      @travel="handleTravel"
    />
    <CharacterInfoModal
      :is-open="showCharacterInfo"
      :character="gameStore.playerCharacter"
      @close="showCharacterInfo = false"
    />
    <NotificationCenter
      v-if="runtimeNotifications.length > 0"
      :notifications="runtimeNotifications"
      @dismiss="dismissRuntimeNotification"
    />
  </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watchEffect, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import CharacterInfoModal from './CharacterInfoModal.vue';
import ChapterStatusStrip from './ChapterStatusStrip.vue';
import ContextStatusCard from './ContextStatusCard.vue';
import GameInfoCenterDialog from './GameInfoCenterDialog.vue';
import GameTopBar from './GameTopBar.vue';
import GameInteractionPanel from './GameInteractionPanel.vue';
import GameSystemDialogs from './GameSystemDialogs.vue';
import NotificationCenter, { type NotificationItem } from './NotificationCenter.vue';
import StoryViewport from './StoryViewport.vue';
import type { ConsistencyPolicy } from '../types/game';
import {
  createFreeTextAction,
  createOptionAction,
  createContinueAction,
  validateFreeTextInput,
} from '../utils/playerInput';
import { useInputMode } from '../composables/useInputMode';
import { useGameHotkeys } from '../composables/useGameHotkeys';
import { useStoryFlow } from '../composables/useStoryFlow';
import { useUiPanels } from '../composables/useUiPanels';
import { playClick } from '../utils/audioSystem';
import { getStorySettings, saveStorySettings, type StorySettings } from '../utils/storySettings';
import { invokeWithTimeout } from '../utils/tauriInvoke';
import { buildLocationLabelMap, formatLocationLabel } from '../shared/locationLabel';

const router = useRouter();
const gameStore = useGameStore();

const {
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showAudioPanel,
  showStorySettings,
  showInfoTabs,
  showConsistencySettings,
  showSystemMenu,
  showCharacterInfo,
  showShortcutsDialog,
  closeSystemMenu,
  closeAllDialogs,
} = useUiPanels();
const storySettings = ref<StorySettings>(getStorySettings());
const consistencyPolicy = ref<ConsistencyPolicy>({
  recent_window: 3,
  cross_chapter_window: 3,
  duplicate_recent_threshold: 0.92,
  duplicate_cross_chapter_threshold: 0.88,
  weight_warning: 5,
  weight_critical: 12,
  code_weights: {},
});
const {
  inputMode,
  freeTextInput,
  inputValidation,
  setInputMode,
} = useInputMode(validateFreeTextInput);
const storyViewportRef = ref<{ scrollToBottom: () => void } | null>(null);
const previousChapterParagraphs = ref<string[]>([]);
const isDevMode = import.meta.env.DEV;
const travelPending = ref(false);
const showMobileStatusCard = ref(false);
const MOBILE_STATUS_CARD_STORAGE_KEY = 'nobody_mobile_status_card_expanded';

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
const locationNameMap = computed(() =>
  buildLocationLabelMap(
    (gameStore.gameState?.script?.world_setting?.locations ?? []).map((loc) => ({
      id: loc.id,
      name: loc.name,
    })),
  ),
);
const currentLocationLabel = computed(() =>
  formatLocationLabel(
    gameStore.playerCharacter?.location || gameStore.currentScene?.location,
    locationNameMap.value,
  ),
);
const worldLocationList = computed(() => {
  return (gameStore.gameState?.script?.world_setting?.locations ?? []).map((loc) => ({
    id: loc.id,
    name: loc.name,
    spiritual_energy: loc.spiritual_energy,
  }));
});
const recentCombatReview = computed(() => {
  const events = gameStore.gameState?.event_history ?? [];
  return events
    .filter((event) =>
      event.event_type === 'combat_explanation'
      || event.event_type === 'encounter'
      || event.event_type === 'combat')
    .slice(-6)
    .reverse()
    .map((event) => `[t=${event.timestamp}] ${event.description}`);
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
const plotInteractionState = computed(() => {
  if (gameStore.plotState?.interaction_state) {
    return gameStore.plotState.interaction_state;
  }
  if (!gameStore.isGameInitialized) {
    return 'auto_advance';
  }
  if (!gameStore.isWaitingForInput) {
    return 'auto_advance';
  }
  return gameStore.availableOptions.length > 0 ? 'waiting_for_choice' : 'waiting_for_free_text';
});
const isNoInputAdvanceState = computed(
  () =>
    gameStore.isGameInitialized &&
    (
      plotInteractionState.value === 'cooldown' ||
      gameStore.plotState?.last_option_generation_source === 'not_waiting_for_input' ||
      gameStore.plotState?.last_option_generation_source === 'consistency_non_waiting_fallback'
    )
);
const shouldShowInputPanel = computed(
  () =>
    gameStore.isGameInitialized &&
    !isNoInputAdvanceState.value &&
    (plotInteractionState.value === 'waiting_for_choice'
      || plotInteractionState.value === 'waiting_for_free_text')
);
const shouldAutoAdvance = computed(
  () =>
    gameStore.isGameInitialized &&
    (!gameStore.isWaitingForInput
      || plotInteractionState.value === 'cooldown'
      || isNoInputAdvanceState.value)
);
const hasBlockingOverlay = computed(
  () =>
    showSaveDialog.value
    || showLoadDialog.value
    || showLLMDialog.value
    || showStorySettings.value
    || showConsistencySettings.value
    || showInfoTabs.value
    || showCharacterInfo.value
    || showShortcutsDialog.value,
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
    rule_fallback_latency_budget: '规则回退（时延预算）',
    previous_reused: '复用上一组选项',
    not_waiting_for_input: '当前无需输入',
    consistency_non_waiting_fallback: '一致性兜底自动推进',
  };
  return labels[source] ?? source;
});
const optionSourceHint = computed(() => {
  const source = gameStore.plotState?.last_option_generation_source ?? '';
  const diag = gameStore.plotState?.last_generation_diagnostics ?? '';
  if (source === 'rule_fallback_latency_budget') {
    return '受时延预算影响，已跳过 LLM 选项再生成';
  }
  if (diag.includes('skipped(latency_budget)')) {
    return '部分增强步骤因时延预算被跳过';
  }
  return '';
});
const handleBackToMenu = () => {
  router.push('/');
};
const interactionStateLabel = computed(() => {
  const mapping: Record<string, string> = {
    auto_advance: '自动推进',
    waiting_for_choice: '等待选项',
    waiting_for_free_text: '等待自由输入',
    resolving: '处理中',
    cooldown: '冷却阶段',
  };
  return mapping[plotInteractionState.value] ?? plotInteractionState.value;
});
const mobileStatusSummary = computed(
  () => `${chapterProgressLabel.value} · ${interactionStateLabel.value}`,
);
const consistencyRiskScore = computed(() => {
  const structured = gameStore.plotState?.last_consistency_risk_score;
  if (typeof structured === 'number') {
    return structured;
  }
  const diag = gameStore.plotState?.last_generation_diagnostics ?? '';
  const matched = diag.match(/风险分[=:：]\s*(\d+)/);
  if (!matched) {
    return null;
  }
  const value = Number(matched[1]);
  return Number.isFinite(value) ? value : null;
});
const dismissedNotificationIds = ref<string[]>([]);
const {
  isLoading,
  loadingMessage,
  autoAdvanceRunning,
  autoAdvanceStopHint,
  handleOptionSelect,
  handleFreeTextSubmit,
  handleContinue,
  requestStopAutoAdvance,
} = useStoryFlow({
  gameStore,
  shouldAutoAdvance,
  freeTextInput,
  validateFreeTextInput,
  createOptionAction,
  createFreeTextAction,
  createContinueAction,
  playClick,
});
const runtimeNotifications = computed<NotificationItem[]>(() => {
  const out: NotificationItem[] = [];

  if (gameStore.error) {
    out.push({
      id: 'runtime-error',
      kind: 'error',
      title: '系统错误',
      message: gameStore.error,
      priority: 'banner',
    });
  }
  if (autoAdvanceStopHint.value) {
    out.push({
      id: 'auto-advance-stop',
      kind: 'validation',
      title: '自动推进已暂停',
      message: autoAdvanceStopHint.value,
      priority: 'toast',
    });
  }

  return out.filter((item) => !dismissedNotificationIds.value.includes(item.id));
});

const dismissRuntimeNotification = (id: string) => {
  if (!dismissedNotificationIds.value.includes(id)) {
    dismissedNotificationIds.value.push(id);
  }
};

const handleSaved = (slotId: number) => {
  console.log(`游戏已保存到槽位 ${slotId}`);
};

const handleLoaded = (slotId: number) => {
  console.log(`已从槽位 ${slotId} 加载游戏`);
};

const handleTravel = async (locationId: string) => {
  if (!locationId) return;
  try {
    travelPending.value = true;
    await gameStore.travelToLocation(locationId);
  } catch (error) {
    console.error('地点移动失败：', error);
  } finally {
    travelPending.value = false;
  }
};

const toggleAudioPanel = () => {
  playClick();
  showAudioPanel.value = !showAudioPanel.value;
};

const toggleSystemMenu = () => {
  playClick();
  showSystemMenu.value = !showSystemMenu.value;
  if (!showSystemMenu.value) {
    showAudioPanel.value = false;
  }
};

const toggleMobileStatusCard = () => {
  playClick();
  showMobileStatusCard.value = !showMobileStatusCard.value;
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(
      MOBILE_STATUS_CARD_STORAGE_KEY,
      showMobileStatusCard.value ? '1' : '0',
    );
  }
};

const openShortcutsDialog = () => {
  closeSystemMenu();
  showShortcutsDialog.value = true;
};

const openLlmDialog = () => {
  closeSystemMenu();
  showLLMDialog.value = true;
};

const openStorySettingsDialog = () => {
  closeSystemMenu();
  showStorySettings.value = true;
};

const openConsistencySettingsFromMenu = async () => {
  closeSystemMenu();
  await openConsistencySettings();
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

const openConsistencySettings = async () => {
  try {
    const policy = await invokeWithTimeout<ConsistencyPolicy>(
      'get_consistency_policy',
      undefined,
      8000,
      '读取一致性策略超时',
    );
    consistencyPolicy.value = policy;
    showConsistencySettings.value = true;
  } catch (error) {
    console.error('读取一致性策略失败：', error);
  }
};

const applyConsistencyPolicy = async (policy: ConsistencyPolicy) => {
  consistencyPolicy.value = policy;
  try {
    const updated = await invokeWithTimeout<ConsistencyPolicy>(
      'update_consistency_policy',
      { policy },
      8000,
      '保存一致性策略超时',
    );
    consistencyPolicy.value = updated;
  } catch (error) {
    console.error('保存一致性策略失败：', error);
  }
};

const resetConsistencyPolicy = async () => {
  try {
    const reset = await invokeWithTimeout<ConsistencyPolicy>(
      'reset_consistency_policy',
      undefined,
      8000,
      '重置一致性策略超时',
    );
    consistencyPolicy.value = reset;
  } catch (error) {
    console.error('重置一致性策略失败：', error);
  }
};

const scrollToBottom = () => {
  storyViewportRef.value?.scrollToBottom();
};

watchEffect(() => {
  if (gameStore.isPlotInitialized) {
    void applyStorySettings(storySettings.value);
  }
});

// 监听章节内容变化，自动滚动到底部
watch(currentChapterParagraphs, (newParagraphs) => {
  if (newParagraphs.length > previousChapterParagraphs.value.length) {
    // 只有当有新内容时才滚动到底部
    requestAnimationFrame(() => {
      scrollToBottom();
    });
  }
  previousChapterParagraphs.value = [...newParagraphs];
}, { deep: true });

// 键盘快捷键支持
const handleKeydown = (event: KeyboardEvent) => {
  if (!gameStore.isGameInitialized) {
    return;
  }

  if (event.key === 'Escape') {
    closeAllDialogs();
    return;
  }

  const target = event.target instanceof HTMLElement ? event.target : null;
  const inTextInput = Boolean(
    target
    && (
      target.tagName === 'TEXTAREA'
      || target.tagName === 'INPUT'
      || target.tagName === 'SELECT'
      || target.isContentEditable
    ),
  );
  if (inTextInput || hasBlockingOverlay.value) {
    return;
  }

  if (event.key === 'Enter' && isNoInputAdvanceState.value) {
    event.preventDefault();
    handleContinue();
    return;
  }

  if (event.key === 'Enter' && inputMode.value === 'freeText' && freeTextInput.value.trim()) {
    event.preventDefault();
    handleFreeTextSubmit();
  }

  if (inputMode.value === 'options' && gameStore.availableOptions.length > 0) {
    const num = parseInt(event.key);
    if (num >= 1 && num <= 5 && num <= gameStore.availableOptions.length) {
      event.preventDefault();
      const option = gameStore.availableOptions[num - 1];
      handleOptionSelect(option);
    }
  }

  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    if (gameStore.isGameInitialized) {
      showSaveDialog.value = true;
    }
  }
};

useGameHotkeys(handleKeydown);

onMounted(() => {
  if (typeof window === 'undefined') {
    return;
  }
  const stored = window.localStorage.getItem(MOBILE_STATUS_CARD_STORAGE_KEY);
  if (stored != null) {
    showMobileStatusCard.value = stored === '1';
  }
});
</script>
