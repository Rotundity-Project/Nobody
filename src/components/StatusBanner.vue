<template>
  <div :class="wrapperClass" role="status" aria-live="polite">
    <p class="status-title text-sm font-medium">{{ title }}</p>
    <p v-if="message" class="status-message mt-1 text-xs opacity-90">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    kind?: 'loading' | 'success' | 'error' | 'info' | 'auto_advance' | 'validation';
    title: string;
    message?: string;
  }>(),
  {
    kind: 'info',
    message: '',
  },
);

const baseClass = 'rounded-lg border px-3 py-2 shadow-sm';

const wrapperClass = computed(() => {
  switch (props.kind) {
    case 'loading':
      return `${baseClass} status-loading`;
    case 'success':
      return `${baseClass} status-success`;
    case 'error':
      return `${baseClass} status-error`;
    case 'auto_advance':
      return `${baseClass} status-auto-advance`;
    case 'validation':
      return `${baseClass} status-validation`;
    default:
      return `${baseClass} status-default`;
  }
});
</script>

<style scoped>
.status-title,
.status-message {
  overflow-wrap: anywhere;
  word-break: break-word;
}

.status-loading {
  border-color: color-mix(in srgb, var(--ink-title-color) 40%, transparent);
  background: color-mix(in srgb, var(--ink-paper) 88%, transparent);
  color: color-mix(in srgb, var(--ink-title-color) 72%, var(--ink-text-primary));
}

.status-success {
  border-color: color-mix(in srgb, var(--ink-text-cool) 35%, transparent);
  background: color-mix(in srgb, var(--ink-text-cool) 11%, var(--ink-paper));
  color: var(--ink-text-cool);
}

.status-error {
  border-color: color-mix(in srgb, var(--ink-accent-main) 35%, transparent);
  background: color-mix(in srgb, var(--ink-accent-main) 10%, var(--ink-paper));
  color: var(--ink-accent-main);
}

.status-auto-advance {
  border-color: color-mix(in srgb, var(--ink-text-cool) 35%, transparent);
  background: color-mix(in srgb, var(--ink-text-cool) 9%, var(--ink-paper));
  color: color-mix(in srgb, var(--ink-text-cool) 85%, var(--ink-text-primary));
}

.status-validation {
  border-color: color-mix(in srgb, var(--ink-accent-note) 35%, transparent);
  background: color-mix(in srgb, var(--ink-accent-note) 12%, var(--ink-paper));
  color: color-mix(in srgb, var(--ink-accent-note) 76%, var(--ink-text-primary));
}

.status-default {
  border-color: var(--ink-border-soft);
  background: color-mix(in srgb, var(--ink-paper-elevated) 90%, transparent);
  color: var(--ink-text-muted);
}
</style>
