<template>
  <div class="min-h-screen flex items-center justify-center px-4 py-8">
    <div class="panel-surface w-full max-w-3xl rounded-2xl p-6 sm:p-10">
      <div class="space-y-3 text-center">
        <p class="text-xs uppercase tracking-[0.3em] text-amber-200/70">Immortal Chronicle</p>
        <h1 class="text-4xl sm:text-6xl font-display text-glow text-amber-200">
          Nobody
        </h1>
        <p class="text-lg sm:text-xl text-slate-300 font-story">修仙模拟器</p>
      </div>

      <div class="mt-8 grid gap-4 lg:grid-cols-[1.1fr_0.9fr]">
        <div class="space-y-3">
          <button
            type="button"
            data-testid="new-game-btn"
            @click="handleNewGame"
            class="w-full sm:w-64 rounded-lg bg-amber-600 px-6 py-3 font-medium text-slate-900 transition-colors duration-200 hover:bg-amber-500"
          >
            新游戏
          </button>
          <button
            type="button"
            data-testid="continue-game-btn"
            @click="openLoadDialog"
            class="w-full sm:w-64 rounded-lg bg-slate-700 px-6 py-3 font-medium text-slate-100 transition-colors duration-200 hover:bg-slate-600"
          >
            读取存档
          </button>
          <button
            type="button"
            data-testid="llm-settings-btn"
            @click="handleSettings"
            class="w-full sm:w-64 rounded-lg bg-emerald-600 px-6 py-3 font-medium text-slate-900 transition-colors duration-200 hover:bg-emerald-500"
          >
            LLM 设置
          </button>
          <p class="pt-1 text-xs text-slate-400">
            建议流程：新游戏 -> 选择剧本 -> 进入主界面推进剧情
          </p>
        </div>

        <div
          id="recent-save-card"
          data-testid="recent-save-card"
          class="rounded-xl border border-slate-700 bg-slate-900/60 p-4"
          :aria-busy="recentSaveLoading ? 'true' : 'false'"
        >
          <h2 class="text-sm font-semibold text-slate-200">最近存档</h2>
          <p
            v-if="latestSave"
            id="latest-save-summary"
            data-testid="latest-save-summary"
            class="mt-2 text-xs leading-5 text-slate-300"
            aria-live="polite"
            aria-atomic="true"
          >
            槽位 {{ latestSave.slot_id }} · {{ latestSavePlayerLabel }} · {{ latestSaveRealmLabel }}
              <span class="mt-1 flex flex-wrap items-center gap-1.5">
              <span
                v-if="showLatestSaveLocationTag"
                class="rounded border border-slate-600/80 bg-slate-800/70 px-1.5 py-0.5 text-[11px] text-slate-200"
              >
                位置：{{ latestSaveLocationLabel }}
              </span>
              <span class="rounded border border-slate-600/80 bg-slate-800/70 px-1.5 py-0.5 text-[11px] text-slate-200">
                时间：{{ latestSaveGameTimeLabel }}
              </span>
            </span>
            <span class="mt-1 block text-[11px] text-slate-400">
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
            class="mt-2 text-xs leading-5 text-slate-400"
          >
            正在读取最近存档...
          </p>
          <p
            v-else
            id="no-save-hint"
            data-testid="no-save-hint"
            class="mt-2 text-xs leading-5 text-slate-400"
          >
            暂无可用存档，可先开始新游戏创建进度。
          </p>

          <p
            v-if="recentSaveError"
            id="recent-save-error"
            class="mt-2 text-xs text-red-300"
          >
            {{ recentSaveError }}
          </p>
          <button
            type="button"
            v-if="recentSaveError && !recentSaveLoading"
            data-testid="retry-load-saves-btn"
            class="mt-2 rounded-md border border-red-400/40 px-2.5 py-1 text-[11px] text-red-200 transition-colors hover:bg-red-500/10"
            :aria-label="retryLoadSavesAriaLabel"
            aria-describedby="recent-save-error recent-save-refresh-label"
            @click="fetchLatestSave"
          >
            重试读取
          </button>
          <p
            v-else-if="lastRefreshLabel"
            id="recent-save-refresh-label"
            data-testid="recent-save-refresh-label"
            class="mt-2 text-[11px] text-slate-500"
          >
            最近刷新：{{ lastRefreshLabel }}（{{ relativeRefreshLabel }}）
          </p>
          <p
            v-if="shouldShowRefreshStatus && lastRefreshStatusLabel"
            id="recent-save-refresh-status"
            data-testid="recent-save-refresh-status"
            class="mt-1 text-[11px]"
            :class="refreshStatusToneClass"
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            刷新状态：{{ lastRefreshStatusLabel }}
          </p>

          <div class="mt-4 rounded-lg border border-slate-700/70 bg-slate-950/30 p-3">
            <p class="mb-2 text-[11px] uppercase tracking-[0.18em] text-slate-400">存档操作</p>
            <div class="grid gap-2 sm:grid-cols-3">
            <button
              type="button"
              data-testid="recent-save-btn"
              class="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-3 py-2 text-xs font-medium text-emerald-100 transition-colors hover:bg-emerald-500/20 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="!latestSave || quickLoadPending || recentSaveLoading"
              :aria-label="recentSaveActionAriaLabel"
              :aria-describedby="recentSaveActionDescribedBy"
              @click="loadLatestSave"
            >
              {{ quickLoadPending ? '加载中...' : '继续最近存档' }}
            </button>
            <button
              type="button"
              data-testid="refresh-save-btn"
              class="rounded-md border border-slate-600 bg-slate-900/70 px-3 py-2 text-xs font-medium text-slate-200 transition-colors hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
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
              data-testid="open-audio-btn"
              class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs font-medium text-amber-100 transition-colors hover:bg-amber-500/20"
              :aria-label="audioPanelToggleAriaLabel"
              :aria-describedby="audioPanelToggleDescribedBy"
              aria-controls="main-menu-audio-panel"
              :aria-expanded="showAudioPanel ? 'true' : 'false'"
              @click="toggleAudioPanel"
            >
              {{ showAudioPanel ? '收起音量控制' : '音量控制' }}
            </button>
          </div>
          </div>

          <div class="mt-3 rounded-lg border border-slate-700/70 bg-slate-950/30 p-3">
            <div class="mb-2 flex items-center justify-between">
              <span class="text-[11px] uppercase tracking-[0.18em] text-slate-400">音频快捷</span>
              <span
                id="quick-volume-status"
                data-testid="quick-volume-status"
                class="text-[11px]"
                :class="quickMasterVolume <= 0 ? 'text-amber-200' : 'text-slate-400'"
                role="status"
                aria-live="polite"
                aria-atomic="true"
              >
                {{ quickMasterVolume <= 0 ? '已静音' : `当前 ${Math.round(quickMasterVolume * 100)}%` }}
              </span>
            </div>
            <div
              class="flex flex-wrap items-center gap-2"
              role="group"
              aria-label="快捷音量预设"
            >
            <button
              type="button"
              data-testid="quick-mute-btn"
              class="rounded-md border border-amber-500/40 px-2.5 py-1 text-[11px] text-amber-100 transition-colors hover:bg-amber-500/10"
              :aria-pressed="quickMasterVolume <= 0 ? 'true' : 'false'"
              @click="toggleQuickMute"
            >
              {{ quickMasterVolume <= 0 ? '恢复' : '静音' }}
            </button>
            <button
              type="button"
              data-testid="quick-volume-30-btn"
              class="rounded-md border px-2.5 py-1 text-[11px] transition-colors"
              :class="isActiveQuickVolume(0.3)
                ? 'border-emerald-400 bg-emerald-500/15 text-emerald-200'
                : 'border-slate-600 text-slate-200 hover:bg-slate-800'"
              :aria-pressed="isActiveQuickVolume(0.3) ? 'true' : 'false'"
              @click="applyQuickVolume(0.3)"
            >
              30%
            </button>
            <button
              type="button"
              data-testid="quick-volume-60-btn"
              class="rounded-md border px-2.5 py-1 text-[11px] transition-colors"
              :class="isActiveQuickVolume(0.6)
                ? 'border-emerald-400 bg-emerald-500/15 text-emerald-200'
                : 'border-slate-600 text-slate-200 hover:bg-slate-800'"
              :aria-pressed="isActiveQuickVolume(0.6) ? 'true' : 'false'"
              @click="applyQuickVolume(0.6)"
            >
              60%
            </button>
            <button
              type="button"
              data-testid="quick-volume-100-btn"
              class="rounded-md border px-2.5 py-1 text-[11px] transition-colors"
              :class="isActiveQuickVolume(1)
                ? 'border-emerald-400 bg-emerald-500/15 text-emerald-200'
                : 'border-slate-600 text-slate-200 hover:bg-slate-800'"
              :aria-pressed="isActiveQuickVolume(1) ? 'true' : 'false'"
              @click="applyQuickVolume(1)"
            >
              100%
            </button>
            <button
              type="button"
              data-testid="quick-bgm-btn"
              class="rounded-md border px-2.5 py-1 text-[11px] transition-colors"
              :class="quickBgmEnabled
                ? 'border-emerald-400 bg-emerald-500/15 text-emerald-200'
                : 'border-slate-600 text-slate-200 hover:bg-slate-800'"
              :aria-pressed="quickBgmEnabled ? 'true' : 'false'"
              @click="toggleQuickBgm"
            >
              {{ quickBgmEnabled ? 'BGM 开' : 'BGM 关' }}
            </button>
            <button
              type="button"
              data-testid="quick-sfx-btn"
              class="rounded-md border px-2.5 py-1 text-[11px] transition-colors"
              :class="quickSfxEnabled
                ? 'border-emerald-400 bg-emerald-500/15 text-emerald-200'
                : 'border-slate-600 text-slate-200 hover:bg-slate-800'"
              :aria-pressed="quickSfxEnabled ? 'true' : 'false'"
              @click="toggleQuickSfx"
            >
              {{ quickSfxEnabled ? '音效 开' : '音效 关' }}
            </button>
          </div>
          </div>
        </div>
      </div>

      <div
        v-if="showAudioPanel"
        id="main-menu-audio-panel"
        class="mt-6 rounded-xl border border-slate-700 bg-slate-900/60 p-4"
      >
        <AudioControlPanel />
      </div>
    </div>

    <SaveLoadDialog
      :is-open="showLoadDialog"
      mode="load"
      @close="showLoadDialog = false"
      @loaded="handleLoadedFromDialog"
    />
    <LLMConfigDialog :is-open="showLLMDialog" @close="showLLMDialog = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import AudioControlPanel from './AudioControlPanel.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import { getAudioSettings, playClick, setBgmEnabled, setMasterVolume, setSfxEnabled } from '../utils/audioSystem';
