<template>
  <div
    class="game-shell h-screen overflow-hidden text-[var(--ink-text-primary)]"
    :class="activeThemeClass"
  >
    <div class="mx-auto flex h-full w-full max-w-[1380px] flex-col px-4 pb-4 pt-4 sm:px-7 sm:pb-6 sm:pt-5">
      <GameRuntimeTopBar
        :chapter-index-label="chapterIndexLabel"
        :chapter-name-label="chapterNameLabel"
        :interaction-state-tone-class="interactionStateToneClass"
        :interaction-state-label="interactionStateLabel"
        :game-time-label="gameTimeLabel"
        :spirit-stone-label="spiritStoneLabel"
        :character-creation-duration-label="characterCreationDurationLabel"
        @back-to-menu="handleBackToMenu"
      />

      <div class="runtime-content">
        <aside class="runtime-panel runtime-side-left">
          <GameRuntimeLeftStatusPanels
            :chapter-index-label="chapterIndexLabel"
            :chapter-name-label="chapterNameLabel"
            :current-location-label="currentLocationLabel"
            :chapter-rhythm-label="chapterRhythmLabel"
            :rhythm-tone-class="rhythmToneClass"
            :player-realm-label="playerRealmLabel"
            :player-root-elements="playerRootElements"
            :player-root-type-label="playerRootTypeLabel"
            :player-root-label="playerRootLabel"
          />
          <GameRuntimeWorldRegistryPanel
            v-bind="worldRegistryPanelProps"
            v-on="worldRegistryPanelListeners"
          />
        </aside>

        <section class="runtime-panel runtime-main-panel">
          <GameRuntimeMainHeader
            :chapter-index-label="chapterIndexLabel"
            :scene-headline-label="sceneHeadlineLabel"
            :show-scene-glyph="showSceneGlyph"
            :chapter-rhythm-label="chapterRhythmLabel"
            :rhythm-tone-class="rhythmToneClass"
          />
          <div class="runtime-main-body">
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
          </div>
        </section>

        <aside class="runtime-panel runtime-side-right">
          <GameRuntimeInteractionCard
            v-bind="interactionCardProps"
            v-on="interactionCardListeners"
          />
        </aside>
      </div>

      <GameRuntimeBottomBar
        v-bind="bottomBarProps"
        v-on="bottomBarListeners"
      />
    </div>

    <GameSystemDialogs
      v-bind="systemDialogsProps"
      v-on="systemDialogsListeners"
    />
    <GameInfoCenterDialog
      :is-open="showInfoTabs"
      :game-store="gameStore"
      :player-realm-label="playerRealmLabel"
      :player-combat-power-label="'不显示'"
      :chapter-progress-label="chapterProgressLabel"
      :chapter-interaction-label="chapterInteractionLabel"
      :world-location-list="worldLocationList"
      :recent-combat-review="recentCombatReview"
      :travel-pending="travelPending"
      :is-dev-mode="isDevMode"
      :option-source-label="optionSourceLabel"
      :option-source-hint="optionSourceHint || undefined"
      :consistency-risk-score="consistencyRiskScore"
      :no-name-debug-text="noNameDebugText"
      :no-name-mode="noNameMode"
      @close="showInfoTabs = false"
      @clear-error="gameStore.clearError()"
      @travel="handleTravel"
      @set-no-name-mode="setNoNameMode"
    />
    <CharacterInfoModal
      :is-open="showCharacterInfo"
      :character="gameStore.playerCharacter"
      @close="showCharacterInfo = false"
    />
    <RuntimeQuickPanelsDialog
      v-bind="quickPanelsDialogProps"
      v-on="quickPanelsDialogListeners"
    />
    <NotificationCenter
      v-if="runtimeNotifications.length > 0"
      :notifications="runtimeNotifications"
      @dismiss="dismissRuntimeNotification"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import CharacterInfoModal from './CharacterInfoModal.vue';
