<template>
  <div
    v-if="visible"
    class="input-mode-tabs inline-flex items-center gap-2 rounded-[10px] border p-1"
  >
    <button
      type="button"
      class="ink-tab-btn rounded-[8px] px-3 py-1 text-sm"
      :class="mode === 'options' ? 'ink-tab-btn-active' : 'ink-tab-btn-idle'"
      @click="$emit('switch-mode', 'options')"
    >
      选项
    </button>
    <button
      type="button"
      class="ink-tab-btn rounded-[8px] px-3 py-1 text-sm"
      :class="mode === 'freeText' ? 'ink-tab-btn-active' : 'ink-tab-btn-idle'"
      @click="$emit('switch-mode', 'freeText')"
    >
      自由输入
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  mode: 'options' | 'freeText';
}>();

defineEmits<{
  'switch-mode': [mode: 'options' | 'freeText'];
}>();
</script>

<style scoped>
.ink-tab-btn {
  border: 1px solid transparent;
  transition: border-color 180ms ease, background-color 180ms ease, color 180ms ease;
}

.ink-tab-btn-active {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
  color: var(--ink-text-primary);
}

.ink-tab-btn-idle {
  color: var(--ink-text-muted);
}

.ink-tab-btn-idle:hover {
  border-color: var(--ink-border-strong);
  background: var(--ink-paper-elevated);
  color: var(--ink-text-primary);
}

.input-mode-tabs {
  border-color: var(--ink-border-strong);
  background: var(--ink-card-bg);
}
</style>
