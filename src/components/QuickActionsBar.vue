<template>
  <div class="flex flex-wrap gap-2">
    <button
      class="rounded-lg bg-slate-700 px-4 py-2 text-white transition-colors duration-200 hover:bg-slate-600"
      @click="$emit('open-character')"
    >
      角色信息
    </button>
    <button
      class="rounded-lg bg-slate-700 px-4 py-2 text-white transition-colors duration-200 hover:bg-slate-600"
      @click="$emit('open-info')"
    >
      信息面板
    </button>
    <button
      :disabled="!isGameInitialized"
      class="rounded-lg px-4 py-2 transition-colors duration-200"
      :class="[
        isGameInitialized
          ? 'bg-amber-500 text-slate-900 hover:bg-amber-400'
          : 'cursor-not-allowed bg-gray-600 text-gray-400'
      ]"
      @click="$emit('open-save')"
    >
      保存
    </button>
    <button
      class="rounded-lg bg-slate-700 px-4 py-2 text-white transition-colors duration-200 hover:bg-slate-600"
      @click="$emit('open-load')"
    >
      加载
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  isGameInitialized: boolean;
}>();

defineEmits<{
  'open-character': [];
  'open-info': [];
  'open-save': [];
  'open-load': [];
}>();
</script>
