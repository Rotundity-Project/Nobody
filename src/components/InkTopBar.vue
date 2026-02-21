<template>
  <header class="ink-topbar rounded-2xl px-4 py-3 sm:px-5 sm:py-4">
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0 flex items-center gap-3">
        <button
          type="button"
          class="ink-icon-btn"
          title="返回主菜单"
          aria-label="返回主菜单"
          @click="$emit('back')"
        >
          <svg
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.8"
              d="M10 19l-7-7m0 0l7-7m-7 7h18"
            />
          </svg>
        </button>
        <div class="min-w-0">
          <p class="font-display text-sm tracking-[0.18em] text-[var(--ink-title-color)]">
            无名修途
          </p>
          <p class="truncate text-xs text-[var(--ink-text-muted)]">
            {{ chapterTitle }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="ink-main-action hidden sm:inline-flex"
          :disabled="!isGameInitialized"
          @click="$emit('open-info')"
        >
          观山海
        </button>
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
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import SystemCenterMenu from './SystemCenterMenu.vue';

defineProps<{
  isSystemMenuOpen: boolean;
  showAudioPanel: boolean;
  isGameInitialized: boolean;
  chapterTitle: string;
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
  (event: 'open-info'): void;
}>();
</script>
