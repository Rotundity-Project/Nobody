<template>
  <div class="main-menu-shell min-h-screen px-4 py-8 sm:px-6 sm:py-10" :class="activeThemeClass">
    <div class="main-menu-guide">
      建议流程：新游戏 -> 选择剧本 -> 进入主界面推进剧情
    </div>

    <div class="main-menu-frame mx-auto w-full max-w-[1320px]">
      <aside class="menu-left-rail">
        <button
          type="button"
          data-testid="new-game-btn"
          class="menu-btn menu-btn-primary"
          @click="handleNewGame"
        >
          新游戏
        </button>
        <button
          type="button"
          data-testid="continue-game-btn"
          class="menu-btn"
          :aria-expanded="showSavePanel ? 'true' : 'false'"
          aria-controls="recent-save-card"
          @click="openLoadDialog"
        >
          读取存档
        </button>
        <button
          type="button"
          data-testid="llm-settings-btn"
          class="menu-btn"
          @click="handleSettings"
        >
          LLM 设置
        </button>
        <button
          type="button"
          data-testid="open-audio-btn"
          class="menu-btn"
          :aria-label="audioPanelToggleAriaLabel"
          :aria-describedby="audioPanelToggleDescribedBy"
          aria-controls="main-menu-audio-panel"
          :aria-expanded="showAudioPanel ? 'true' : 'false'"
          @click="toggleAudioPanel"
        >
          {{ showAudioPanel ? '收起游戏设置' : '游戏设置' }}
        </button>
        <p
          id="quick-volume-status"
          data-testid="quick-volume-status"
          class="menu-volume-pill"
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {{ quickMasterVolume <= 0 ? '已静音' : `当前 ${Math.round(quickMasterVolume * 100)}%` }}
        </p>
      </aside>

      <main class="menu-main-stage">
        <header class="main-menu-header">
          <p class="main-menu-en-title">NOBODY</p>
          <p class="main-menu-cn-title">小人物</p>
          <button
            type="button"
            class="main-menu-seal"
            title="轻触印章，观心问路"
            @click="handleMainSealClick"
          >
            <span>无名印</span>
          </button>
        </header>
        <p v-if="mainSealWhisper" class="main-menu-seal-whisper">{{ mainSealWhisper }}</p>

        <section class="menu-card mt-6">
          <h2 class="menu-card-title">最近存档</h2>
          <p
            v-if="latestSave"
            id="latest-save-summary"
            data-testid="latest-save-summary"
            class="menu-text-primary mt-2 text-sm leading-6"
            aria-live="polite"
            aria-atomic="true"
          >
            槽位 {{ latestSave.slot_id }} · {{ latestSavePlayerLabel }} · {{ latestSaveRealmLabel }}
            <span class="mt-1 flex flex-wrap items-center gap-1.5">
              <span
                v-if="showLatestSaveLocationTag"
                class="menu-chip"
              >
                位置：{{ latestSaveLocationLabel }}
              </span>
              <span class="menu-chip">
                时间：{{ latestSaveGameTimeLabel }}
              </span>
            </span>
            <span class="menu-text-muted mt-1 block text-xs">
              <template v-if="hasLatestSaveTimestamp">
                最近保存：{{ latestSaveTimestampLabel }}（{{ latestSaveAgeLabel }}）
              </template>
              <template v-else>
                最近保存：时间未知
              </template>
            </span>
          </p>
          <p
            v-else-if="recentSaveLoading"
            id="loading-save-hint"
            data-testid="loading-save-hint"
            class="menu-text-muted mt-2 text-sm leading-6"
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            正在读取最近存档...
          </p>
          <p
            v-else
            id="no-save-hint"
            data-testid="no-save-hint"
            class="menu-text-muted mt-2 text-sm leading-6"
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            暂无可用存档，可先开始新游戏创建进度。
          </p>
          <p
            v-if="recentSaveError"
            id="recent-save-error"
            data-testid="recent-save-error"
            class="menu-text-error mt-2 text-sm"
            role="alert"
            aria-live="assertive"
            aria-atomic="true"
          >
            {{ recentSaveError }}
          </p>
          <p
            v-if="shouldShowRefreshStatus && lastRefreshStatusLabel"
            id="recent-save-refresh-status"
            data-testid="recent-save-refresh-status"
            class="mt-1 text-xs"
            :class="refreshStatusToneClass"
            :role="refreshStatusRole"
            :aria-live="refreshStatusAriaLive"
            aria-atomic="true"
          >
            刷新状态：{{ lastRefreshStatusLabel }}
          </p>
          <p
            v-if="lastRefreshLabel"
            id="recent-save-refresh-label"
            data-testid="recent-save-refresh-label"
            class="menu-text-muted mt-1 text-xs"
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            最近刷新：{{ lastRefreshLabel }}（{{ relativeRefreshLabel }}）
          </p>
        </section>
      </main>
    </div>

    <div
      v-if="showSavePanel"
      class="menu-overlay"
      @click.self="showSavePanel = false"
    >
      <section
        id="recent-save-card"
        data-testid="recent-save-card"
        class="menu-dialog"
        :aria-busy="recentSaveLoading ? 'true' : 'false'"
      >
        <header class="menu-dialog-header">
          <h2 class="menu-card-title">读取存档</h2>
          <button type="button" class="menu-inline-btn" @click="showSavePanel = false">关闭</button>
        </header>
        <div class="menu-dialog-body">
          <aside class="menu-dialog-left">
            <p id="save-actions-heading" class="menu-sub-title">存档槽位</p>
            <div
              data-testid="save-actions-group"
              class="mt-2 grid gap-2"
              role="group"
              aria-labelledby="save-actions-heading"
              :aria-describedby="saveActionsGroupDescribedBy"
            >
              <button
                v-for="slot in saveSlots"
                :key="slot.slot_id"
                type="button"
                class="menu-chip-btn text-left"
                :class="selectedSaveSlotId === slot.slot_id ? 'menu-chip-btn-active' : ''"
                @click="selectedSaveSlotId = slot.slot_id"
              >
                槽位 {{ slot.slot_id }} · {{ normalizeSaveText(slot.player_name, '未命名角色') }}
              </button>
              <p v-if="saveSlots.length === 0" class="menu-text-muted text-xs">暂无存档槽位</p>
            </div>
            <button
              type="button"
              data-testid="refresh-save-btn"
              class="menu-btn mt-3 text-sm disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="recentSaveLoading || quickLoadPending"
              :aria-label="refreshActionAriaLabel"
              aria-controls="recent-save-card"
              :aria-describedby="refreshActionDescribedBy"
              @click="fetchLatestSave"
            >
              {{ recentSaveLoading ? '刷新中...' : '刷新存档' }}
            </button>
            <button
              type="button"
              class="menu-inline-btn mt-2"
              @click="showSavePanel = true"
            >
              打开完整存档列表
            </button>
          </aside>
          <section class="menu-dialog-right">
            <p class="menu-sub-title">存档信息</p>
            <p v-if="selectedSaveSlot" class="menu-text-primary mt-2 text-sm leading-6">
              槽位 {{ selectedSaveSlot.slot_id }} · {{ normalizeSaveText(selectedSaveSlot.player_name, '未命名角色') }}
              · {{ normalizeSaveText(selectedSaveSlot.realm, '境界未知') }}
              <span class="menu-text-muted mt-1 block text-xs">
                <template v-if="formatLocationLabel(selectedSaveSlot.location) !== '未知'">
                  位置：{{ formatLocationLabel(selectedSaveSlot.location) }} ·
                </template>
                时间：{{ normalizeSaveText(selectedSaveSlot.game_time, '时间未知') }}
              </span>
            </p>
            <p v-else class="menu-text-muted mt-2 text-sm">请选择左侧存档槽。</p>
            <div class="mt-4 flex flex-wrap gap-2">
              <button
                type="button"
                data-testid="recent-save-btn"
                class="menu-btn menu-btn-primary text-sm disabled:cursor-not-allowed disabled:opacity-60"
                :disabled="!latestSave || quickLoadPending || recentSaveLoading"
                :aria-label="recentSaveActionAriaLabel"
                :aria-describedby="recentSaveActionDescribedBy"
                @click="loadLatestSave"
              >
                {{ quickLoadPending ? '加载中...' : '继续最近存档' }}
              </button>
              <button
                v-if="recentSaveError && !recentSaveLoading"
                type="button"
                data-testid="retry-load-saves-btn"
                class="menu-inline-btn"
                :aria-label="retryLoadSavesAriaLabel"
                :aria-describedby="retryLoadSavesDescribedBy"
                @click="fetchLatestSave"
              >
                重试读取
              </button>
            </div>
          </section>
        </div>
      </section>
    </div>

    <div
      v-if="showAudioPanel"
      id="main-menu-audio-panel"
      class="menu-overlay"
      @click.self="showAudioPanel = false"
    >
      <section class="menu-dialog">
        <header class="menu-dialog-header">
          <h2 class="menu-card-title">游戏设置</h2>
          <button type="button" class="menu-inline-btn" @click="showAudioPanel = false">关闭</button>
        </header>
        <div class="menu-dialog-body">
          <aside class="menu-dialog-left">
            <p id="quick-audio-heading" class="menu-sub-title">设置分组</p>
            <div class="mt-2 grid gap-2">
              <button
                type="button"
                class="menu-chip-btn menu-chip-btn-active text-left"
              >
                音量控制
              </button>
            </div>
          </aside>
          <section class="menu-dialog-right">
            <div class="menu-sub-card mt-4">
              <AudioControlPanel />
            </div>
          </section>
        </div>
      </section>
    </div>
    <div
      v-if="showLLMDialog"
      id="main-menu-llm-panel"
      class="menu-overlay"
      @click.self="showLLMDialog = false"
    >
      <section class="menu-dialog">
        <header class="menu-dialog-header">
          <h2 class="menu-card-title">LLM 设置</h2>
          <button type="button" class="menu-inline-btn" @click="showLLMDialog = false">关闭</button>
        </header>
        <div class="menu-dialog-body">
          <aside class="menu-dialog-left">
            <p class="menu-sub-title">设置分组</p>
            <div class="mt-2 grid gap-2">
              <button
                type="button"
                class="menu-chip-btn menu-chip-btn-active text-left"
              >
                模型配置
              </button>
            </div>
          </aside>
          <section class="menu-dialog-right">
            <div class="menu-sub-card">
              <LLMConfigDialog :is-open="showLLMDialog" inline @close="showLLMDialog = false" />
            </div>
          </section>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import AudioControlPanel from './AudioControlPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import { getAudioSettings, playClick } from '../utils/audioSystem';
