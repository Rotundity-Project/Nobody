<template>
  <div v-if="visible" class="space-y-2">
    <button
      v-for="(option, index) in options"
      :key="index"
      :disabled="disabled"
      class="w-full rounded-lg border-2 p-4 text-left transition-all duration-200"
      :class="[
        disabled
          ? 'cursor-not-allowed border-gray-600 bg-slate-700 opacity-50'
          : 'cursor-pointer border-amber-400/60 bg-slate-800/80 hover:bg-slate-700'
      ]"
      @click="$emit('select', option)"
    >
      <p class="text-slate-100">{{ option.description }}</p>
      <p
        v-if="option.requirements && option.requirements.length > 0"
        class="mt-1 text-sm text-slate-400"
      >
        条件：{{ option.requirements.join('，') }}
      </p>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { PlayerOption } from '../types/game';

defineProps<{
  visible: boolean;
  options: PlayerOption[];
  disabled: boolean;
}>();

defineEmits<{
  select: [option: PlayerOption];
}>();
</script>