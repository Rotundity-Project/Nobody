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

    <div class="pointer-events-none absolute bottom-4 right-4 flex w-full max-w-sm flex-col gap-2 px-4">
      <div
        v-for="item in toasts"
        :key="item.id"
        class="pointer-events-auto rounded-lg border border-slate-700 bg-slate-900/95 p-3 shadow-lg"
      >
        <div class="mb-1 flex items-center justify-between gap-3">
          <p class="text-sm font-medium text-slate-100">{{ item.title }}</p>
          <button
            class="rounded bg-slate-700 px-2 py-0.5 text-xs text-slate-200 hover:bg-slate-600"
            @click="$emit('dismiss', item.id)"
          >
            关闭
          </button>
        </div>
        <p
          v-if="item.message"
          class="text-xs text-slate-300"
        >
          {{ item.message }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
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

defineEmits<{
  dismiss: [id: string];
}>();

const banner = computed(() =>
  props.notifications.find((n) => n.priority === 'banner')
    ?? null,
);
const toasts = computed(() =>
  props.notifications.filter((n) => n.priority !== 'banner'),
);
</script>