import { useGameStore } from '../stores/gameStore';
import type { SaveInfo } from '../types/game';
import { formatLocationLabel } from '../shared/locationLabel';
import { getUiTheme } from '../utils/uiTheme';

const router = useRouter();
const gameStore = useGameStore();

const showLLMDialog = ref(false);
const activeThemeClass = ref(getUiTheme());
const showAudioPanel = ref(false);
const showSavePanel = ref(true);
const mainSealWhisper = ref('');
const recentSaveLoading = ref(false);
const quickLoadPending = ref(false);
const recentSaveError = ref('');
const latestSave = ref<SaveInfo | null>(null);
const saveSlots = ref<SaveInfo[]>([]);
const selectedSaveSlotId = ref<number | null>(null);
const lastRefreshAt = ref<number | null>(null);
const lastRefreshSucceeded = ref<boolean | null>(null);
const refreshNowTick = ref(Date.now());
let refreshTickerId: number | null = null;
let mainSealWhisperTimer: number | null = null;
const REFRESH_SUCCESS_TOAST_MS = 8000;
const quickMasterVolume = ref(getAudioSettings().master);
const mainSealWhispers = [
  '无名印启：先立本心，再入尘世。',
  '印文有言：不争其名，自得其道。',
  '朱印轻鸣：凡人亦可问天路。',
];

