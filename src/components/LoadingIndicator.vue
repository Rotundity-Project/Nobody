<template>
  <div class="flex items-center justify-center gap-3 text-center">
    <div
      class="inline-block animate-spin rounded-full border-2 border-slate-600 border-t-amber-300"
      :class="sizeClass"
    ></div>
    <div class="text-left">
      <p class="text-slate-100">{{ message }}</p>
      <p v-if="detail" class="text-xs text-slate-400">{{ detail }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    message?: string;
    detail?: string;
    size?: 'sm' | 'md' | 'lg';
  }>(),
  {
    message: '处理中...',
    detail: '',
    size: 'md',
  },
);

const sizeClass = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'h-4 w-4';
    case 'lg':
      return 'h-8 w-8';
    default:
      return 'h-6 w-6';
  }
});
</script>
