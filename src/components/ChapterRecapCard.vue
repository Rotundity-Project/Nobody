<template>
  <div
    v-if="visible"
    class="chapter-recap mb-4 rounded-lg p-4"
  >
    <p class="chapter-recap-title text-xs uppercase tracking-[0.25em]">
      上一章摘要
    </p>
    <p class="chapter-recap-summary mt-2 whitespace-pre-wrap font-story text-[15px] leading-7">
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

<style scoped>
.chapter-recap {
  border: 1px solid var(--chapter-recap-border);
  background: var(--chapter-recap-bg);
}

.chapter-recap-title {
  color: var(--chapter-recap-title);
}

.chapter-recap-summary {
  color: var(--ink-text-primary);
}
</style>