const handleMainSealClick = () => {
  playClick();
  const idx = Math.floor(Math.random() * mainSealWhispers.length);
  mainSealWhisper.value = mainSealWhispers[idx];
  if (mainSealWhisperTimer != null) {
    window.clearTimeout(mainSealWhisperTimer);
  }
  mainSealWhisperTimer = window.setTimeout(() => {
    mainSealWhisper.value = '';
    mainSealWhisperTimer = null;
  }, 2200);
};

const normalizeSaveText = (value: string | null | undefined, fallback: string): string => {
  if (!value) {
    return fallback;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : fallback;
};
const latestSavePlayerLabel = computed(() =>
  normalizeSaveText(latestSave.value?.player_name, '未命名角色'),
);
const latestSaveRealmLabel = computed(() =>
  normalizeSaveText(latestSave.value?.realm, '境界未知'),
);
const latestSaveGameTimeLabel = computed(() =>
  normalizeSaveText(latestSave.value?.game_time, '时间未知'),
);
const latestSaveLocationLabel = computed(() => formatLocationLabel(latestSave.value?.location));
const showLatestSaveLocationTag = computed(
  () => latestSaveLocationLabel.value.trim().length > 0 && latestSaveLocationLabel.value !== '未知',
);
const selectedSaveSlot = computed(() => {
  if (!selectedSaveSlotId.value) {
    return latestSave.value;
  }
  return saveSlots.value.find((slot) => slot.slot_id === selectedSaveSlotId.value) ?? latestSave.value;
});
const latestSaveTimestampLabel = computed(() => {
  const ts = latestSave.value?.timestamp;
  if (!ts || !Number.isFinite(ts)) {
    return '未知';
  }
  const date = new Date(ts * 1000);
  if (Number.isNaN(date.getTime())) {
    return '未知';
  }
  return date.toLocaleString();
});
const hasLatestSaveTimestamp = computed(() => latestSaveTimestampLabel.value !== '未知');
const latestSaveAgeLabel = computed(() => {
  const ts = latestSave.value?.timestamp;
  if (!ts || !Number.isFinite(ts)) {
    return '时间未知';
  }
  const diffSeconds = Math.max(0, Math.floor((refreshNowTick.value - ts * 1000) / 1000));
  if (diffSeconds < 60) {
    return `${diffSeconds} 秒前`;
  }
  const diffMinutes = Math.floor(diffSeconds / 60);
  if (diffMinutes < 60) {
    return `${diffMinutes} 分钟前`;
  }
  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours} 小时前`;
  }
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays} 天前`;
});
const lastRefreshLabel = computed(() => {
  if (!lastRefreshAt.value) {
    return '';
  }
  const date = new Date(lastRefreshAt.value);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  return date.toLocaleTimeString();
});
const relativeRefreshLabel = computed(() => {
  if (!lastRefreshAt.value) {
    return '';
  }
  const diffMs = Math.max(0, refreshNowTick.value - lastRefreshAt.value);
  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 3) {
    return '刚刚';
  }
  if (seconds < 60) {
    return `${seconds} 秒前`;
  }
  const minutes = Math.floor(seconds / 60);
  return `${minutes} 分钟前`;
});
const lastRefreshStatusLabel = computed(() => {
  if (recentSaveLoading.value) {
    return '刷新中';
  }
  if (lastRefreshSucceeded.value == null) {
    return '';
  }
  return lastRefreshSucceeded.value ? '成功' : '失败';
});
const shouldShowRefreshStatus = computed(() => {
  if (recentSaveLoading.value) {
    return true;
  }
  if (lastRefreshSucceeded.value === false) {
    return true;
  }
  if (lastRefreshSucceeded.value === true && lastRefreshAt.value) {
    return refreshNowTick.value - lastRefreshAt.value < REFRESH_SUCCESS_TOAST_MS;
  }
  return false;
});
const isRefreshStatusSuccess = computed(
  () => !recentSaveLoading.value && lastRefreshSucceeded.value === true,
);
const refreshStatusToneClass = computed(() => {
  if (recentSaveLoading.value) {
    return 'text-sky-300';
  }
  return isRefreshStatusSuccess.value ? 'text-emerald-300' : 'text-amber-300';
});
const refreshStatusRole = computed(() =>
  recentSaveLoading.value || isRefreshStatusSuccess.value ? 'status' : 'alert',
);
const refreshStatusAriaLive = computed(() =>
  recentSaveLoading.value || isRefreshStatusSuccess.value ? 'polite' : 'assertive',
);
const refreshActionDescribedBy = computed(() => {
  const ids: string[] = [];
  if (lastRefreshLabel.value) {
    ids.push('recent-save-refresh-label');
  }
  if (recentSaveError.value) {
    ids.push('recent-save-error');
  }
  if (shouldShowRefreshStatus.value && lastRefreshStatusLabel.value) {
    ids.push('recent-save-refresh-status');
  }
  return ids.length > 0 ? ids.join(' ') : undefined;
});
const refreshActionAriaLabel = computed(() => {
  if (recentSaveLoading.value) {
    return '刷新存档，当前正在刷新中';
  }
  if (lastRefreshSucceeded.value === true) {
    const absoluteLabel = lastRefreshLabel.value ? `时间 ${lastRefreshLabel.value}` : '时间未知';
    const relativeLabel = relativeRefreshLabel.value || '刚刚';
    return `刷新存档，最近一次刷新成功，${absoluteLabel}，${relativeLabel}`;
  }
  if (lastRefreshSucceeded.value === false) {
    const absoluteLabel = lastRefreshLabel.value ? `时间 ${lastRefreshLabel.value}` : '时间未知';
    const relativeLabel = relativeRefreshLabel.value || '时间未知';
    return `刷新存档，最近一次刷新失败，${absoluteLabel}，${relativeLabel}`;
  }
  return '刷新存档，尚未执行刷新';
});
const recentSaveActionAriaLabel = computed(() => {
  if (quickLoadPending.value) {
    return '继续最近存档，当前正在加载';
  }
  if (recentSaveLoading.value) {
    return '继续最近存档，正在读取可用存档';
  }
  if (!latestSave.value) {
    return '继续最近存档，当前没有可用存档';
  }
  return `继续最近存档，槽位 ${latestSave.value.slot_id}，${latestSavePlayerLabel.value}，${latestSaveRealmLabel.value}`;
});
const recentSaveActionDescribedBy = computed(() => {
  const ids: string[] = [];
  if (recentSaveError.value) {
    ids.push('recent-save-error');
  }
  if (latestSave.value) {
    ids.push('latest-save-summary');
  } else {
    ids.push('no-save-hint');
  }
  return ids.join(' ');
});
const saveActionsGroupDescribedBy = computed(() => {
  const ids: string[] = [];
  if (recentSaveError.value) {
    ids.push('recent-save-error');
  }
  if (recentSaveLoading.value) {
    ids.push('loading-save-hint');
  } else if (latestSave.value) {
    ids.push('latest-save-summary');
  } else {
    ids.push('no-save-hint');
  }
  if (lastRefreshLabel.value) {
    ids.push('recent-save-refresh-label');
  }
  if (shouldShowRefreshStatus.value && lastRefreshStatusLabel.value) {
    ids.push('recent-save-refresh-status');
  }
  return ids.join(' ');
});
const audioPanelToggleDescribedBy = computed(() => 'quick-volume-status');
const audioPanelToggleAriaLabel = computed(() => {
  const volumeLabel = quickMasterVolume.value <= 0
    ? '当前已静音'
    : `当前音量 ${Math.round(quickMasterVolume.value * 100)}%`;
  const actionLabel = showAudioPanel.value ? '收起音量控制' : '展开音量控制';
  return `${actionLabel}，${volumeLabel}`;
});
const retryLoadSavesDescribedBy = computed(() => {
  const ids = ['recent-save-error'];
  if (lastRefreshLabel.value) {
    ids.push('recent-save-refresh-label');
  }
  if (shouldShowRefreshStatus.value && lastRefreshStatusLabel.value) {
    ids.push('recent-save-refresh-status');
  }
  return ids.join(' ');
});
const retryLoadSavesAriaLabel = computed(() => {
  if (!recentSaveError.value) {
    return '重试读取最近存档';
  }
  return `重试读取最近存档，最近错误：${recentSaveError.value}`;
});

