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
            data-testid="new-game-btn"
            @click="handleNewGame"
            class="w-full sm:w-64 rounded-lg bg-amber-600 px-6 py-3 font-medium text-slate-900 transition-colors duration-200 hover:bg-amber-500"
          >
            新游戏
          </button>
          <button
            data-testid="continue-game-btn"
            @click="openLoadDialog"
            class="w-full sm:w-64 rounded-lg bg-slate-700 px-6 py-3 font-medium text-slate-100 transition-colors duration-200 hover:bg-slate-600"
          >
            读取存档
          </button>
          <button
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

        <div class="rounded-xl border border-slate-700 bg-slate-900/60 p-4">
          <h2 class="text-sm font-semibold text-slate-200">最近存档</h2>
          <p
            v-if="latestSave"
            class="mt-2 text-xs leading-5 text-slate-300"
          >
            槽位 {{ latestSave.slot_id }} · {{ latestSave.player_name }} · {{ latestSave.realm }}
            <span class="mt-1 flex flex-wrap items-center gap-1.5">
              <span class="rounded border border-slate-600/80 bg-slate-800/70 px-1.5 py-0.5 text-[11px] text-slate-200">
                位置：{{ latestSaveLocationLabel }}
              </span>
              <span class="rounded border border-slate-600/80 bg-slate-800/70 px-1.5 py-0.5 text-[11px] text-slate-200">
                时间：{{ latestSave.game_time }}
              </span>
            </span>
            <span class="mt-1 block text-[11px] text-slate-400">
              最近保存：{{ latestSaveTimestampLabel }}（{{ latestSaveAgeLabel }}）
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
            class="mt-2 text-xs leading-5 text-slate-400"
          >
            当前没有可用存档。你可以开始新游戏后再保存进度。
          </p>

          <p
            v-if="recentSaveError"
            class="mt-2 text-xs text-red-300"
          >
            {{ recentSaveError }}
          </p>
          <button
            v-if="recentSaveError && !recentSaveLoading"
            data-testid="retry-load-saves-btn"
            class="mt-2 rounded-md border border-red-400/40 px-2.5 py-1 text-[11px] text-red-200 transition-colors hover:bg-red-500/10"
            @click="fetchLatestSave"
          >
            重试读取
          </button>
          <p
            v-else-if="lastRefreshLabel"
            data-testid="recent-save-refresh-label"
            class="mt-2 text-[11px] text-slate-500"
          >
            最近刷新：{{ lastRefreshLabel }}（{{ relativeRefreshLabel }}）
          </p>
          <p
            v-if="lastRefreshStatusLabel"
            data-testid="recent-save-refresh-status"
            class="mt-1 text-[11px]"
            :class="lastRefreshSucceeded ? 'text-emerald-300' : 'text-amber-300'"
          >
            刷新状态：{{ lastRefreshStatusLabel }}
          </p>

          <div class="mt-4 rounded-lg border border-slate-700/70 bg-slate-950/30 p-3">
            <p class="mb-2 text-[11px] uppercase tracking-[0.18em] text-slate-400">存档操作</p>
            <div class="grid gap-2 sm:grid-cols-3">
            <button
              data-testid="recent-save-btn"
              class="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-3 py-2 text-xs font-medium text-emerald-100 transition-colors hover:bg-emerald-500/20 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="!latestSave || quickLoadPending || recentSaveLoading"
              @click="loadLatestSave"
            >
              {{ quickLoadPending ? '加载中...' : '继续最近存档' }}
            </button>
            <button
              data-testid="refresh-save-btn"
              class="rounded-md border border-slate-600 bg-slate-900/70 px-3 py-2 text-xs font-medium text-slate-200 transition-colors hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="recentSaveLoading || quickLoadPending"
              @click="fetchLatestSave"
            >
              {{ recentSaveLoading ? '刷新中...' : '刷新存档' }}
            </button>
            <button
              data-testid="open-audio-btn"
              class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs font-medium text-amber-100 transition-colors hover:bg-amber-500/20"
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
                data-testid="quick-volume-status"
                class="text-[11px]"
                :class="quickMasterVolume <= 0 ? 'text-amber-200' : 'text-slate-400'"
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
              data-testid="quick-mute-btn"
              class="rounded-md border border-amber-500/40 px-2.5 py-1 text-[11px] text-amber-100 transition-colors hover:bg-amber-500/10"
              :aria-pressed="quickMasterVolume <= 0 ? 'true' : 'false'"
              @click="toggleQuickMute"
            >
              {{ quickMasterVolume <= 0 ? '恢复' : '静音' }}
            </button>
            <button
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
const quickMasterVolume = ref(getAudioSettings().master);
const quickBgmEnabled = ref(getAudioSettings().bgmEnabled);
const quickSfxEnabled = ref(getAudioSettings().sfxEnabled);
const previousMasterVolume = ref(Math.max(0.01, quickMasterVolume.value || 0.55));

const latestSaveLocationLabel = computed(() => formatLocationLabel(latestSave.value?.location));
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
  if (lastRefreshSucceeded.value == null) {
    return '';
  }
  return lastRefreshSucceeded.value ? '成功' : '失败';
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
