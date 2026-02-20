<template>
  <div
    v-if="visible"
    class="border-b border-slate-800/80 bg-slate-900/60 px-3 py-1.5 sm:px-6 sm:py-2"
  >
    <div class="mx-auto max-w-7xl overflow-x-auto">
      <div class="flex min-w-max items-center gap-1.5 text-[11px] text-slate-300 sm:gap-2 sm:text-xs">
      <span class="rounded border border-slate-700/70 bg-slate-900/60 px-2 py-0.5">章节：{{ chapterProgress }}</span>
      <span class="rounded border border-slate-700/70 bg-slate-900/60 px-2 py-0.5">交互：{{ chapterInteraction }}</span>
      <span class="rounded border border-slate-700/70 bg-slate-900/60 px-2 py-0.5">状态：{{ interactionState }}</span>
      <span
        v-if="optionSourceLabel"
        class="rounded border border-slate-700/70 bg-slate-900/60 px-2 py-0.5"
      >
        来源：{{ optionSourceLabel }}
      </span>
      <span
        v-if="optionSourceHint"
        class="rounded border border-amber-400/40 bg-amber-400/10 px-2 py-0.5 text-[11px] text-amber-200"
      >
        {{ optionSourceHint }}
      </span>
      </div>
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
