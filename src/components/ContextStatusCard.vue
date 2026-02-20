<template>
  <div
    v-if="visible"
    class="panel-surface rounded-xl border border-slate-700/80 px-4 py-3"
  >
    <div class="grid gap-2 text-sm text-slate-200 md:grid-cols-3">
      <div class="rounded-lg bg-slate-900/60 px-3 py-2">
        <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400">角色快照</p>
        <p class="mt-1 font-medium text-slate-100">{{ playerName }}</p>
        <p class="text-xs text-slate-300">{{ playerRealm }}</p>
      </div>
      <div class="rounded-lg bg-slate-900/60 px-3 py-2">
        <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400">章节进度</p>
        <p class="mt-1 text-slate-100">{{ chapterProgress }}</p>
        <p class="text-xs text-slate-300">{{ chapterInteraction }}</p>
      </div>
      <div class="rounded-lg bg-slate-900/60 px-3 py-2">
        <p class="text-[11px] uppercase tracking-[0.2em] text-slate-400">当前位置</p>
        <p class="mt-1 text-slate-100">{{ locationLabel }}</p>
        <p class="text-xs text-slate-300">{{ interactionStateLabel }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  playerName: string;
  playerRealm: string;
  chapterProgress: string;
  chapterInteraction: string;
  locationLabel: string;
  interactionStateLabel: string;
}>();
</script>