const fetchLatestSave = async () => {
  recentSaveLoading.value = true;
  recentSaveError.value = '';
  try {
    const slots = await gameStore.listSaveSlots();
    if (!Array.isArray(slots) || slots.length === 0) {
      latestSave.value = null;
      saveSlots.value = [];
      selectedSaveSlotId.value = null;
      return;
    }
    const sorted = [...slots].sort((a, b) => b.timestamp - a.timestamp);
    saveSlots.value = sorted;
    latestSave.value = sorted[0] ?? null;
    if (selectedSaveSlotId.value == null || !sorted.some((slot) => slot.slot_id === selectedSaveSlotId.value)) {
      selectedSaveSlotId.value = sorted[0]?.slot_id ?? null;
    }
  } catch (error) {
    latestSave.value = null;
    saveSlots.value = [];
    selectedSaveSlotId.value = null;
    recentSaveError.value = error instanceof Error ? error.message : '读取最近存档失败';
  } finally {
    recentSaveLoading.value = false;
    lastRefreshAt.value = Date.now();
    lastRefreshSucceeded.value = recentSaveError.value.length === 0;
  }
};

const syncQuickVolumeFromSettings = () => {
  const settings = getAudioSettings();
  const clamped = Math.min(1, Math.max(0, settings.master));
  quickMasterVolume.value = clamped;
};

