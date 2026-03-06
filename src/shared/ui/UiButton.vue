<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="buttonClass"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'neutral' | 'primary' | 'danger' | 'info';
  size?: 'sm' | 'md';
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
}>(), {
  variant: 'neutral',
  size: 'md',
  disabled: false,
  type: 'button',
});

defineEmits<{
  (event: 'click', payload: MouseEvent): void;
}>();

const variantClass = computed(() => {
  if (props.disabled) {
    return 'ui-btn-disabled cursor-not-allowed';
  }

  switch (props.variant) {
    case 'primary':
      return 'ui-btn-primary';
    case 'danger':
      return 'ui-btn-danger';
    case 'info':
      return 'ui-btn-info';
    default:
      return 'ui-btn-neutral';
  }
});

const sizeClass = computed(() => (
  props.size === 'sm' ? 'rounded px-3 py-1 text-sm' : 'rounded-lg px-4 py-2 text-sm'
));

const buttonClass = computed(() => [
  'ui-btn-base transition-colors duration-200 disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-1',
  sizeClass.value,
  variantClass.value,
]);
</script>

<style scoped>
.ui-btn-base {
  border: 1px solid transparent;
}

.ui-btn-base:focus-visible {
  --tw-ring-color: var(--ui-btn-focus-ring);
  --tw-ring-offset-color: var(--ui-btn-focus-ring-offset);
}

.ui-btn-neutral {
  background: var(--ink-card-bg-muted);
  color: var(--ink-text-primary);
  border-color: var(--ink-border-soft);
}

.ui-btn-neutral:hover {
  background: var(--ink-card-bg-soft);
}

.ui-btn-primary {
  background: var(--ui-btn-primary-bg);
  color: var(--ui-btn-primary-text);
  border-color: var(--ui-btn-primary-border);
}

.ui-btn-primary:hover {
  background: var(--ui-btn-primary-hover-bg);
}

.ui-btn-danger {
  background: var(--ui-btn-danger-bg);
  color: var(--ui-btn-danger-text);
  border-color: var(--ui-btn-danger-border);
}

.ui-btn-danger:hover {
  background: var(--ui-btn-danger-hover-bg);
}

.ui-btn-info {
  background: var(--ui-btn-info-bg);
  color: var(--ui-btn-info-text);
  border-color: var(--ui-btn-info-border);
}

.ui-btn-info:hover {
  background: var(--ui-btn-info-hover-bg);
}

.ui-btn-disabled {
  background: var(--ink-card-bg-muted);
  color: var(--ink-text-muted);
  border-color: var(--ink-border-soft);
}
</style>
