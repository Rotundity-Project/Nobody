<template>
  <div class="game-shell min-h-screen text-[var(--ink-text-primary)]" :class="activeThemeClass">
    <div class="mx-auto flex min-h-screen w-full max-w-[1380px] flex-col px-4 pb-4 pt-4 sm:px-7 sm:pb-6 sm:pt-5">
      <header class="runtime-topbar">
        <div class="runtime-top-left">
          <button type="button" class="runtime-seal-btn" @click="handleBackToMenu">返</button>
          <p class="runtime-brand">NOBODY</p>
        </div>
        <div class="runtime-top-center">
          <p class="runtime-top-main">
            <span class="runtime-chapter-number">{{ chapterIndexLabel }}</span>
            <span class="runtime-chapter-name">{{ chapterNameLabel }}</span>
          </p>
          <span class="runtime-state-badge" :class="interactionStateToneClass">
            <span class="runtime-state-dot" aria-hidden="true"></span>
            <span class="runtime-state-label">{{ interactionStateLabel }}</span>
          </span>
        </div>
        <div class="runtime-top-right">
          <p>{{ gameTimeLabel }}</p>
          <p class="runtime-resource-pill">灵石 · {{ spiritStoneLabel }}</p>
          <p v-if="characterCreationDurationLabel" class="runtime-sub-text">
            创建角色耗时 · {{ characterCreationDurationLabel }}
          </p>
        </div>
      </header>

      <div class="runtime-content">
        <aside class="runtime-panel runtime-side-left">
          <section class="runtime-card">
            <h3 class="runtime-card-title">系统中枢</h3>
            <p class="runtime-chapter-title">
              <span class="runtime-chapter-number">{{ chapterIndexLabel }}</span>
              <span class="runtime-chapter-name">{{ chapterNameLabel }}</span>
            </p>
            <p class="runtime-sub-text">所在：{{ currentLocationLabel }}</p>
            <div class="runtime-rhythm-badge" :class="rhythmToneClass">
              <span class="runtime-rhythm-icon" aria-hidden="true">
                <svg
                  v-if="chapterRhythmLabel === '推演'"
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 16c2-5 4-8 8-8s6 3 8 8" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M7 10h10" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v12" />
                </svg>
                <svg
                  v-else-if="chapterRhythmLabel === '凝思'"
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 16c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 12c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6" />
                </svg>
                <svg
                  v-else
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 12c2.8-2.2 4.4-2.2 7.2 0 2.8 2.2 4.4 2.2 7.2 0" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 8c1.6-1.4 2.8-1.4 4.4 0" />
                </svg>
              </span>
              <span class="runtime-rhythm-label">节奏</span>
              <span class="runtime-rhythm-value">{{ chapterRhythmLabel }}</span>
            </div>
          </section>
          <section class="runtime-card">
            <h3 class="runtime-card-title">人物</h3>
            <p class="runtime-body-text">{{ playerRealmLabel }}</p>
            <div v-if="playerRootElements.length > 0" class="runtime-root-line">
              <div
                v-for="item in playerRootElements"
                :key="item.element"
                class="runtime-root-item"
              >
                <span class="runtime-root-icon" :class="item.colorClass" aria-hidden="true">
                  <svg
                    v-if="item.element === 'Earth'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 18h18L16 8h-8L3 18Z" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Metal'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <circle cx="12" cy="12" r="6.5" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 5.5v13M5.5 12h13" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Wood'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 20V8" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 10c3.5 0 5-2 5-4-3 0-5 1.8-5 4Z" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 13c-3.5 0-5-2-5-4 3 0 5 1.8 5 4Z" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Water'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4c3.6 4.2 5.5 7 5.5 9.5A5.5 5.5 0 0 1 12 19a5.5 5.5 0 0 1-5.5-5.5C6.5 11 8.4 8.2 12 4Z" />
                  </svg>
                  <svg
                    v-else
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4c2.5 2 4.5 4.2 4.5 6.8 0 3.4-2.5 5.8-4.5 9.2-2-3.4-4.5-5.8-4.5-9.2C7.5 8.2 9.5 6 12 4Z" />
                  </svg>
                </span>
                <span class="runtime-root-name" :class="item.colorClass">{{ item.label }}</span>
              </div>
              <span class="runtime-root-type">{{ playerRootTypeLabel }}</span>
            </div>
            <p v-else class="runtime-sub-text">{{ playerRootLabel }}</p>
          </section>
          <section class="runtime-card">
            <div class="flex items-center justify-between gap-2">
              <h3 class="runtime-card-title">世界属性表</h3>
              <button
                type="button"
                class="runtime-bottom-btn px-2 py-1 text-[11px]"
                @click="refreshWorldRegistryPanel"
              >
                刷新
              </button>
            </div>
            <p class="runtime-sub-text">
              会话：{{ worldRegistrySessionLabel }}
            </p>
            <p class="runtime-sub-text">
              来源：{{ worldRegistrySourceLabel }}
            </p>
            <div class="mt-2 grid grid-cols-2 gap-2 text-xs text-[#6b655d]">
              <p>人物：{{ worldRegistryCounts.characters }}</p>
              <p>地图点：{{ worldRegistryCounts.map_nodes }}</p>
              <p>地图边：{{ worldRegistryCounts.map_edges }}</p>
              <p>功法：{{ worldRegistryCounts.techniques }}</p>
              <p>背包：{{ worldRegistryCounts.inventory_items }}</p>
              <p>势力：{{ worldRegistryCounts.factions }}</p>
              <p>剧情态：{{ worldRegistryCounts.story_state }}</p>
              <p>事实：{{ worldRegistryCounts.world_facts }}</p>
            </div>
            <details class="mt-2">
              <summary class="cursor-pointer text-xs text-[#6b655d]">查看 JSON 预览</summary>
              <pre class="mt-2 max-h-40 overflow-auto rounded-lg bg-black/5 p-2 text-[10px] leading-4 text-[#524d45]">{{ worldRegistryPreview }}</pre>
            </details>
            <details class="mt-2">
              <summary class="cursor-pointer text-xs text-[#6b655d]">提交 Patch(JSON)</summary>
              <textarea
                v-model="worldRegistryPatchInput"
                class="mt-2 min-h-[120px] w-full rounded-lg border border-[#d2c8b7] bg-white/80 p-2 text-[11px] leading-4 text-[#4b463f]"
                spellcheck="false"
              />
              <div class="mt-2 flex items-center gap-2">
                <button
                  type="button"
                  class="runtime-bottom-btn px-2 py-1 text-[11px]"
                  :disabled="worldRegistryPatchSubmitting"
                  @click="applyWorldRegistryPatchFromPanel"
                >
                  {{ worldRegistryPatchSubmitting ? '提交中...' : '提交补丁' }}
                </button>
                <button
                  type="button"
                  class="runtime-bottom-btn px-2 py-1 text-[11px]"
                  @click="resetWorldRegistryPatchTemplate"
                >
                  重置模板
                </button>
              </div>
              <p v-if="worldRegistryPatchError" class="mt-1 text-xs text-[#b14534]">{{ worldRegistryPatchError }}</p>
            </details>
            <details class="mt-2">
              <summary class="cursor-pointer text-xs text-[#6b655d]">按表追加一行</summary>
              <div class="mt-2 space-y-2 text-xs">
                <label class="flex flex-col gap-1">
                  <span class="text-[#6b655d]">目标表</span>
                  <select
                    v-model="worldRegistrySelectedTable"
                    class="rounded-lg border border-[#d2c8b7] bg-white/80 px-2 py-1 text-[12px] text-[#4b463f]"
                  >
                    <option v-for="item in worldRegistryTableOptions" :key="item" :value="item">{{ item }}</option>
                  </select>
                </label>
                <label class="flex flex-col gap-1">
                  <span class="text-[#6b655d]">行 JSON（对象）</span>
                  <textarea
                    v-model="worldRegistryRowInput"
                    class="min-h-[96px] w-full rounded-lg border border-[#d2c8b7] bg-white/80 p-2 text-[11px] leading-4 text-[#4b463f]"
                    spellcheck="false"
                  />
                </label>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    :disabled="worldRegistryPatchSubmitting"
                    @click="appendRowToRegistryTable"
                  >
                    追加一行
                  </button>
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    @click="loadSelectedTableFirstRowTemplate"
                  >
                    从首行载入模板
                  </button>
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    @click="loadMinimalRowTemplateForSelectedTable"
                  >
                    最小合法模板
                  </button>
                </div>
                <label class="flex flex-col gap-1">
                  <span class="text-[#6b655d]">目标索引（用于替换/删除）</span>
                  <input
                    v-model.number="worldRegistrySelectedIndex"
                    type="number"
                    min="0"
                    class="w-28 rounded-lg border border-[#d2c8b7] bg-white/80 px-2 py-1 text-[12px] text-[#4b463f]"
                  />
                </label>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    @click="loadSelectedTableRowByIndex"
                  >
                    载入该行
                  </button>
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    :disabled="worldRegistryPatchSubmitting"
                    @click="replaceRowInRegistryTable"
                  >
                    替换该行
                  </button>
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    :disabled="worldRegistryPatchSubmitting"
                    @click="deleteRowInRegistryTable"
                  >
                    删除该行
                  </button>
                </div>
                <label class="flex flex-col gap-1">
                  <span class="text-[#6b655d]">主键字段（用于按主键更新/新增）</span>
                  <input
                    v-model="worldRegistryKeyField"
                    type="text"
                    class="w-40 rounded-lg border border-[#d2c8b7] bg-white/80 px-2 py-1 text-[12px] text-[#4b463f]"
                  />
                </label>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="runtime-bottom-btn px-2 py-1 text-[11px]"
                    :disabled="worldRegistryPatchSubmitting"
                    @click="upsertRowByKeyInRegistryTable"
                  >
                    按主键更新/新增
                  </button>
                </div>
                <p v-if="worldRegistryRowError" class="text-xs text-[#b14534]">{{ worldRegistryRowError }}</p>
                <div class="mt-1 rounded-lg bg-black/5 p-2">
                  <p class="text-[11px] text-[#6b655d]">当前表预览（{{ worldRegistrySelectedTableRows.length }} 行）</p>
                  <ul class="mt-1 max-h-28 overflow-auto space-y-1 text-[11px] text-[#4b463f]">
                    <li v-for="item in worldRegistrySelectedTableRowsPaged" :key="`${item.index}-${item.label}`">
                      [{{ item.index }}] {{ item.label }}
                    </li>
                  </ul>
                  <div class="mt-1 flex items-center gap-2">
                    <button
                      type="button"
                      class="runtime-bottom-btn px-2 py-1 text-[11px]"
                      :disabled="worldRegistryRowPage <= 0"
                      @click="worldRegistryRowPage = Math.max(0, worldRegistryRowPage - 1)"
                    >
                      上一页
                    </button>
                    <button
                      type="button"
                      class="runtime-bottom-btn px-2 py-1 text-[11px]"
                      :disabled="(worldRegistryRowPage + 1) * worldRegistryRowPageSize >= worldRegistrySelectedTableRows.length"
                      @click="worldRegistryRowPage = worldRegistryRowPage + 1"
                    >
                      下一页
                    </button>
                  </div>
                </div>
              </div>
            </details>
          </section>
        </aside>

        <section class="runtime-panel runtime-main-panel">
          <div class="runtime-main-header">
            <div class="runtime-main-scene">
              <span class="runtime-main-chapter">{{ chapterIndexLabel }}</span>
              <span v-if="showSceneGlyph" class="runtime-main-scene-icon" aria-hidden="true">
                <svg
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 20h16" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 20V9h12v11" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 4l7 5H5l7-5Z" />
                </svg>
              </span>
              <span class="runtime-main-scene-name">{{ sceneHeadlineLabel }}</span>
            </div>
            <div class="runtime-rhythm-badge runtime-rhythm-badge-compact" :class="rhythmToneClass">
              <span class="runtime-rhythm-icon" aria-hidden="true">
                <svg
                  v-if="chapterRhythmLabel === '推演'"
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 16c2-5 4-8 8-8s6 3 8 8" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M7 10h10" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v12" />
                </svg>
                <svg
                  v-else-if="chapterRhythmLabel === '凝思'"
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 16c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 12c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6" />
                </svg>
                <svg
                  v-else
                  viewBox="0 0 24 24"
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 12c2.8-2.2 4.4-2.2 7.2 0 2.8 2.2 4.4 2.2 7.2 0" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 8c1.6-1.4 2.8-1.4 4.4 0" />
                </svg>
              </span>
              <span class="runtime-rhythm-label">节奏</span>
              <span class="runtime-rhythm-value">{{ chapterRhythmLabel }}</span>
            </div>
          </div>
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
          <section class="runtime-card runtime-interaction-card">
            <h3 class="runtime-card-title runtime-interaction-title">题签交互</h3>
            <p class="runtime-sub-text runtime-interaction-subtitle">选项自由输入</p>
            <GameInteractionPanel
              :should-show-input-panel="shouldShowInputPanel"
              :error="userFacingError"
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
            <div v-if="shouldShowLlmSetupShortcut" class="runtime-llm-shortcut">
              <p class="runtime-sub-text">检测到本轮选项续写未命中 LLM，可直接打开设置后重试。</p>
              <button type="button" class="runtime-bottom-btn px-3 py-1 text-xs" @click="openLlmDialogFromError">
                打开 LLM 设置
              </button>
            </div>
          </section>
        </aside>
      </div>

      <div class="runtime-bottom-bar">
        <InkQuickActionDock
          :is-game-initialized="gameStore.isGameInitialized"
          @open-character="openCharacterDialog"
          @open-info="openInfoDialog"
          @open-backpack="openQuickPanel('backpack')"
          @open-techniques="openQuickPanel('techniques')"
          @open-factions="openQuickPanel('factions')"
          @open-world="openQuickPanel('world')"
          @open-save="openSaveDialog"
          @open-load="openLoadDialog"
        />
        <div class="runtime-bottom-right">
          <button type="button" class="runtime-bottom-btn" @click="openStorySettingsDialog">系统设置</button>
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
      @close="showInfoTabs = false"
      @clear-error="gameStore.clearError()"
      @travel="handleTravel"
    />
    <CharacterInfoModal
      :is-open="showCharacterInfo"
      :character="gameStore.playerCharacter"
      @close="showCharacterInfo = false"
    />
    <RuntimeQuickPanelsDialog
      :is-open="showQuickPanel"
      :active-tab="activeQuickPanelTab"
      :panels="quickPanels"
      @update:active-tab="activeQuickPanelTab = $event"
      @close="showQuickPanel = false"
    />
    <NotificationCenter
      v-if="runtimeNotifications.length > 0"
      :notifications="runtimeNotifications"
      @dismiss="dismissRuntimeNotification"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watchEffect, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useGameStore } from '../stores/gameStore';