const handleNewGame = () => {
  playClick();
  router.push('/script-select');
};

const openLoadDialog = () => {
  playClick();
  showSavePanel.value = true;
  void fetchLatestSave();
};

const handleSettings = () => {
  playClick();
  showLLMDialog.value = true;
};

const toggleAudioPanel = () => {
  playClick();
  syncQuickVolumeFromSettings();
  showAudioPanel.value = !showAudioPanel.value;
};

const loadLatestSave = async () => {
  if (!latestSave.value) {
    return;
  }
  playClick();
  quickLoadPending.value = true;
  recentSaveError.value = '';
  try {
    await gameStore.loadGame(latestSave.value.slot_id);
    router.push('/game');
  } catch (error) {
    recentSaveError.value = error instanceof Error ? error.message : '加载最近存档失败';
  } finally {
    quickLoadPending.value = false;
  }
};

onMounted(() => {
  syncQuickVolumeFromSettings();
  refreshNowTick.value = Date.now();
  refreshTickerId = window.setInterval(() => {
    refreshNowTick.value = Date.now();
  }, 1000);
  window.addEventListener('focus', syncQuickVolumeFromSettings);
  void fetchLatestSave();
});

onUnmounted(() => {
  if (refreshTickerId != null) {
    window.clearInterval(refreshTickerId);
  }
  if (mainSealWhisperTimer != null) {
    window.clearTimeout(mainSealWhisperTimer);
  }
  window.removeEventListener('focus', syncQuickVolumeFromSettings);
});
</script>

