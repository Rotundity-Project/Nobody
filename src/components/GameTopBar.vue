<template>
  <div class="flex items-center gap-4">
    <button
      class="text-gray-400 transition-colors hover:text-white"
      title="返回"
      @click="$emit('back')"
    >
      <svg
        class="h-6 w-6"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10 19l-7-7m0 0l7-7m-7 7h18"
        />
      </svg>
    </button>
    <h1 class="font-display text-xl text-amber-200">
      Nobody
    </h1>
  </div>
  <div class="flex flex-wrap gap-2">
    <SystemCenterMenu
      :is-open="isSystemMenuOpen"
      :show-audio-panel="showAudioPanel"
      @toggle-menu="$emit('toggle-menu')"
      @close-menu="$emit('close-menu')"
      @toggle-audio="$emit('toggle-audio')"
      @open-shortcuts="$emit('open-shortcuts')"
      @open-llm="$emit('open-llm')"
      @open-story-settings="$emit('open-story-settings')"
      @open-consistency="$emit('open-consistency')"
    />
    <QuickActionsBar
      :is-game-initialized="isGameInitialized"
      @open-character="$emit('open-character')"
      @open-info="$emit('open-info')"
      @open-save="$emit('open-save')"
      @open-load="$emit('open-load')"
    />
  </div>
</template>

<script setup lang="ts">
import QuickActionsBar from './QuickActionsBar.vue';
import SystemCenterMenu from './SystemCenterMenu.vue';

defineProps<{
  isSystemMenuOpen: boolean;
  showAudioPanel: boolean;
  isGameInitialized: boolean;
}>();

defineEmits<{
  (event: 'back'): void;
  (event: 'toggle-menu'): void;
  (event: 'close-menu'): void;
  (event: 'toggle-audio'): void;
  (event: 'open-shortcuts'): void;
  (event: 'open-llm'): void;
  (event: 'open-story-settings'): void;
  (event: 'open-consistency'): void;
  (event: 'open-character'): void;
  (event: 'open-info'): void;
  (event: 'open-save'): void;
  (event: 'open-load'): void;
}>();
</script>