import CharacterInfoModal from './CharacterInfoModal.vue';
import GameInfoCenterDialog from './GameInfoCenterDialog.vue';
import GameSystemDialogs from './GameSystemDialogs.vue';
import GameInteractionPanel from './GameInteractionPanel.vue';
import InkQuickActionDock from './InkQuickActionDock.vue';
import NotificationCenter, { type NotificationItem } from './NotificationCenter.vue';
import RuntimeQuickPanelsDialog, { type RuntimeQuickTab } from './RuntimeQuickPanelsDialog.vue';
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
const travelPending = ref(false);
const showQuickPanel = ref(false);
const QUICK_PANEL_TAB_STORAGE_KEY = 'runtime.quick_panel.active_tab';
const allowedQuickTabs: RuntimeQuickTab[] = ['backpack', 'techniques', 'factions', 'world'];
const loadSavedQuickPanelTab = (): RuntimeQuickTab => {
  try {
    const raw = localStorage.getItem(QUICK_PANEL_TAB_STORAGE_KEY) ?? '';
    if (allowedQuickTabs.includes(raw as RuntimeQuickTab)) {
      return raw as RuntimeQuickTab;
    }
  } catch {
    // Ignore storage errors in restricted contexts.
  }
  return 'backpack';
};
const activeQuickPanelTab = ref<RuntimeQuickTab>(loadSavedQuickPanelTab());

