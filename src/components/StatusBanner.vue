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
      return `${baseClass} border-[#b78c4a]/40 bg-[#faf7f2] text-[#7a5f2f]`;
    case 'success':
      return `${baseClass} border-[#3b7a6b]/35 bg-[#f3f8f6] text-[#2c675a]`;
    case 'error':
      return `${baseClass} border-[#b23e3e]/35 bg-[#fdf5f4] text-[#9a3434]`;
    case 'auto_advance':
      return `${baseClass} border-[#3b7a6b]/35 bg-[#f1f7f5] text-[#2d6b5e]`;
    case 'validation':
      return `${baseClass} border-[#8a6a3a]/35 bg-[#faf6ef] text-[#70552d]`;
    default:
      return `${baseClass} border-[#d9d0c0] bg-[#f8f4ec] text-[#5e5a54]`;
  }
});
</script>

<style scoped>
.status-title,
.status-message {
  overflow-wrap: anywhere;
  word-break: break-word;
}
</style>