<style scoped>
.main-menu-shell {
  font-family: 'Noto Serif SC', 'STKaiti', 'KaiTi', serif;
  color: var(--ink-text-primary);
  background:
    radial-gradient(circle at 12% 20%, color-mix(in srgb, var(--ink-title-color) 24%, transparent), transparent 35%),
    radial-gradient(circle at 85% 6%, color-mix(in srgb, var(--ink-text-cool) 18%, transparent), transparent 30%),
    linear-gradient(145deg, var(--ink-paper), var(--ink-paper-elevated));
}

.main-menu-guide {
  max-width: 1320px;
  margin: 0 auto 10px auto;
  text-align: right;
  font-size: 12px;
  color: var(--ink-text-cool);
  text-decoration: underline;
  text-underline-offset: 4px;
  text-decoration-color: color-mix(in srgb, var(--ink-text-cool) 45%, transparent);
}

.main-menu-frame {
  display: grid;
  grid-template-columns: 240px 1fr;
  gap: 24px;
  align-items: start;
}

.menu-left-rail {
  position: sticky;
  top: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.menu-main-stage {
  min-height: 72vh;
  background: color-mix(in srgb, var(--ink-paper) 90%, transparent);
  border: 1px solid var(--ink-border-soft);
  border-radius: 16px;
  box-shadow: var(--ink-shadow-panel);
  padding: 24px;
}

.main-menu-header {
  position: relative;
  text-align: center;
  padding: 18px 0 8px;
}

.main-menu-en-title {
  margin: 0;
  font-family: 'ZCOOL QingKe HuangYou', 'Cinzel', 'Noto Serif SC', serif;
  font-size: clamp(3.3rem, 8vw, 6.2rem);
  line-height: 1;
  letter-spacing: 0.06em;
  color: var(--ink-text-primary);
}

.main-menu-cn-title {
  margin: 10px 0 0;
  font-size: clamp(1.3rem, 2.3vw, 2rem);
  color: var(--ink-text-muted);
}

.main-menu-seal {
  position: absolute;
  right: 6%;
  top: 8px;
  width: 88px;
  height: 88px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--ink-accent-seal);
  border-radius: 6px;
  transform: rotate(-9deg) translateY(0);
  color: var(--ink-accent-seal);
  background: color-mix(in srgb, var(--ink-accent-seal) 8%, transparent);
  font-size: 14px;
  letter-spacing: 0.08em;
  box-shadow: 0 3px 12px color-mix(in srgb, var(--ink-accent-seal) 30%, transparent);
  transition: transform 180ms ease, box-shadow 180ms ease, background-color 180ms ease;
  animation: main-seal-float 2.8s ease-in-out infinite;
}

