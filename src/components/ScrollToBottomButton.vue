<template>
  <button
    v-if="visible"
    type="button"
    class="scroll-bottom-btn absolute bottom-20 right-4 z-10 inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs shadow-lg transition-colors"
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

<style scoped>
.scroll-bottom-btn {
  border: 1px solid var(--scroll-bottom-border);
  background: var(--scroll-bottom-bg);
  color: var(--ink-text-primary);
}

.scroll-bottom-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}
</style>
