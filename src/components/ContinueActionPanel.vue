<template>
  <div class="space-y-3 text-center">
    <p v-if="message" class="text-sm text-slate-400">{{ message }}</p>
    <button
      class="rounded-lg bg-amber-500 px-4 py-2 text-slate-900 transition-colors hover:bg-amber-400"
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