.main-menu-seal::before {
  content: '';
  position: absolute;
  inset: -5px;
  border: 1px solid color-mix(in srgb, var(--ink-accent-seal) 40%, transparent);
  border-radius: 10px;
  opacity: 0;
  transform: scale(0.92);
  transition: opacity 180ms ease, transform 180ms ease;
}

.main-menu-seal:hover {
  transform: rotate(-9deg) translateY(-2px);
  background: color-mix(in srgb, var(--ink-accent-seal) 14%, transparent);
  box-shadow: 0 8px 18px color-mix(in srgb, var(--ink-accent-seal) 34%, transparent);
}

.main-menu-seal:hover::before,
.main-menu-seal:focus-visible::before {
  opacity: 1;
  transform: scale(1);
}

.main-menu-seal:active {
  transform: rotate(-9deg) translateY(0) scale(0.98);
}

.main-menu-seal:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--ink-accent-seal) 36%, transparent),
    0 8px 18px color-mix(in srgb, var(--ink-accent-seal) 34%, transparent);
}

.main-menu-seal-whisper {
  margin: 2px 0 0;
  text-align: center;
  color: var(--ink-text-muted);
  font-size: 12px;
  letter-spacing: 0.04em;
}

.menu-card {
  background: var(--ink-card-bg);
  border: 1px solid var(--ink-border-soft);
  border-radius: 12px;
  box-shadow: var(--ink-shadow-card);
  padding: 20px;
}

