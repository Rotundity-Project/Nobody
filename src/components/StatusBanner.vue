<template>
  <div :class="wrapperClass" role="status" aria-live="polite">
    <p class="status-title">{{ title }}</p>
    <p v-if="message" class="status-message">{{ message }}</p>
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

.status-title {
  margin: 0;
  font-size: var(--ui-theme-status-title-size);
  font-weight: var(--ui-theme-status-title-weight);
  line-height: var(--ui-theme-status-title-line-height);
  letter-spacing: var(--ui-theme-status-title-letter-spacing);
}

.status-message {
  margin-top: var(--space-1);
  margin-bottom: 0;
  font-size: var(--ui-theme-status-message-size);
  line-height: var(--ui-theme-status-message-line-height);
  opacity: var(--ui-theme-status-message-opacity);
}

.status-loading {
  border-color: var(--status-loading-border);
  background: var(--status-loading-bg);
  color: var(--status-loading-text);
}

.status-success {
  border-color: var(--status-success-border);
  background: var(--status-success-bg);
  color: var(--status-success-text);
}

.status-error {
  border-color: var(--status-error-border);
  background: var(--status-error-bg);
  color: var(--status-error-text);
}

.status-auto-advance {
  border-color: var(--status-auto-border);
  background: var(--status-auto-bg);
  color: var(--status-auto-text);
}

.status-validation {
  border-color: var(--status-validation-border);
  background: var(--status-validation-bg);
  color: var(--status-validation-text);
}

.status-default {
  border-color: var(--ink-border-soft);
  background: var(--status-default-bg);
  color: var(--ink-text-muted);
}
</style>
