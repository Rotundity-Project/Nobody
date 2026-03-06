<template>
  <div class="space-y-3 text-center">
    <p v-if="message" class="continue-message text-sm">{{ message }}</p>
    <button
      class="continue-btn rounded-lg px-4 py-2"
      @click="$emit('continue')"
    >
      {{ buttonText }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  message?: string;
  buttonText: string;
}>();

defineEmits<{
  continue: [];
}>();
</script>

<style scoped>
.continue-message {
  color: var(--ink-text-muted);
}

.continue-btn {
  background: var(--ink-accent-main);
  color: var(--ink-text-primary);
  transition:
    background-color var(--ink-motion-fast, 180ms) ease,
    box-shadow var(--ink-motion-fast, 180ms) ease;
}

.continue-btn:hover {
  background: var(--ink-accent-seal);
  box-shadow: var(--ink-action-shadow-hover);
}
</style>