.menu-card-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 1.45rem;
  font-weight: 700;
}

.menu-btn {
  width: 100%;
  border-radius: 10px;
  border: 1px solid var(--ink-title-color);
  background: var(--ink-card-bg);
  color: var(--ink-text-primary);
  padding: 13px 16px;
  font-size: 1rem;
  transition: box-shadow 180ms ease, transform 120ms ease, background-color 180ms ease;
}

.menu-btn:hover {
  background: var(--ink-paper);
  box-shadow: var(--ink-shadow-card);
}

.menu-btn:active {
  transform: translateY(1px);
}

.menu-btn-primary {
  background: color-mix(in srgb, var(--ink-title-color) 28%, var(--ink-paper));
}

.menu-volume-pill {
  margin: 0;
  text-align: center;
  border: 1px solid var(--ink-border-accent);
  border-radius: 999px;
  padding: 4px 8px;
  background: color-mix(in srgb, var(--ink-card-bg) 88%, transparent);
  color: var(--ink-text-muted);
}

.menu-chip,
.menu-chip-btn,
.menu-inline-btn {
  border: 1px solid var(--ink-border-accent);
  border-radius: 8px;
  background: var(--ink-card-bg);
  color: var(--ink-text-primary);
}

.menu-chip {
  padding: 2px 8px;
  font-size: 12px;
}

.menu-chip-btn {
  padding: 6px 10px;
  font-size: 12px;
}

.menu-inline-btn {
  padding: 6px 12px;
  font-size: 14px;
}

.menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: color-mix(in srgb, var(--ink-text-primary) 30%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.menu-dialog {
  width: min(100%, 1080px);
  max-height: min(90vh, 840px);
  overflow: auto;
  border-radius: 16px;
  border: 1px solid var(--ink-border-soft);
  background: color-mix(in srgb, var(--ink-paper) 88%, var(--ink-card-bg));
  box-shadow: var(--ink-shadow-panel);
  padding: 20px;
}

.menu-dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

.menu-dialog-body {
  margin-top: 14px;
  display: grid;
  gap: 16px;
  grid-template-columns: 280px 1fr;
}

.menu-dialog-left,
.menu-dialog-right {
  border: 1px solid var(--ink-border-soft);
  border-radius: 12px;
  background: var(--ink-card-bg-muted);
  padding: 12px;
}

.menu-sub-card {
  border: 1px solid var(--ink-border-soft);
  border-radius: 10px;
  background: var(--ink-card-bg);
  padding: 12px;
}

.menu-sub-title {
  margin: 0;
  color: var(--ink-title-color);
  letter-spacing: 0.06em;
  font-size: 12px;
}

.menu-text-primary {
  color: var(--ink-text-primary);
}

.menu-text-muted {
  color: var(--ink-text-muted);
}

.menu-text-error {
  color: color-mix(in srgb, var(--ink-accent-main) 86%, var(--ink-text-primary));
}

.menu-chip-btn-active {
  border-color: var(--ink-title-color);
}

@media (max-width: 980px) {
  .main-menu-frame {
    grid-template-columns: 1fr;
  }

  .menu-left-rail {
    position: static;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .menu-dialog-body {
    grid-template-columns: 1fr;
  }

  .main-menu-seal {
    position: static;
    margin: 12px auto 0;
  }
}

@keyframes main-seal-float {
  0%,
  100% {
    transform: rotate(-9deg) translateY(0);
  }
  50% {
    transform: rotate(-9deg) translateY(-2px);
  }
}
</style>
