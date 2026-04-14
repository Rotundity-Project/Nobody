<template>
  <div class="space-y-3 text-center">
    <LoadingIndicator
      :message="message"
      detail="请稍候，剧情正在推进..."
      :progress="progress"
      :progress-text="progressText"
      size="lg"
    />
    <p
      v-if="stage"
      class="loading-meta text-xs"
    >
      当前阶段：{{ stage }}
    </p>
    <p
      v-if="elapsedLabel"
      class="loading-meta text-[11px]"
    >
      {{ elapsedLabel }}
    </p>
    <div
      v-if="canStopAutoAdvance"
      class="space-y-2"
    >
      <p class="loading-hint text-xs">
        自动推进进行中，可随时中断并回到手动控制。
      </p>
      <button
        class="loading-stop-btn rounded px-3 py-1 text-xs"
        @click="$emit('stop-auto-advance')"
      >
        中断自动推进
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import LoadingIndicator from './LoadingIndicator.vue';

const props = withDefaults(
  defineProps<{
    message: string;
    stage?: string;
    progress?: number | null;
    progressText?: string;
    elapsedMs?: number | null;
    canStopAutoAdvance?: boolean;
  }>(),
  {
    stage: '',
    progress: null,
    progressText: '',
    elapsedMs: null,
    canStopAutoAdvance: false,
  },
);

const elapsedLabel = computed(() => {
  if (props.elapsedMs == null) {
    return '';
  }
  if (props.elapsedMs < 1000) {
    return `已耗时 ${props.elapsedMs} ms`;
  }
  return `已耗时 ${(props.elapsedMs / 1000).toFixed(1)} s`;
});

defineEmits<{
  'stop-auto-advance': [];
}>();
</script>

<style scoped>
.loading-meta {
  color: var(--ink-text-muted);
}

.loading-hint {
  color: var(--ink-text-cool);
}

.loading-stop-btn {
  background: var(--ink-card-bg-soft);
  color: var(--ink-text-primary);
  transition:
    background-color var(--ink-motion-fast, 180ms) ease,
    border-color var(--ink-motion-fast, 180ms) ease;
}

.loading-stop-btn:hover {
  background: var(--ink-card-bg-muted);
}
</style>
