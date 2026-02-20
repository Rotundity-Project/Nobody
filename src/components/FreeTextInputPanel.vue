<template>
  <div v-if="visible" class="space-y-2">
    <textarea
      :value="modelValue"
      :disabled="disabled"
      rows="3"
      maxlength="200"
      placeholder="输入你想执行的行为，例如：我去后山修炼。"
      class="w-full rounded-lg border border-slate-700 bg-slate-800 p-3 text-white outline-none focus:border-amber-400"
      @input="onInput"
    />
    <p
      v-if="validationMessage"
      class="text-sm"
      :class="valid ? 'text-gray-300' : 'text-amber-300'"
    >
      {{ validationMessage }}
    </p>
    <button
      :disabled="disabled || !valid"
      class="rounded-lg px-4 py-2 transition-colors"
      :class="disabled || !valid ? 'cursor-not-allowed bg-gray-600 text-gray-400' : 'bg-amber-500 text-slate-900 hover:bg-amber-400'"
      @click="$emit('submit')"
    >
      提交自由输入
    </button>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  'update:modelValue': [value: string];
  submit: [];
}>();

defineProps<{
  visible: boolean;
  modelValue: string;
  disabled: boolean;
  valid: boolean;
  validationMessage: string;
}>();

const onInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
};
</script>