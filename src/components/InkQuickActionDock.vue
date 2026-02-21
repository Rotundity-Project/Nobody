<template>
  <div
    class="ink-dock rounded-2xl px-3 py-2 sm:px-4"
    role="group"
    aria-label="常用操作"
  >
    <button
      type="button"
      class="ink-dock-btn"
      @click="$emit('open-character')"
    >
      人物
    </button>
    <button
      type="button"
      class="ink-dock-btn"
      @click="$emit('open-info')"
    >
      行旅
    </button>
    <button
      type="button"
      class="ink-dock-btn ink-dock-btn-primary"
      :disabled="!isGameInitialized"
      @click="$emit('open-save')"
    >
      存档
    </button>
    <button
      type="button"
      class="ink-dock-btn"
      @click="$emit('open-load')"
    >
      读档
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
