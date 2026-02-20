<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm text-slate-300">背景音乐</p>
        <p class="text-xs text-slate-500">控制环境氛围与场景铺垫</p>
      </div>
      <button
        data-testid="toggle-bgm-btn"
        class="rounded-full px-3 py-1 text-xs font-semibold transition-colors"
        :class="settings.bgmEnabled ? 'bg-amber-400 text-slate-900' : 'bg-slate-700 text-slate-300'"
        @click="toggleBgm"
      >
        {{ settings.bgmEnabled ? '开启' : '关闭' }}
      </button>
    </div>

    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm text-slate-300">界面音效</p>
        <p class="text-xs text-slate-500">按钮点击与交互提示音</p>
      </div>
      <button
        data-testid="toggle-sfx-btn"
        class="rounded-full px-3 py-1 text-xs font-semibold transition-colors"
        :class="settings.sfxEnabled ? 'bg-emerald-400 text-slate-900' : 'bg-slate-700 text-slate-300'"
        @click="toggleSfx"
      >
        {{ settings.sfxEnabled ? '开启' : '关闭' }}
      </button>
    </div>

    <div>
      <div class="flex items-center justify-between">
        <p class="text-sm text-slate-300">总音量</p>
        <span class="text-xs text-slate-400">{{ Math.round(settings.master * 100) }}%</span>
      </div>
      <input
        v-model.number="settings.master"
        data-testid="master-volume-range"
        type="range"
        min="0"
        max="1"
        step="0.01"
        class="mt-2 w-full accent-amber-400"
        @input="updateMaster"
      />

      <div class="mt-2 flex flex-wrap gap-2">
        <button
          data-testid="volume-preset-low"
          class="rounded-md border border-slate-600 px-2.5 py-1 text-[11px] text-slate-200 transition-colors hover:bg-slate-800"
          @click="setMasterPreset(0.25)"
        >低</button>
        <button
          data-testid="volume-preset-mid"
          class="rounded-md border border-slate-600 px-2.5 py-1 text-[11px] text-slate-200 transition-colors hover:bg-slate-800"
          @click="setMasterPreset(0.55)"
        >中</button>
        <button
          data-testid="volume-preset-high"
          class="rounded-md border border-slate-600 px-2.5 py-1 text-[11px] text-slate-200 transition-colors hover:bg-slate-800"
          @click="setMasterPreset(0.8)"
        >高</button>
        <button
          data-testid="volume-toggle-mute"
          class="rounded-md border border-amber-500/40 px-2.5 py-1 text-[11px] text-amber-100 transition-colors hover:bg-amber-500/10"
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
