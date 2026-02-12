<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm text-slate-300">背景音乐</p>
        <p class="text-xs text-slate-500">柔和的修炼气息</p>
      </div>
      <button
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
        <p class="text-xs text-slate-500">按钮与交互提示</p>
      </div>
      <button
        class="rounded-full px-3 py-1 text-xs font-semibold transition-colors"
        :class="settings.sfxEnabled ? 'bg-emerald-400 text-slate-900' : 'bg-slate-700 text-slate-300'"
        @click="toggleSfx"
      >
        {{ settings.sfxEnabled ? '开启' : '关闭' }}
      </button>
    </div>

    <div>
      <div class="flex items-center justify-between">
        <p class="text-sm text-slate-300">音量</p>
        <span class="text-xs text-slate-400">{{ Math.round(settings.master * 100) }}%</span>
      </div>
      <input
        v-model.number="settings.master"
        type="range"
        min="0"
        max="1"
        step="0.01"
        class="mt-2 w-full accent-amber-400"
        @input="updateMaster"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watchEffect } from 'vue';
import {
  applyAudioSettings,
  getAudioSettings,
  playClick,
  setBgmEnabled,
  setMasterVolume,
  setSfxEnabled,
} from '../utils/audioSystem';

const settings = reactive(getAudioSettings());

watchEffect(() => {
  applyAudioSettings(settings);
});

const updateMaster = () => {
  setMasterVolume(settings.master);
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