import { useGameStore } from '../stores/gameStore';
import type { SaveInfo } from '../types/game';
import { formatLocationLabel } from '../shared/locationLabel';

const router = useRouter();
const gameStore = useGameStore();

const showLLMDialog = ref(false);
const showAudioPanel = ref(false);
const showLoadDialog = ref(false);
const recentSaveLoading = ref(false);
const quickLoadPending = ref(false);
const recentSaveError = ref('');
const latestSave = ref<SaveInfo | null>(null);
const lastRefreshAt = ref<number | null>(null);
const lastRefreshSucceeded = ref<boolean | null>(null);
const refreshNowTick = ref(Date.now());
let refreshTickerId: number | null = null;
const REFRESH_SUCCESS_TOAST_MS = 8000;
const quickMasterVolume = ref(getAudioSettings().master);
const quickBgmEnabled = ref(getAudioSettings().bgmEnabled);
const quickSfxEnabled = ref(getAudioSettings().sfxEnabled);
const previousMasterVolume = ref(Math.max(0.01, quickMasterVolume.value || 0.55));

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
const refreshActionDescribedBy = computed(() => {
  const ids = ['recent-save-refresh-label'];
  if (shouldShowRefreshStatus.value && lastRefreshStatusLabel.value) {
    ids.push('recent-save-refresh-status');
  }
  return ids.join(' ');
});
const refreshActionAriaLabel = computed(() => {
  if (recentSaveLoading.value) {
    return '刷新存档，当前正在刷新中';
  }
  if (lastRefreshSucceeded.value === true) {
    const suffix = relativeRefreshLabel.value || '刚刚';
    return `刷新存档，最近一次刷新成功，${suffix}`;
  }
  if (lastRefreshSucceeded.value === false) {
    const suffix = relativeRefreshLabel.value || '时间未知';
    return `刷新存档，最近一次刷新失败，${suffix}`;
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
  return `继续最近存档，槽位 ${latestSave.value.slot_id}`;
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
const audioPanelToggleDescribedBy = computed(() => 'quick-volume-status');
const audioPanelToggleAriaLabel = computed(() => {
  const volumeLabel = quickMasterVolume.value <= 0
    ? '当前已静音'
    : `当前音量 ${Math.round(quickMasterVolume.value * 100)}%`;
  const bgmLabel = quickBgmEnabled.value ? 'BGM 开' : 'BGM 关';
  const sfxLabel = quickSfxEnabled.value ? '音效 开' : '音效 关';
  const actionLabel = showAudioPanel.value ? '收起音量控制' : '展开音量控制';
  return `${actionLabel}，${volumeLabel}，${bgmLabel}，${sfxLabel}`;
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
      return;
    }
    latestSave.value = [...slots].sort((a, b) => b.timestamp - a.timestamp)[0] ?? null;
  } catch (error) {
    latestSave.value = null;
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
  quickBgmEnabled.value = settings.bgmEnabled;
  quickSfxEnabled.value = settings.sfxEnabled;
  if (clamped > 0) {
    previousMasterVolume.value = clamped;
  }
};

const handleNewGame = () => {
  playClick();
  router.push('/script-select');
};

const openLoadDialog = () => {
  playClick();
  showLoadDialog.value = true;
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

const applyQuickVolume = (value: number) => {
  const clamped = Math.min(1, Math.max(0, value));
  quickMasterVolume.value = clamped;
  if (clamped > 0) {
    previousMasterVolume.value = clamped;
  }
  setMasterVolume(clamped);
  playClick();
};

const isActiveQuickVolume = (value: number) =>
  quickMasterVolume.value > 0 && Math.abs(quickMasterVolume.value - value) < 0.001;

const toggleQuickMute = () => {
  if (quickMasterVolume.value <= 0) {
    applyQuickVolume(previousMasterVolume.value);
    return;
  }
  previousMasterVolume.value = quickMasterVolume.value;
  applyQuickVolume(0);
};

const toggleQuickBgm = () => {
  const next = !quickBgmEnabled.value;
  quickBgmEnabled.value = next;
  setBgmEnabled(next);
  playClick();
};

const toggleQuickSfx = () => {
  const next = !quickSfxEnabled.value;
  quickSfxEnabled.value = next;
  setSfxEnabled(next);
  playClick();
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

const handleLoadedFromDialog = () => {
  showLoadDialog.value = false;
  router.push('/game');
  void fetchLatestSave();
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
  window.removeEventListener('focus', syncQuickVolumeFromSettings);
});
</script>
