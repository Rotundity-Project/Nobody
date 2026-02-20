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
    return 'cursor-not-allowed bg-slate-700 text-slate-400';
  }

  switch (props.variant) {
    case 'primary':
      return 'bg-amber-500 text-slate-900 hover:bg-amber-400';
    case 'danger':
      return 'bg-red-700 text-white hover:bg-red-600';
    case 'info':
      return 'bg-sky-700 text-white hover:bg-sky-600';
    default:
      return 'bg-slate-700 text-white hover:bg-slate-600';
  }
});

const sizeClass = computed(() => (
  props.size === 'sm' ? 'rounded px-3 py-1 text-sm' : 'rounded-lg px-4 py-2 text-sm'
));

const buttonClass = computed(() => [
  'transition-colors duration-200 disabled:opacity-60',
  sizeClass.value,
  variantClass.value,
]);
</script>