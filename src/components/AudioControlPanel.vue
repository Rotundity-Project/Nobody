<template>
  <div class="audio-panel space-y-4">
    <div v-if="showSwitches" class="flex items-center justify-between">
      <div>
        <p class="text-sm text-[var(--ink-text-primary)]">背景音乐</p>
        <p class="text-xs text-[var(--ink-text-muted)]">控制环境氛围与场景铺垫</p>
      </div>
      <button
        data-testid="toggle-bgm-btn"
        class="audio-toggle-btn rounded-full px-3 py-1 text-xs font-semibold transition-colors"
        :class="settings.bgmEnabled ? 'audio-toggle-on' : 'audio-toggle-off'"
        @click="toggleBgm"
      >
        {{ settings.bgmEnabled ? '开启' : '关闭' }}
      </button>
    </div>

    <div v-if="showSwitches" class="flex items-center justify-between">
      <div>
        <p class="text-sm text-[var(--ink-text-primary)]">界面音效</p>
        <p class="text-xs text-[var(--ink-text-muted)]">按钮点击与交互提示音</p>
      </div>
      <button
        data-testid="toggle-sfx-btn"
        class="audio-toggle-btn rounded-full px-3 py-1 text-xs font-semibold transition-colors"
        :class="settings.sfxEnabled ? 'audio-toggle-on-cool' : 'audio-toggle-off'"
        @click="toggleSfx"
      >
        {{ settings.sfxEnabled ? '开启' : '关闭' }}
      </button>
    </div>

    <div>
      <div class="flex items-center justify-between">
        <p class="text-sm text-[var(--ink-text-primary)]">总音量</p>
        <span class="text-xs text-[var(--ink-text-muted)]">{{ Math.round(settings.master * 100) }}%</span>
      </div>
      <input
        v-model.number="settings.master"
        data-testid="master-volume-range"
        type="range"
        min="0"
        max="1"
        step="0.01"
        class="audio-range mt-2 w-full"
        @input="updateMaster"
      />

      <div class="mt-2 flex flex-wrap gap-2">
        <button
          data-testid="volume-preset-low"
          class="audio-chip-btn"
          @click="setMasterPreset(0.25)"
        >低</button>
        <button
          data-testid="volume-preset-mid"
          class="audio-chip-btn"
          @click="setMasterPreset(0.55)"
        >中</button>
        <button
          data-testid="volume-preset-high"
          class="audio-chip-btn"
          @click="setMasterPreset(0.8)"
        >高</button>
        <button
          data-testid="volume-toggle-mute"
          class="audio-chip-btn audio-chip-btn-accent"
          @click="toggleMute"
        >{{ isMuted ? '恢复音量' : '静音' }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, watchEffect } from 'vue';
import {
  applyAudioSettings,
  getAudioSettings,
  playClick,
  setBgmEnabled,
  setMasterVolume,
  setSfxEnabled,
} from '../utils/audioSystem';

withDefaults(defineProps<{
  showSwitches?: boolean;
}>(), {
  showSwitches: true,
});

const settings = reactive(getAudioSettings());
let previousMasterVolume = Math.max(0.01, settings.master || 0.55);

const isMuted = computed(() => settings.master <= 0);

watchEffect(() => {
  applyAudioSettings(settings);
});

const updateMaster = () => {
  settings.master = Math.min(1, Math.max(0, settings.master));
  if (settings.master > 0) {
    previousMasterVolume = settings.master;
  }
  setMasterVolume(settings.master);
};

const setMasterPreset = (value: number) => {
  settings.master = value;
  previousMasterVolume = value;
  setMasterVolume(value);
  playClick();
};

const toggleMute = () => {
  if (settings.master <= 0) {
    settings.master = previousMasterVolume;
  } else {
    previousMasterVolume = settings.master;
    settings.master = 0;
  }
  setMasterVolume(settings.master);
  playClick();
};

const toggleBgm = () => {
  settings.bgmEnabled = !settings.bgmEnabled;
  setBgmEnabled(settings.bgmEnabled);
  playClick();
};

const toggleSfx = () => {
  settings.sfxEnabled = !settings.sfxEnabled;
  setSfxEnabled(settings.sfxEnabled);
  playClick();
};
</script>

<style scoped>
.audio-panel {
  color: var(--ink-text-primary);
}

.audio-toggle-btn {
  border: 1px solid var(--ink-border-accent);
}

.audio-toggle-on {
  background: var(--audio-toggle-on-bg);
  color: var(--audio-toggle-on-text);
}

.audio-toggle-on-cool {
  background: var(--audio-toggle-on-cool-bg);
  color: var(--ink-text-cool);
}

.audio-toggle-off {
  background: var(--ink-paper-elevated);
  color: var(--ink-text-muted);
}

.audio-range {
  accent-color: var(--ink-title-color);
}

.audio-chip-btn {
  border: 1px solid var(--ink-border-accent);
  border-radius: 8px;
  background: var(--ink-paper-elevated);
  color: var(--ink-text-primary);
  padding: 4px 10px;
  font-size: 11px;
  transition: background-color 180ms ease, border-color 180ms ease;
}

.audio-chip-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}

.audio-chip-btn-accent {
  border-color: var(--ink-title-color);
  color: var(--audio-chip-accent-text);
  background: var(--audio-chip-accent-bg);
}
</style>