const currentChapterTitle = computed(
  () => gameStore.plotState?.current_chapter?.title || gameStore.currentScene?.name || '第一章'
);
const realmStageLabel = (subLevel?: number): string => {
  if (!subLevel || subLevel <= 1) return '初期';
  if (subLevel === 2) return '中期';
  if (subLevel === 3) return '后期';
  return '圆满';
};
const playerRealmLabel = computed(() => {
  const realm = gameStore.playerCharacter?.stats?.cultivation_realm;
  if (!realm) {
    return '凡人';
  }
  return `${realm.name}${realmStageLabel(realm.sub_level)}（${realm.level}-${realm.sub_level}）`;
});
const chapterProgressLabel = computed(() => {
  const chapter = gameStore.plotState?.current_chapter;
  if (!chapter) {
    return '0 / 无';
  }
  return `${chapter.index} / ${chapter.title || '未命名章节'}`;
});
const chapterIndexLabel = computed(() => {
  const chapter = gameStore.plotState?.current_chapter;
  const idx = chapter?.index ?? 1;
  const cn = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
  const suffix = idx >= 0 && idx < cn.length ? cn[idx] : String(idx);
  return `第${suffix}章`;
});
const chapterNameLabel = computed(() => {
  const raw = currentChapterTitle.value.trim();
  const chapterOnlyPattern = /^第[零一二三四五六七八九十百千万\d]+章$/u;
  const stripped = raw
    .replace(/^第[零一二三四五六七八九十百千万\d]+章[\s·、:：-]*/u, '')
    .trim();
  if (stripped.length > 0 && !chapterOnlyPattern.test(stripped)) {
    return stripped;
  }
  const sceneName = (gameStore.currentScene?.name ?? '').trim();
  if (sceneName.length > 0 && !chapterOnlyPattern.test(sceneName)) {
    return sceneName;
  }
  return '未命名章节';
});
const sceneHeadlineLabel = computed(() => {
  const chapterName = chapterNameLabel.value.trim();
  if (chapterName.length > 0 && !/^第[零一二三四五六七八九十百千万\d]+章$/u.test(chapterName)) {
    return chapterName;
  }
  const location = currentLocationLabel.value.trim();
  return location.length > 0 ? location : '无名之地';
});
const showSceneGlyph = computed(() => /[堂殿阁楼宫门院府台塔]/u.test(sceneHeadlineLabel.value));
const chapterInteractionLabel = computed(() => {
  const chapter = gameStore.plotState?.current_chapter;
  if (!chapter) {
    return '0 / 0-0';
  }
  const min = gameStore.plotState?.settings?.min_interactions_per_chapter ?? 0;
  const max = gameStore.plotState?.settings?.max_interactions_per_chapter ?? 0;
  return `${chapter.interaction_count} / ${min}-${max}`;
});
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
const gameTimeLabel = computed(() => {
  const time = gameStore.gameState?.game_time;
  if (!time) {
    return '第0年 · 第0月 · 第0日';
  }
  return `第${time.year}年 · 第${time.month}月 · 第${time.day}日`;
});
const spiritStoneLabel = computed(() => {
  const inventory = gameStore.playerCharacter?.inventory ?? [];
  if (inventory.length === 0) {
    return '0';
  }
  const parsed = inventory.reduce((sum, item) => {
    const text = String(item ?? '');
    if (!/灵石|spirit\s*stone/i.test(text)) {
      return sum;
    }
    const num = text.match(/(\d+)/);
    return sum + (num ? Number(num[1]) : 1);
  }, 0);
  if (parsed > 0) {
    return parsed.toLocaleString();
  }
  return inventory.length.toLocaleString();
});
const characterCreationDurationLabel = computed(() => {
  const ms = gameStore.lastInitializationDurationMs;
  if (typeof ms !== 'number' || !Number.isFinite(ms) || ms <= 0) {
    return '';
  }
  const secs = ms / 1000;
  return `${secs.toFixed(2)}s`;
});
const backpackPanelItems = computed(() => {
  const inventory = gameStore.playerCharacter?.inventory ?? [];
  const toItem = (item: string, index: number) => {
    const isSpiritStone = /灵石|spirit\s*stone/i.test(item);
    return {
      id: `bag-${index}`,
      title: String(item),
      meta: isSpiritStone ? `灵石统计：${spiritStoneLabel.value}` : undefined,
      badge: isSpiritStone ? '灵石' : undefined,
      featured: isSpiritStone,
    };
  };
  return inventory
    .map((item, index) => toItem(String(item), index))
    .sort((a, b) => Number(b.featured) - Number(a.featured));
});
const techniquePanelItems = computed(() => {
  const learned = gameStore.playerCharacter?.stats?.techniques ?? [];
  const worldTechniques = gameStore.gameState?.script?.world_setting?.techniques ?? [];
  const worldMap = new Map(worldTechniques.map((tech) => [tech.name, tech]));
  const fromLearned = learned.map((name, index) => {
    const mapped = worldMap.get(name);
    return {
      id: `learned-${index}-${name}`,
      title: name,
      description: mapped?.description || '已掌握功法',
      meta: mapped ? `需求境界：${mapped.required_realm_level}` : '来源：角色面板',
    };
  });
  const learnedSet = new Set(learned);
  const fromWorld = worldTechniques
    .filter((tech) => !learnedSet.has(tech.name))
    .slice(0, 24)
    .map((tech) => ({
      id: `world-tech-${tech.id}`,
      title: tech.name,
      description: tech.description,
      meta: `需求境界：${tech.required_realm_level}`,
    }));
  return [...fromLearned, ...fromWorld];
});
const factionPanelItems = computed(() => {
  const scriptFactions = gameStore.gameState?.script?.world_setting?.factions ?? [];
  const worldFactions = Object.values(gameStore.gameState?.world_state?.factions ?? {});
  const merged = [...scriptFactions];
  for (const faction of worldFactions) {
    if (!merged.some((item) => item.id === faction.id)) {
      merged.push(faction);
    }
  }
  return merged.map((faction) => ({
    id: `faction-${faction.id}`,
    title: faction.name,
    description: faction.description,
    meta: `势力等级：${faction.power_level}`,
  }));
});
const worldPanelItems = computed(() => ([
  {
    id: 'world-session',
    title: `会话：${worldRegistrySessionLabel.value}`,
    description: `来源：${worldRegistrySourceLabel.value}`,
  },
  {
    id: 'world-count-characters',
    title: `人物：${worldRegistryCounts.value.characters}`,
    meta: `地图点：${worldRegistryCounts.value.map_nodes}，地图边：${worldRegistryCounts.value.map_edges}`,
  },
  {
    id: 'world-count-assets',
    title: `功法：${worldRegistryCounts.value.techniques}，背包：${worldRegistryCounts.value.inventory_items}`,
    meta: `势力：${worldRegistryCounts.value.factions}，剧情态：${worldRegistryCounts.value.story_state}`,
  },
  {
    id: 'world-count-facts',
    title: `事实：${worldRegistryCounts.value.world_facts}`,
    meta: `当前位置：${currentLocationLabel.value}`,
  },
]));
const quickPanels = computed(() => ([
  {
    id: 'backpack' as const,
    label: '背包',
    title: '背包',
    subtitle: '当前携带物与可追踪资源',
    emptyText: '背包为空。',
    items: backpackPanelItems.value,
  },
  {
    id: 'techniques' as const,
    label: '功法',
    title: '功法',
    subtitle: '已掌握与世界可见功法',
    emptyText: '尚未获得可展示功法。',
    items: techniquePanelItems.value,
  },
  {
    id: 'factions' as const,
    label: '势力',
    title: '势力',
    subtitle: '世界中的门派与组织',
    emptyText: '暂无势力信息。',
    items: factionPanelItems.value,
  },
  {
    id: 'world' as const,
    label: '世界',
    title: '世界快照',
    subtitle: '本轮世界状态索引',
    emptyText: '暂无世界快照。',
    items: worldPanelItems.value,
  },
]));
const playerRootLabel = computed(() => {
  const root = gameStore.playerCharacter?.stats?.spiritual_root;
  if (!root) {
    return '灵根未显';
  }
  const elements = (root.elements?.length ? root.elements : [root.element]).map((value) => String(value));
  const mapped = elements.map((value) => {
    const names: Record<string, string> = {
      Fire: '火',
      Water: '水',
      Wood: '木',
      Metal: '金',
      Earth: '土',
    };
    return names[value] ?? value;
  });
  if (mapped.length === 1) {
    return `${mapped[0]}灵根`;
  }
  if (mapped.length === 2) {
    return `${mapped.join('')}双灵根`;
  }
  if (mapped.length === 3) {
    return `${mapped.join('')}三灵根`;
  }
  return `${mapped.join('/')}杂灵根`;
});
type RootElement = 'Fire' | 'Water' | 'Wood' | 'Metal' | 'Earth';
const rootElementNameMap: Record<RootElement, string> = {
  Fire: '火',
  Water: '水',
  Wood: '木',
  Metal: '金',
  Earth: '土',
};
const rootElementClassMap: Record<RootElement, string> = {
  Fire: 'runtime-root-fire',
  Water: 'runtime-root-water',
  Wood: 'runtime-root-wood',
  Metal: 'runtime-root-metal',
  Earth: 'runtime-root-earth',
};
const playerRootElements = computed(() => {
  const root = gameStore.playerCharacter?.stats?.spiritual_root;
  if (!root) return [];
  const values = (root.elements?.length ? root.elements : [root.element])
    .map((value) => String(value))
    .filter((value): value is RootElement =>
      value === 'Fire' || value === 'Water' || value === 'Wood' || value === 'Metal' || value === 'Earth');

  return values.map((element) => ({
    element,
    label: rootElementNameMap[element],
    colorClass: rootElementClassMap[element],
  }));
});
const playerRootTypeLabel = computed(() => {
  const count = playerRootElements.value.length;
  if (count <= 1) return '灵根';
  if (count === 2) return '双灵根';
  if (count === 3) return '三灵根';
  return '杂灵根';
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
const worldRegistrySessionLabel = computed(() => {
  const sid = gameStore.worldRegistry?.session_id?.trim();
  if (!sid) return '未注册';
  return sid.length > 20 ? `${sid.slice(0, 20)}...` : sid;
});
const worldRegistrySourceLabel = computed(() => {
  return gameStore.worldRegistry?.source || 'unknown';
});
const worldRegistryCounts = computed(() => {
  const tables = gameStore.worldRegistry?.tables;
  return {
    characters: tables?.characters?.length ?? 0,
    map_nodes: tables?.map_nodes?.length ?? 0,
    map_edges: tables?.map_edges?.length ?? 0,
    techniques: tables?.techniques?.length ?? 0,
    inventory_items: tables?.inventory_items?.length ?? 0,
    factions: tables?.factions?.length ?? 0,
    story_state: tables?.story_state?.length ?? 0,
    world_facts: tables?.world_facts?.length ?? 0,
  };
});
const worldRegistryPreview = computed(() => {
  if (!gameStore.worldRegistry) return '{}';
  try {
    return JSON.stringify(gameStore.worldRegistry, null, 2);
  } catch {
    return '{}';
  }
});
const worldRegistryPatchTemplate = `{
  "world_facts": [
    {
      "fact_id": "fact_manual_1",
      "subject": "player",
      "predicate": "goal",
      "object": "secure_supplies"
    }
  ]
}`;
const worldRegistryPatchInput = ref<string>(worldRegistryPatchTemplate);
const worldRegistryPatchSubmitting = ref(false);
const worldRegistryPatchError = ref('');
const worldRegistrySelectedTable = ref<
  | 'characters'
  | 'map_nodes'
  | 'map_edges'
  | 'techniques'
  | 'inventory_items'
  | 'factions'
  | 'story_state'
  | 'world_facts'
>('world_facts');
const worldRegistryDefaultKeyMap: Record<string, string> = {
  characters: 'character_id',
  map_nodes: 'location_id',
  map_edges: 'from_id',
  techniques: 'technique_id',
  inventory_items: 'item_id',
  factions: 'faction_id',
  story_state: 'chapter_index',
  world_facts: 'fact_id',
};
const worldRegistryTableOptions = [
  'characters',
  'map_nodes',
  'map_edges',
  'techniques',
  'inventory_items',
  'factions',
  'story_state',
  'world_facts',
] as const;
const worldRegistryRowInput = ref<string>(
  JSON.stringify(
    {
      fact_id: 'fact_manual_2',
      subject: 'player',
      predicate: 'plan',
      object: 'visit_market',
    },
    null,
    2,
  ),
);
const worldRegistryRowError = ref('');
const worldRegistrySelectedIndex = ref<number>(0);
const worldRegistryKeyField = ref<string>(worldRegistryDefaultKeyMap.world_facts);
const worldRegistryRowPage = ref(0);
const worldRegistryRowPageSize = 6;
const worldRegistrySelectedTableRows = computed(() => {
  const table = worldRegistrySelectedTable.value;
  const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
  return rows.map((row, index) => {
    const key = worldRegistryKeyField.value.trim();
    const asRecord = row as Record<string, unknown>;
    const keyVal = key ? asRecord?.[key] : undefined;
    const label = keyVal !== undefined
      ? `${String(keyVal)}`
      : JSON.stringify(row).slice(0, 80);
    return { index, label };
  });
});
const worldRegistrySelectedTableRowsPaged = computed(() => {
  const start = worldRegistryRowPage.value * worldRegistryRowPageSize;
  return worldRegistrySelectedTableRows.value.slice(start, start + worldRegistryRowPageSize);
});
const buildMinimalRowTemplateForTable = (table: string): Record<string, unknown> => {
  switch (table) {
    case 'characters':
      return {
        character_id: 'char_manual_1',
        name: 'NewCharacter',
        role: 'npc',
        realm_stage: 'Qi',
        realm_substage: 0,
        location_id: gameStore.playerCharacter?.location || 'sect',
      };
    case 'map_nodes':
      return {
        location_id: 'loc_manual_1',
        name: 'NewLocation',
        description: 'newly discovered location',
        spiritual_density: 0.5,
      };
    case 'map_edges':
      return {
        from_id: gameStore.playerCharacter?.location || 'sect',
        to_id: 'loc_manual_1',
        travel_days: 1,
        travel_risk: 0,
      };
    case 'techniques':
      return {
        technique_id: 'tech_manual_1',
        name: 'ManualTechnique',
        description: 'generated from panel template',
        owner_character_id: 'player',
      };
    case 'inventory_items':
      return {
        item_id: 'item_manual_1',
        owner_character_id: 'player',
        name: 'ManualItem',
        item_type: 'material',
        quantity: 1,
        effect_desc: 'no effect',
      };
    case 'factions':
      return {
        faction_id: 'faction_manual_1',
        name: 'ManualFaction',
        description: 'new faction',
      };
    case 'story_state':
      return {
        chapter_index: 1,
        chapter_goal: 'clear immediate objective',
        current_arc: 'manual_arc',
        pending_conflicts: ['resource_shortage'],
      };
    default:
      return {
        fact_id: 'fact_manual_3',
        subject: 'player',
        predicate: 'intent',
        object: 'explore',
      };
  }
};
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
  const decodeEscapedText = (value: string): string =>
    value
      .replace(/\\\\/g, '\\')
      .replace(/\\"/g, '"')
      .replace(/\\n/g, '\n')
      .replace(/\\t/g, '\t');
  const normalizeGeneratedBlock = (block: string): string => {
    const trimmed = block.trim();
    if (trimmed.length === 0) {
      return '';
    }
    if (!trimmed.includes('"text"')) {
      return trimmed;
    }
    const textField = trimmed.match(/"text"\s*:\s*"([\s\S]*?)"\s*(?:,|\})/);
    if (!textField || !textField[1]) {
      return trimmed;
    }
    return decodeEscapedText(textField[1]).trim();
  };
  const content = gameStore.plotState?.current_chapter?.content ?? [];
  const combined = content.length > 0 ? content.join('\n\n') : gameStore.currentScene?.description ?? '';
  return combined
    .split(/\n{2,}/)
    .map((text) => normalizeGeneratedBlock(text))
    .flatMap((text) => text.split(/\n{2,}/))
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
const shouldAutoFollowNewParagraph = computed(
  () => shouldAutoAdvance.value && !hasBlockingOverlay.value,
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
    || showShortcutsDialog.value
    || showQuickPanel.value
);
const optionSourceLabel = computed(() => {
  const source = gameStore.plotState?.last_option_generation_source;
  if (!source) {
    return '';
  }
  const labels: Record<string, string> = {
    llm_structured: '模型结构化',
    llm_regenerated: '模型再生成',
    rule_fallback: '规则回退',
    rule_fallback_latency_budget: '规则回退（时延预算）',
    previous_reused: '复用上一组选项',
    not_waiting_for_input: '当前无需输入',
    consistency_non_waiting_fallback: '一致性兜底自动推进',
  };
  return labels[source] ?? '未知来源';
});
const optionSourceHint = computed(() => {
  const source = gameStore.plotState?.last_option_generation_source ?? '';
  const diag = gameStore.plotState?.last_generation_diagnostics ?? '';
  if (source === 'rule_fallback_latency_budget') {
    return '受时延预算影响，已跳过模型选项再生成';
  }
  if (diag.includes('skipped(latency_budget)')) {
    return '部分增强步骤因时延预算被跳过';
  }
  return '';
});
const userFacingError = computed(() => {
  const raw = gameStore.error ?? '';
  if (!raw) {
    return null;
  }
  if (isDevMode) {
    return raw;
  }
  const isVerboseDiagnostics = raw.includes('；选项来源：')
    || raw.includes('；耗时(ms)：')
    || raw.includes('回退：')
    || raw.includes('已降级为骨架生成')
    || raw.includes('双通道生成：');
  if (isVerboseDiagnostics) {
    return '本轮剧情生成质量不稳定，建议检查 LLM 设置后重试。';
  }
  return raw;
});
const shouldShowLlmSetupShortcut = computed(() => {
  const err = userFacingError.value ?? '';
  return err.includes('选项续写未获取到 LLM 剧情文本');
});
const handleBackToMenu = () => {
  router.push('/');
};
const closeRuntimeQuickPanels = () => {
  showQuickPanel.value = false;
};
const openCharacterDialog = () => {
  playClick();
  closeAllDialogs();
  closeRuntimeQuickPanels();
  showCharacterInfo.value = true;
};
const openInfoDialog = () => {
  playClick();
  closeAllDialogs();
  closeRuntimeQuickPanels();
  showInfoTabs.value = true;
};
const openSaveDialog = () => {
  playClick();
  closeAllDialogs();
  closeRuntimeQuickPanels();
  showSaveDialog.value = true;
};
const openLoadDialog = () => {
  playClick();
  closeAllDialogs();
  closeRuntimeQuickPanels();
  showLoadDialog.value = true;
};
const openLlmDialogFromError = () => {
  playClick();
  closeAllDialogs();
  closeRuntimeQuickPanels();
  showLLMDialog.value = true;
};
const openQuickPanel = (tab: RuntimeQuickTab) => {
  playClick();
  closeAllDialogs();
  activeQuickPanelTab.value = tab;
  showQuickPanel.value = true;
};
watch(activeQuickPanelTab, (tab) => {
  try {
    localStorage.setItem(QUICK_PANEL_TAB_STORAGE_KEY, tab);
  } catch {
    // Ignore storage errors in restricted contexts.
  }
});
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
const interactionStateToneClass = computed(() => {
  if (plotInteractionState.value === 'resolving') return 'runtime-state-gold';
  if (plotInteractionState.value === 'waiting_for_free_text') return 'runtime-state-ink';
  if (plotInteractionState.value === 'cooldown') return 'runtime-state-ember';
  return 'runtime-state-cool';
});
const activeThemeClass = computed(() => {
  return 'theme-scroll';
});
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

  if (characterCreationDurationLabel.value) {
    out.push({
      id: `character-init-${characterCreationDurationLabel.value}`,
      kind: 'info',
      title: '角色创建完成',
      message: `创建耗时 ${characterCreationDurationLabel.value}`,
      priority: 'toast',
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

const openStorySettingsDialog = () => {
  playClick();
  showStorySettings.value = true;
};

const refreshWorldRegistryPanel = async () => {
  playClick();
  await gameStore.refreshWorldRegistry();
};
const resetWorldRegistryPatchTemplate = () => {
  playClick();
  worldRegistryPatchInput.value = worldRegistryPatchTemplate;
  worldRegistryPatchError.value = '';
};
const applyWorldRegistryPatchFromPanel = async () => {
  worldRegistryPatchError.value = '';
  let parsed: unknown;
  try {
    parsed = JSON.parse(worldRegistryPatchInput.value);
  } catch {
    worldRegistryPatchError.value = 'Patch JSON 解析失败，请检查格式。';
    return;
  }
  try {
    worldRegistryPatchSubmitting.value = true;
    await gameStore.applyWorldRegistryPatch(parsed);
  } catch (error) {
    worldRegistryPatchError.value = error instanceof Error ? error.message : String(error);
  } finally {
    worldRegistryPatchSubmitting.value = false;
  }
};
const appendRowToRegistryTable = async () => {
  worldRegistryRowError.value = '';
  let row: unknown;
  try {
    row = JSON.parse(worldRegistryRowInput.value);
  } catch {
    worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
    return;
  }
  if (!row || typeof row !== 'object' || Array.isArray(row)) {
    worldRegistryRowError.value = '行 JSON 必须是对象。';
    return;
  }
  const table = worldRegistrySelectedTable.value;
  const patch = { [table]: [row] };
  try {
    worldRegistryPatchSubmitting.value = true;
    await gameStore.applyWorldRegistryPatch(patch);
  } catch (error) {
    worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
  } finally {
    worldRegistryPatchSubmitting.value = false;
  }
};
const loadSelectedTableFirstRowTemplate = () => {
  playClick();
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
  const row = rows.length > 0 ? rows[0] : {};
  worldRegistryRowInput.value = JSON.stringify(row, null, 2);
};
const loadMinimalRowTemplateForSelectedTable = () => {
  playClick();
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  worldRegistryRowInput.value = JSON.stringify(buildMinimalRowTemplateForTable(table), null, 2);
};
const upsertRowByKeyInRegistryTable = async () => {
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  const keyField = worldRegistryKeyField.value.trim();
  if (!keyField) {
    worldRegistryRowError.value = '主键字段不能为空。';
    return;
  }
  let row: unknown;
  try {
    row = JSON.parse(worldRegistryRowInput.value);
  } catch {
    worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
    return;
  }
  if (!row || typeof row !== 'object' || Array.isArray(row)) {
    worldRegistryRowError.value = '行 JSON 必须是对象。';
    return;
  }
  const patch = { [table]: [{ __op: 'upsert_by_key', __key_field: keyField, row }] };
  try {
    worldRegistryPatchSubmitting.value = true;
    await gameStore.applyWorldRegistryPatch(patch);
  } catch (error) {
    worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
  } finally {
    worldRegistryPatchSubmitting.value = false;
  }
};
const loadSelectedTableRowByIndex = () => {
  playClick();
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
  const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
  if (idx >= rows.length) {
    worldRegistryRowError.value = `索引越界：${idx}，当前 ${table} 共 ${rows.length} 行。`;
    return;
  }
  worldRegistryRowInput.value = JSON.stringify(rows[idx], null, 2);
};
const replaceRowInRegistryTable = async () => {
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
  let row: unknown;
  try {
    row = JSON.parse(worldRegistryRowInput.value);
  } catch {
    worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
    return;
  }
  if (!row || typeof row !== 'object' || Array.isArray(row)) {
    worldRegistryRowError.value = '行 JSON 必须是对象。';
    return;
  }
  const patch = { [table]: [{ __op: 'replace', __index: idx, row }] };
  try {
    worldRegistryPatchSubmitting.value = true;
    await gameStore.applyWorldRegistryPatch(patch);
  } catch (error) {
    worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
  } finally {
    worldRegistryPatchSubmitting.value = false;
  }
};
const deleteRowInRegistryTable = async () => {
  worldRegistryRowError.value = '';
  const table = worldRegistrySelectedTable.value;
  const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
  const patch = { [table]: [{ __op: 'delete', __index: idx }] };
  try {
    worldRegistryPatchSubmitting.value = true;
    await gameStore.applyWorldRegistryPatch(patch);
  } catch (error) {
    worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
  } finally {
    worldRegistryPatchSubmitting.value = false;
  }
};
watch(worldRegistrySelectedTable, (table) => {
  worldRegistryRowPage.value = 0;
  worldRegistryKeyField.value = worldRegistryDefaultKeyMap[table] ?? 'id';
  worldRegistryRowInput.value = JSON.stringify(buildMinimalRowTemplateForTable(table), null, 2);
});

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

// 仅在自动推进阶段跟随新段落，避免手动阅读时强制跳动到底部
watch(currentChapterParagraphs, (newParagraphs) => {
  if (
    shouldAutoFollowNewParagraph.value
    && newParagraphs.length > previousChapterParagraphs.value.length
  ) {
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
    closeRuntimeQuickPanels();
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
</script>

<style scoped>
.game-shell {
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'STSong', 'SimSun', serif;
  background:
    radial-gradient(circle at 10% 22%, color-mix(in srgb, var(--ink-title-color) 22%, transparent), transparent 34%),
    radial-gradient(circle at 82% 8%, rgba(59, 122, 107, 0.08), transparent 28%),
    linear-gradient(145deg, var(--ink-paper), var(--ink-paper-elevated));
}

.runtime-topbar {
  height: 56px;
  border-radius: 12px;
  border: 1px solid var(--ink-border-soft);
  border-bottom-color: #d0c5b5;
  background: var(--ink-card-bg);
  padding: 0 18px;
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 14px;
}

.runtime-top-left {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-self: start;
}

.runtime-brand {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: 0.08em;
}

.runtime-seal-btn {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: 1px solid var(--ink-accent-main);
  background: color-mix(in srgb, var(--ink-accent-main) 14%, transparent);
  color: var(--ink-accent-main);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-top-center {
  display: flex;
  align-items: center;
  gap: 10px;
}

.runtime-top-main {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
  font-size: 18px;
  line-height: 1.2;
}

.runtime-chapter-number {
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 700;
}

.runtime-chapter-name {
  color: var(--ink-text-primary);
  font-size: 18px;
  font-weight: 600;
}

.runtime-state-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 16px;
  border: 1px solid var(--ink-border-accent);
  background: var(--ink-card-bg);
  padding: 3px 10px;
}

.runtime-state-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: currentColor;
  opacity: 0.9;
}

.runtime-state-label {
  font-size: 12px;
  font-weight: 600;
}

.runtime-state-cool {
  color: var(--ink-text-cool);
}

.runtime-state-gold {
  color: var(--ink-title-color);
}

.runtime-state-ink {
  color: var(--ink-text-ink);
}

.runtime-state-ember {
  color: var(--ink-accent-main);
}

.runtime-top-right {
  justify-self: end;
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.runtime-resource-pill {
  border-radius: 999px;
  border: 1px solid var(--ink-border-accent);
  background: #f8f4ec;
  color: var(--ink-title-color);
  padding: 4px 11px;
  font-size: 12px;
  line-height: 1.3;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.5);
}

.runtime-content {
  margin-top: 18px;
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr) 280px;
  gap: 20px;
}

.runtime-panel {
  min-height: 0;
  min-width: 0;
}

.runtime-card,
.runtime-main-panel {
  position: relative;
  border-radius: 14px;
  border: 1px solid var(--ink-border-strong);
  background: var(--ink-card-bg);
  box-shadow: var(--ink-shadow-card);
}

.runtime-card {
  padding: 20px;
  background-image: radial-gradient(circle at 80% 14%, rgba(59, 122, 107, 0.04), transparent 30%);
}

.runtime-side-left .runtime-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.runtime-side-left {
  display: grid;
  align-content: start;
  gap: 22px;
}

.runtime-card-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: 1.35;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-chapter-title {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
}

.runtime-sub-text {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.6;
  letter-spacing: 0.01em;
}

.runtime-root-line {
  margin-top: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.runtime-root-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.runtime-root-icon {
  display: inline-flex;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
}

.runtime-root-name {
  font-size: 14px;
  line-height: 1.4;
}

.runtime-root-type {
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.5;
}

.runtime-root-earth {
  color: #8d6a46;
}

.runtime-root-metal {
  color: var(--ink-title-color);
}

.runtime-root-wood {
  color: var(--ink-text-ink);
}

.runtime-root-water {
  color: #3b6f9b;
}

.runtime-root-fire {
  color: var(--ink-accent-main);
}

.runtime-body-text {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: 16px;
  line-height: 1.65;
  letter-spacing: 0.005em;
}

.runtime-accent-cool {
  color: var(--ink-text-cool);
}

.runtime-rhythm-badge {
  margin-top: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--ink-border-accent);
  border-radius: 16px;
  background: var(--ink-card-bg);
  padding: 4px 12px;
}

.runtime-rhythm-badge-compact {
  margin-top: 0;
  padding: 3px 10px;
}

.runtime-rhythm-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.runtime-rhythm-label {
  color: var(--ink-text-muted);
  font-size: 12px;
  letter-spacing: 0.02em;
}

.runtime-rhythm-value {
  font-size: 14px;
  font-weight: 700;
}

.runtime-rhythm-cool .runtime-rhythm-icon,
.runtime-rhythm-cool .runtime-rhythm-value {
  color: var(--ink-text-cool);
}

.runtime-rhythm-gold .runtime-rhythm-icon,
.runtime-rhythm-gold .runtime-rhythm-value {
  color: var(--ink-title-color);
}

.runtime-rhythm-ink .runtime-rhythm-icon,
.runtime-rhythm-ink .runtime-rhythm-value {
  color: var(--ink-text-ink);
}

.runtime-main-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 24px;
  border-color: #ddcfbc;
  background: var(--ink-paper);
  background-image:
    radial-gradient(circle at 10% 88%, rgba(178, 62, 62, 0.035), transparent 35%),
    radial-gradient(circle at 88% 16%, rgba(59, 122, 107, 0.04), transparent 30%);
}

.runtime-main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0 16px;
  margin-bottom: 8px;
}

.runtime-main-scene {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

.runtime-main-chapter {
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-main-scene-icon {
  color: var(--ink-title-color);
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.runtime-main-scene-name {
  color: var(--ink-text-primary);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.005em;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-main-body {
  min-height: 0;
  flex: 1;
  padding: 0;
}

.runtime-main-body :deep(.runtime-story-scroll) {
  height: 100%;
  min-height: 420px;
  max-height: none;
  overflow-x: hidden;
  border-radius: 12px;
  border: 1px solid var(--ink-border-strong);
  background: var(--ink-card-bg);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  padding: 24px 24px 90px;
}

.runtime-main-body :deep(.runtime-story-scroll::-webkit-scrollbar) {
  width: 8px;
}

.runtime-main-body :deep(.runtime-story-scroll::-webkit-scrollbar-track) {
  border-radius: 4px;
  background: var(--ink-card-bg-muted);
}

.runtime-main-body :deep(.runtime-story-scroll::-webkit-scrollbar-thumb) {
  border-radius: 4px;
  background: var(--ink-border-accent);
}

.runtime-main-body :deep(.runtime-story-scroll) {
  scrollbar-width: thin;
  scrollbar-color: var(--ink-border-accent) var(--ink-card-bg-muted);
}

.runtime-main-body :deep([data-paragraph-index]) {
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-side-right .runtime-interaction-card {
  height: 100%;
  overflow: auto;
  overflow-x: hidden;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.runtime-interaction-title,
.runtime-interaction-subtitle {
  margin: 0;
}

.runtime-interaction-title {
  font-size: 18px;
  line-height: 1.35;
}

.runtime-interaction-subtitle {
  font-size: 14px;
  line-height: 1.6;
}

.runtime-interaction-card :deep(.mx-auto.max-w-3xl) {
  max-width: none;
  margin: 0;
}

.runtime-interaction-card :deep(.ink-interaction-panel) {
  padding: 0;
}

.runtime-interaction-card :deep(.free-text-input),
.runtime-interaction-card :deep(.free-text-foot),
.runtime-interaction-card :deep(.free-text-validation) {
  max-width: 100%;
}

.runtime-llm-shortcut {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-top: 1px dashed var(--ink-border-accent);
  padding-top: 10px;
}

.runtime-bottom-bar {
  margin-top: 18px;
  min-height: 64px;
  border-top: 1px solid #d9cbb8;
  border-radius: 12px;
  background: var(--ink-card-bg-soft);
  padding: 11px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.runtime-bottom-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.runtime-bottom-bar :deep(.ink-dock) {
  gap: 8px;
  padding: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
}

.runtime-bottom-bar :deep(.ink-dock-btn) {
  min-width: 60px;
  border-radius: 12px;
  border-color: #c6b295;
  background: linear-gradient(180deg, #f8f4ec, #efe7da);
  padding: 7px 14px;
  font-size: 14px;
  line-height: 1.1;
}

.runtime-bottom-bar :deep(.ink-dock-btn:hover) {
  border-color: #b78c4a;
  background: linear-gradient(180deg, #fdf9f1, #f3ebdf);
  box-shadow: 0 4px 12px rgba(45, 42, 36, 0.12);
}

.runtime-bottom-bar :deep(.ink-dock-btn-primary) {
  border-color: #b68b46;
  background: linear-gradient(180deg, #f4ecd9, #ead7b5);
  color: #6b4f2f;
}

.runtime-bottom-btn {
  border-radius: 8px;
  border: 1px solid var(--ink-border-accent);
  background: #f8f3ea;
  color: var(--ink-text-primary);
  padding: 8px 18px;
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-bottom-btn:hover,
.runtime-seal-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
  box-shadow: 0 3px 10px rgba(45, 42, 36, 0.1);
}

.runtime-bottom-btn:active,
.runtime-seal-btn:active {
  transform: scale(0.98);
}

@media (max-width: 1180px) {
  .runtime-content {
    grid-template-columns: 1fr;
  }

  .runtime-side-right .runtime-interaction-card {
    height: auto;
    padding: 16px;
    gap: 6px;
  }

  .runtime-topbar {
    grid-template-columns: 1fr;
    height: auto;
    padding: 10px 14px;
  }

  .runtime-top-right {
    justify-self: start;
  }

  .runtime-bottom-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .runtime-main-header {
    flex-wrap: wrap;
    gap: 8px;
    padding: 0 0 12px;
  }

  .runtime-main-body {
    padding: 0;
  }

  .runtime-main-panel {
    padding: 18px;
  }
}
</style>

