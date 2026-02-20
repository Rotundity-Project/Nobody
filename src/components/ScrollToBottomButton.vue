<template>
  <button
    v-if="visible"
    type="button"
    class="absolute bottom-20 right-4 z-10 inline-flex items-center gap-1.5 rounded-full border border-slate-500/70 bg-slate-800/95 px-3 py-1.5 text-xs text-slate-100 shadow-lg transition-colors hover:bg-slate-700"
    title="滚动到底部"
    aria-label="滚动到底部"
    @click="$emit('scroll')"
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
        stroke-width="2"
        d="M19 14l-7 7m0 0l-7-7m7 7V3"
      />
    </svg>
    <span>回到底部</span>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
}>();

defineEmits<{
  (event: 'scroll'): void;
}>();
</script>
