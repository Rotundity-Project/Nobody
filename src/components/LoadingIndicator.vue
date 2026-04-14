<template>
  <div class="w-full space-y-3 text-center">
    <div class="flex items-center justify-center gap-3">
      <div
        class="inline-block animate-spin rounded-full border-2"
        :style="spinnerStyle"
        :class="{
          'h-4 w-4': size === 'sm',
          'h-6 w-6': size === 'md',
          'h-8 w-8': size === 'lg',
        }"
      />
      <div class="text-left">
        <p :style="messageStyle">
          {{ message }}
        </p>
        <p
          v-if="detail"
          class="text-xs"
          :style="detailStyle"
        >
          {{ detail }}
        </p>
      </div>
    </div>
    <div
      class="h-1.5 overflow-hidden rounded-full"
      :style="progressTrackStyle"
      aria-hidden="true"
    >
      <div
        v-if="progressPercent !== null"
        class="h-full rounded-full transition-all duration-300"
        :style="{ ...progressFillStyle, width: `${progressPercent}%` }"
      />
      <div
        v-else
        class="loading-bar h-full w-1/3 rounded-full"
        :style="progressFillStyle"
      />
    </div>
    <p
      v-if="progressText"
      class="text-xs"
      :style="detailStyle"
    >
      {{ progressText }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    message?: string;
    detail?: string;
    size?: 'sm' | 'md' | 'lg';
    progress?: number | null;
    progressText?: string;
  }>(),
  {
    message: '处理中...',
    detail: '',
    size: 'md',
    progress: null,
    progressText: '',
  },
);

const progressPercent = computed(() => {
  if (props.progress == null) {
    return null;
  }
  return Math.max(0, Math.min(100, props.progress));
});
const spinnerStyle = computed(() => ({
  borderColor: 'var(--loading-spinner-ring)',
  borderTopColor: 'var(--loading-accent-color, var(--ink-accent-note))',
}));
const messageStyle = computed(() => ({
  color: 'var(--ink-text-primary)',
}));
const detailStyle = computed(() => ({
  color: 'var(--ink-text-muted)',
}));
const progressTrackStyle = computed(() => ({
  background: 'var(--loading-track-bg)',
}));
const progressFillStyle = computed(() => ({
  background: 'var(--loading-fill-bg)',
}));
</script>

<style scoped>
.loading-bar {
  animation: loading-slide 1.2s ease-in-out infinite;
}

@keyframes loading-slide {
  0% {
    transform: translateX(-120%);
  }
  100% {
    transform: translateX(340%);
  }
}
</style>