import GameInfoCenterDialog from './GameInfoCenterDialog.vue';
import GameRuntimeBottomBar from './GameRuntimeBottomBar.vue';
import GameRuntimeInteractionCard from './GameRuntimeInteractionCard.vue';
import GameRuntimeLeftStatusPanels from './GameRuntimeLeftStatusPanels.vue';
import GameRuntimeMainHeader from './GameRuntimeMainHeader.vue';
import GameRuntimeTopBar from './GameRuntimeTopBar.vue';
import GameRuntimeWorldRegistryPanel from './GameRuntimeWorldRegistryPanel.vue';
import GameSystemDialogs from './GameSystemDialogs.vue';
import NotificationCenter from './NotificationCenter.vue';
import RuntimeQuickPanelsDialog from './RuntimeQuickPanelsDialog.vue';
import StoryViewport from './StoryViewport.vue';
import type { ConsistencyPolicy, NoNameMode } from '../types/game';
import { invokeRuntime } from '../utils/tauriInvoke';
import {
  createFreeTextAction,
  createOptionAction,
  createContinueAction,
  validateFreeTextInput,
} from '../utils/playerInput';
import { useInputMode } from '../composables/useInputMode';
import { useRuntimeKeyboardHotkeys } from '../composables/useRuntimeKeyboardHotkeys';
import { useRuntimeNotifications } from '../composables/useRuntimeNotifications';
import { useRuntimeSettingsActions } from '../composables/useRuntimeSettingsActions';
import { useStoryFlow } from '../composables/useStoryFlow';
import { useUiPanels } from '../composables/useUiPanels';
import { useRuntimeActionFeedback } from '../composables/useRuntimeActionFeedback';
import { useRuntimeInteractionCardBridge } from '../composables/useRuntimeInteractionCardBridge';
import { useRuntimeQuickPanels } from '../composables/useRuntimeQuickPanels';
import { useRuntimeLifecycleRecovery } from '../composables/useRuntimeLifecycleRecovery';
import { useRuntimeStoryContent } from '../composables/useRuntimeStoryContent';
import { useRuntimeShellUi } from '../composables/useRuntimeShellUi';
import { useRuntimeInteractionState } from '../composables/useRuntimeInteractionState';
import { useRuntimeSessionActions } from '../composables/useRuntimeSessionActions';
import { useRuntimeStatusMetrics } from '../composables/useRuntimeStatusMetrics';
import { useRuntimeUiBridge } from '../composables/useRuntimeUiBridge';
import { useRuntimeViewportEffects } from '../composables/useRuntimeViewportEffects';
import { useWorldRegistryPanel } from '../composables/useWorldRegistryPanel';
import { useWorldRegistryPanelBridge } from '../composables/useWorldRegistryPanelBridge';
import { playClick } from '../utils/audioSystem';
import { getStorySettings, type StorySettings } from '../utils/storySettings';
import { getLlmProviderLabel, resolveLlmProviderKey } from '../utils/llmProvider';

const router = useRouter();
const gameStore = useGameStore();

