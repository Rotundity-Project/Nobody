<template>
  <button
    v-if="visible"
    class="absolute right-4 bottom-24 z-10 p-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-white shadow-lg transition-colors"
    title="滚动到底部"
    @click="$emit('scroll')"
  >
    <svg
      class="w-5 h-5"
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
