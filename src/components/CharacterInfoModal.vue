<template>
  <div
    v-if="isOpen"
    class="ink-character-modal-overlay fixed inset-0 z-50 flex items-center justify-center p-4"
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
  border: 1px solid var(--modal-border);
  background: var(--modal-bg);
  box-shadow: var(--modal-shadow);
  backdrop-filter: blur(6px);
}

.ink-character-modal-overlay {
  background: var(--modal-overlay-bg);
}

.ink-close-btn {
  border: 1px solid var(--ink-accent-main);
  border-radius: 8px;
  background: color-mix(in srgb, var(--ink-accent-main) 10%, var(--ink-paper));
  color: color-mix(in srgb, var(--ink-accent-main) 82%, var(--ink-text-primary));
  padding: 7px 15px;
  font-size: 14px;
  transition: border-color 180ms ease, background-color 180ms ease, transform 120ms ease;
}

.ink-close-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}

.ink-close-btn:active {
  transform: scale(0.97);
}
</style>
