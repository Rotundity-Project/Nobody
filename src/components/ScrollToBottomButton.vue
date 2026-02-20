<template>
  <button
    v-if="visible"
    class="absolute bottom-24 right-4 z-10 rounded-lg bg-slate-700 p-2 text-white shadow-lg transition-colors hover:bg-slate-600"
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
