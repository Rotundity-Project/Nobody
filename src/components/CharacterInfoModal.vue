<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4"
    @click.self="$emit('close')"
  >
    <div class="ink-character-modal w-full max-w-md rounded-2xl p-3">
      <CharacterPanel :character="character" />
      <div class="mt-3 text-right">
        <button
          class="ink-close-btn"
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

<style scoped>
.ink-character-modal {
  border: 1px solid #d9d0c0;
  background: rgba(250, 247, 242, 0.96);
  box-shadow: 0 14px 32px rgba(45, 42, 36, 0.14);
  backdrop-filter: blur(6px);
}

.ink-close-btn {
  border: 1px solid #b23e3e;
  border-radius: 8px;
  background: rgba(178, 62, 62, 0.08);
  color: #9a3434;
  padding: 7px 15px;
  font-size: 14px;
  transition: border-color 180ms ease, background-color 180ms ease, transform 120ms ease;
}

.ink-close-btn:hover {
  border-color: #b78c4a;
  background: #faf7f2;
}

.ink-close-btn:active {
  transform: scale(0.97);
}
</style>
