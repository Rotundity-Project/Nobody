<template>
  <div class="pointer-events-none fixed inset-0 z-[70]">
    <div
      v-if="banner"
      class="pointer-events-auto mx-auto mt-4 max-w-3xl px-4"
    >
      <StatusBanner
        :kind="banner.kind"
        :title="banner.title"
        :message="banner.message"
      />
    </div>

    <div class="notify-toast-stack pointer-events-none absolute flex w-full flex-col gap-2 px-4">
      <div
        v-for="item in toasts"
        :key="item.id"
        class="notify-toast pointer-events-auto rounded-lg p-3 shadow-lg"
        :class="`notify-toast-${item.kind}`"
      >
        <div class="mb-1 flex items-center justify-between gap-3">
          <p class="notify-title">
            {{ item.title }}
          </p>
          <button
            class="notify-close-btn rounded px-2 py-0.5 text-xs"
            @click="emit('dismiss', item.id)"
          >
            关闭
          </button>
        </div>
        <p
          v-if="item.message"
          class="notify-message"
        >
          {{ item.message }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue';
import StatusBanner from './StatusBanner.vue';

export type NotificationItem = {
  id: string;
  kind: 'error' | 'info' | 'auto_advance' | 'validation';
  title: string;
  message?: string;
  priority?: 'banner' | 'toast';
};

const props = defineProps<{
  notifications: NotificationItem[];
}>();

const emit = defineEmits<{
  dismiss: [id: string];
}>();

const banner = computed(() =>
  props.notifications.find((n) => n.priority === 'banner')
    ?? null,
);
const toasts = computed(() =>
  props.notifications.filter((n) => n.priority !== 'banner'),
);

const AUTO_DISMISS_MS = 2600;
const autoDismissKinds = new Set<NotificationItem['kind']>(['info', 'auto_advance', 'validation']);
const toastTimers = new Map<string, number>();

const clearToastTimer = (id: string) => {
  const timer = toastTimers.get(id);
  if (timer != null) {
    window.clearTimeout(timer);
    toastTimers.delete(id);
  }
};

watch(
  toasts,
  (nextToasts) => {
    const activeIds = new Set(nextToasts.map((item) => item.id));
    Array.from(toastTimers.keys())
      .filter((id) => !activeIds.has(id))
      .forEach((id) => clearToastTimer(id));

    nextToasts.forEach((item) => {
      if (!autoDismissKinds.has(item.kind) || toastTimers.has(item.id)) {
        return;
      }
      const timer = window.setTimeout(() => {
        toastTimers.delete(item.id);
        emit('dismiss', item.id);
      }, AUTO_DISMISS_MS);
      toastTimers.set(item.id, timer);
    });
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  Array.from(toastTimers.keys()).forEach((id) => clearToastTimer(id));
});
</script>

<style scoped>
.notify-toast {
  border: 1px solid var(--ink-border-soft);
  background: var(--notify-toast-bg);
  max-width: var(--ui-theme-status-max-width);
  padding: var(--ui-theme-status-padding-y) var(--ui-theme-status-padding-x);
  box-shadow: var(--ui-theme-status-shadow);
  animation: notify-toast-fade-in 180ms ease-out;
}

.notify-title {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: var(--ui-theme-status-title-size);
  font-weight: var(--ui-theme-status-title-weight);
  line-height: var(--ui-theme-status-title-line-height);
  letter-spacing: var(--ui-theme-status-title-letter-spacing);
}

.notify-message {
  margin-bottom: 0;
  color: var(--ink-text-muted);
  margin-top: var(--space-1);
  font-size: var(--ui-theme-status-message-size);
  line-height: var(--ui-theme-status-message-line-height);
  opacity: var(--ui-theme-status-message-opacity);
}

.notify-close-btn {
  border: 1px solid var(--ink-border-soft);
  background: var(--notify-close-btn-bg);
  color: var(--ink-text-primary);
}

.notify-close-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}

.notify-toast-info {
  border-color: var(--ink-border-soft);
  background: var(--status-default-bg);
}

.notify-toast-error {
  border-color: var(--status-error-border);
  background: var(--status-error-bg);
}

.notify-toast-validation {
  border-color: var(--status-validation-border);
  background: var(--status-validation-bg);
}

.notify-toast-auto_advance {
  border-color: var(--status-auto-border);
  background: var(--status-auto-bg);
}

.notify-toast-error .notify-title,
.notify-toast-error .notify-message {
  color: var(--status-error-text);
}

.notify-toast-validation .notify-title,
.notify-toast-validation .notify-message {
  color: var(--status-validation-text);
}

.notify-toast-auto_advance .notify-title,
.notify-toast-auto_advance .notify-message {
  color: var(--status-auto-text);
}

@keyframes notify-toast-fade-in {
  from {
    opacity: 0;
    transform: translateY(var(--ui-theme-status-entry-offset-y));
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.notify-toast-stack {
  right: var(--ui-theme-status-inline-offset);
  bottom: var(--ui-theme-status-block-offset);
  max-width: calc(var(--ui-theme-status-max-width) + var(--space-7));
}

@media (prefers-reduced-motion: reduce) {
  .notify-toast {
    animation: none !important;
  }
}
</style>
