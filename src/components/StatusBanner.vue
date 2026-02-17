<template>
  <div :class="wrapperClass" role="status" aria-live="polite">
    <p class="text-sm font-medium">{{ title }}</p>
    <p v-if="message" class="mt-1 text-xs opacity-90">{{ message }}</p>
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

const wrapperClass = computed(() => {
  switch (props.kind) {
    case 'loading':
      return 'rounded border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-amber-100';
    case 'success':
      return 'rounded border border-emerald-500/40 bg-emerald-500/10 px-3 py-2 text-emerald-100';
    case 'error':
      return 'rounded border border-red-500/40 bg-red-500/10 px-3 py-2 text-red-100';
    case 'auto_advance':
      return 'rounded border border-sky-500/40 bg-sky-500/10 px-3 py-2 text-sky-100';
    case 'validation':
      return 'rounded border border-violet-500/40 bg-violet-500/10 px-3 py-2 text-violet-100';
    default:
      return 'rounded border border-slate-600 bg-slate-800/70 px-3 py-2 text-slate-200';
  }
});
</script>
