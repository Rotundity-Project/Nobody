import { onMounted, onUnmounted } from 'vue';

export const useGameHotkeys = (handler: (event: KeyboardEvent) => void) => {
  onMounted(() => {
    window.addEventListener('keydown', handler);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handler);
  });
};