const {
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showStorySettings,
  showInfoTabs,
  showConsistencySettings,
  showCharacterInfo,
  showShortcutsDialog,
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
const showQuickPanel = ref(false);
const noNameMode = ref<NoNameMode>('observeOnly');
const noNameDebugText = computed(() => {
  const getter = (gameStore as { getNoNameTraceDebugText?: () => string }).getNoNameTraceDebugText;
  return typeof getter === 'function' ? getter.call(gameStore) : '暂无 NoName Agent Trace。';
});

const refreshNoNameMode = async () => {
  try {
    noNameMode.value = await invokeRuntime<NoNameMode>('get_noname_mode', undefined);
  } catch (error) {
    console.warn('获取 NoName 模式失败，已忽略：', error);
  }
};

const setNoNameMode = async (mode: string) => {
  const target = (mode as NoNameMode) || 'observeOnly';
  try {
    noNameMode.value = await invokeRuntime<NoNameMode>('set_noname_mode', { mode: target });
  } catch (error) {
    console.warn('设置 NoName 模式失败，已忽略：', error);
  }
};

const safePlayClick = () => {
  try {
    playClick();
  } catch (error) {
    console.warn('播放点击音效失败，已忽略：', error);
  }
};

const worldRegistryPanel = useWorldRegistryPanel(gameStore, safePlayClick);
const { worldRegistryPanelProps, worldRegistryPanelListeners } = useWorldRegistryPanelBridge({
  panel: worldRegistryPanel,
});
const {
  currentChapterTitle, chapterProgressLabel, chapterIndexLabel, chapterNameLabel, sceneHeadlineLabel, showSceneGlyph,
  chapterInteractionLabel, gameTimeLabel, spiritStoneLabel, characterCreationDurationLabel, playerRealmLabel, playerRootLabel,
  playerRootElements, playerRootTypeLabel, currentLocationLabel, worldLocationList, recentCombatReview,
} = useRuntimeStatusMetrics({ gameStore });
const { quickPanels } = useRuntimeQuickPanels({
  gameStore,
  worldRegistrySessionLabel: worldRegistryPanel.worldRegistrySessionLabel,
  worldRegistrySourceLabel: worldRegistryPanel.worldRegistrySourceLabel,
  worldRegistryCounts: worldRegistryPanel.worldRegistryCounts,
  currentLocationLabel,
  spiritStoneLabel,
});
const recapEnabled = computed(() => storySettings.value.recap_enabled);
const {
  currentChapterParagraphs,
  lastChapterSummary,
  shouldShowRecap,
} = useRuntimeStoryContent({
  gameStore,
  recapEnabled,
});
const handleBackToMenu = () => {
  router.push('/');
};
const {
  actionNotification,
  logRuntimeAction,
  notifyRuntimeError,
} = useRuntimeActionFeedback();
const {
  activeQuickPanelTab, activeTheme, closeRuntimeQuickPanels, openCharacterDialog, openInfoDialog, openSaveDialog,
  openLoadDialog, openLlmDialogFromError, openQuickPanel, activeThemeClass, activeThemeLabel, toggleTheme,
  updateUiThemeFromSettings, openStorySettingsDialog,
} = useRuntimeShellUi({
  showQuickPanel,
  showCharacterInfo,
  showInfoTabs,
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showStorySettings,
  closeAllDialogs,
  safePlayClick,
  logRuntimeAction,
  notifyRuntimeError,
});
const {
  plotInteractionState, isNoInputAdvanceState, shouldShowInputPanel, shouldAutoAdvance, shouldAutoFollowNewParagraph,
  hasBlockingOverlay, optionSourceLabel, optionSourceHint, userFacingError, shouldShowLlmSetupShortcut, interactionStateLabel,
  interactionStateToneClass, consistencyRiskScore,
} = useRuntimeInteractionState({
  gameStore,
  isDevMode,
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showStorySettings,
  showConsistencySettings,
  showInfoTabs,
  showCharacterInfo,
  showShortcutsDialog,
  showQuickPanel,
});
const {
  isLoading,
  loadingMessage,
  loadingStage,
  loadingProgress,
  loadingProgressText,
  loadingElapsedMs,
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
const llmProviderKey = computed(() => resolveLlmProviderKey(gameStore.worldRegistry?.llm_model));
const llmProviderLabel = computed(() => getLlmProviderLabel(llmProviderKey.value));
const llmModelRaw = computed(() => gameStore.worldRegistry?.llm_model?.trim() || '未检测到模型标识');
const chapterRhythmLabel = computed(() => {
  if (isLoading.value || plotInteractionState.value === 'resolving') {
    return '推演';
  }
  if (plotInteractionState.value === 'waiting_for_choice') {
    return '舒缓';
  }
  if (plotInteractionState.value === 'waiting_for_free_text') {
    return '凝思';
  }
  return '流转';
});
const rhythmToneClass = computed(() => {
  if (chapterRhythmLabel.value === '推演') return 'runtime-rhythm-gold';
  if (chapterRhythmLabel.value === '凝思') return 'runtime-rhythm-ink';
  return 'runtime-rhythm-cool';
});
const { runtimeNotifications, dismissRuntimeNotification } = useRuntimeNotifications({
  characterCreationDurationLabel,
  autoAdvanceStopHint,
  actionNotification,
  runtimeError: userFacingError,
  backgroundNotice: computed(() => gameStore.backgroundNotice),
  clearBackgroundNotice: () => gameStore.clearBackgroundNotice(),
});
const { travelPending, handleSaved, handleLoaded, handleTravel } = useRuntimeSessionActions({
  gameStore,
  logRuntimeAction,
  notifyRuntimeError,
});
const clearWorldMetrics = () => {
  safePlayClick();
  gameStore.clearGenerationDiagnostics();
  logRuntimeAction('已清空诊断统计');
};
const copyWorldDiagnostics = async () => {
  safePlayClick();
  try {
    if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
      throw new Error('当前环境不支持剪贴板写入');
    }
    await navigator.clipboard.writeText(gameStore.getGenerationDiagnosticsText());
    logRuntimeAction('已复制诊断数据');
  } catch (error) {
    notifyRuntimeError('复制诊断数据', error);
  }
};
const {
  applyStorySettings,
  applyConsistencyPolicy,
  resetConsistencyPolicy,
} = useRuntimeSettingsActions({
  storySettings,
  consistencyPolicy,
  notifyRuntimeError,
});
const {
  bottomBarProps,
  bottomBarListeners,
  systemDialogsProps,
  systemDialogsListeners,
  quickPanelsDialogProps,
  quickPanelsDialogListeners,
} = useRuntimeUiBridge({
  isGameInitialized: computed(() => gameStore.isGameInitialized),
  activeThemeLabel,
  openCharacterDialog,
  openInfoDialog,
  openQuickPanel,
  openSaveDialog,
  openLoadDialog,
  toggleTheme,
  openStorySettingsDialog,
  showSaveDialog,
  showLoadDialog,
  showShortcutsDialog,
  showLLMDialog,
  showStorySettings,
  showConsistencySettings,
  storySettings,
  activeTheme,
  consistencyPolicy,
  handleSaved,
  handleLoaded,
  applyStorySettings,
  updateUiThemeFromSettings,
  applyConsistencyPolicy,
  resetConsistencyPolicy,
  showQuickPanel,
  activeQuickPanelTab,
  quickPanels,
  copyWorldDiagnostics,
  clearWorldMetrics,
});
const { interactionCardProps, interactionCardListeners } = useRuntimeInteractionCardBridge({
  shouldShowInputPanel,
  userFacingError,
  isNoInputAdvanceState,
  availableOptions: computed(() => gameStore.availableOptions),
  inputMode,
  isLoading,
  freeTextInput,
  inputValidation,
  isGameInitialized: computed(() => gameStore.isGameInitialized),
  isWaitingForInput: computed(() => gameStore.isWaitingForInput),
  loadingMessage,
  loadingStage,
  loadingProgress,
  loadingProgressText,
  loadingElapsedMs,
  autoAdvanceRunning,
  autoAdvanceStopHint,
  shouldShowLlmSetupShortcut,
  llmProviderKey,
  llmProviderLabel,
  llmModelRaw,
  setInputMode,
  handleOptionSelect,
  setFreeTextInput: (value) => {
    freeTextInput.value = value;
  },
  handleFreeTextSubmit,
  handleContinue,
  requestStopAutoAdvance,
  openLlmDialogFromError,
});

const scrollToBottom = () => {
  storyViewportRef.value?.scrollToBottom();
};
useRuntimeViewportEffects({
  isPlotInitialized: computed(() => gameStore.isPlotInitialized),
  storySettings,
  applyStorySettings,
  currentChapterParagraphs,
  shouldAutoFollowNewParagraph,
  previousChapterParagraphs,
  scrollToBottom,
});
useRuntimeKeyboardHotkeys({
  gameStore,
  closeAllDialogs,
  closeRuntimeQuickPanels,
  hasBlockingOverlay,
  isNoInputAdvanceState,
  inputMode,
  freeTextInput,
  handleContinue,
  handleFreeTextSubmit,
  handleOptionSelect,
  showSaveDialog,
});
useRuntimeLifecycleRecovery({
  gameStore,
  inputMode,
  freeTextInput,
  setInputMode,
  currentChapterTitle,
  currentChapterIndex: computed(() => gameStore.plotState?.current_chapter.index ?? 0),
  currentSegmentCount: computed(() => gameStore.plotState?.segment_count ?? 0),
  logRuntimeAction,
  notifyRuntimeError,
});

onMounted(() => {
  refreshNoNameMode();
});

watch(
  () => gameStore.plotState?.segment_count,
  (value, prev) => {
    if (value !== prev) {
      refreshNoNameMode();
    }
  },
);
</script>

<style scoped src="../styles/game-runtime-view.css"></style>

