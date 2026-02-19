<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
    @click.self="$emit('close')"
  >
    <div class="w-full max-w-md">
      <CharacterPanel :character="character" />
      <div class="mt-3 text-right">
        <button
          class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
          @click="$emit('close')"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import CharacterPanel from './CharacterPanel.vue';
import type { Character } from '../types/game';

defineProps<{
  isOpen: boolean;
  character: Character | null;
}>();

defineEmits<{
  (event: 'close'): void;
}>();
</script>
