<template>
  <div
    v-if="visible"
    class="border-b border-slate-800/80 bg-slate-900/60 px-6 py-2"
  >
    <div class="mx-auto flex max-w-7xl flex-wrap items-center gap-x-5 gap-y-1 text-xs text-slate-300">
      <span>章节：{{ chapterProgress }}</span>
      <span>交互：{{ chapterInteraction }}</span>
      <span>状态：{{ interactionState }}</span>
      <span v-if="optionSourceLabel">来源：{{ optionSourceLabel }}</span>
      <span
        v-if="optionSourceHint"
        class="rounded border border-amber-400/40 bg-amber-400/10 px-2 py-0.5 text-[11px] text-amber-200"
      >
        {{ optionSourceHint }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  chapterProgress: string;
  chapterInteraction: string;
  interactionState: string;
  optionSourceLabel?: string;
  optionSourceHint?: string;
}>();
</script>
