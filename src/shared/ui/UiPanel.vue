<template>
  <div
    class="panel-surface rounded-2xl"
    :class="paddingClass"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  padding?: 'md' | 'lg';
}>(), {
  padding: 'lg',
});

const paddingClass = computed(() => (props.padding === 'md' ? 'p-4' : 'p-6'));
</script>
