<template>
  <div class="audio-panel space-y-4">
    <div v-if="showSwitches" class="flex items-center justify-between">
      <div>
        <p class="text-sm text-[#2d2a24]">背景音乐</p>
        <p class="text-xs text-[#6a655d]">控制环境氛围与场景铺垫</p>
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
        <p class="text-sm text-[#2d2a24]">界面音效</p>
        <p class="text-xs text-[#6a655d]">按钮点击与交互提示音</p>
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
        <p class="text-sm text-[#2d2a24]">总音量</p>
        <span class="text-xs text-[#6a655d]">{{ Math.round(settings.master * 100) }}%</span>
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
  color: #2d2a24;
}

.audio-toggle-btn {
  border: 1px solid #b7a88c;
}

.audio-toggle-on {
  background: #efe4cf;
  color: #7a612f;
}

.audio-toggle-on-cool {
  background: #edf5f2;
  color: #2f6a5d;
}

.audio-toggle-off {
  background: #f8f3ea;
  color: #6a655d;
}

.audio-range {
  accent-color: #b78c4a;
}

.audio-chip-btn {
  border: 1px solid #b7a88c;
  border-radius: 8px;
  background: #f8f3ea;
  color: #2d2a24;
  padding: 4px 10px;
  font-size: 11px;
  transition: background-color 180ms ease, border-color 180ms ease;
}

.audio-chip-btn:hover {
  border-color: #b78c4a;
  background: #faf7f2;
}

.audio-chip-btn-accent {
  border-color: #b78c4a;
  color: #7a612f;
  background: #efe4cf;
}
</style>
