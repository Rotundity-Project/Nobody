<template>
  <div
    v-if="visible"
    class="mb-6 rounded-lg border border-amber-500/30 bg-slate-950/70 p-4"
  >
    <p class="text-xs uppercase tracking-[0.25em] text-amber-200/70">
      上一章摘要
    </p>
    <p class="mt-2 whitespace-pre-wrap font-story text-[15px] leading-7 text-slate-200">
      {{ summary }}
    </p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  summary: string;
}>();
</script>
