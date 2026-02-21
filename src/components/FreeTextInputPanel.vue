<template>
  <div v-if="visible" class="free-text-panel">
    <textarea
      :value="modelValue"
      :disabled="disabled"
      rows="4"
      maxlength="200"
      placeholder="输入你的抉择，可描述行动、心念或一句对话。"
      class="free-text-input"
      @input="onInput"
    />
    <div class="free-text-foot">
      <p
        v-if="validationMessage"
        class="free-text-validation"
        :class="valid ? 'free-text-validation-ok' : 'free-text-validation-error'"
      >
        {{ validationMessage }}
      </p>
      <button
        :disabled="disabled || !valid"
        class="free-text-submit"
        :class="disabled || !valid ? 'free-text-submit-disabled' : 'free-text-submit-enabled'"
        @click="$emit('submit')"
      >
        提交自由输入
      </button>
    </div>
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

<style scoped>
.free-text-panel {
  display: grid;
  gap: 8px;
  min-width: 0;
}

.free-text-input {
  box-sizing: border-box;
  display: block;
  width: 100%;
  min-height: 132px;
  border-radius: 8px;
  border: 1px solid #d9c0b0;
  background: #ffffff;
  padding: 12px 16px;
  color: #2d2a24;
  line-height: 1.65;
  outline: none;
  transition: border-color 180ms ease, box-shadow 180ms ease, background-color 180ms ease;
}

.free-text-input::placeholder {
  color: #8c8478;
}

.free-text-input:focus {
  border-color: #b78c4a;
  box-shadow: 0 0 0 2px rgba(183, 140, 74, 0.12);
  background: #faf7f2;
}

.free-text-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex-wrap: wrap;
}

.free-text-validation {
  margin: 0;
  font-size: 13px;
}

.free-text-validation-ok {
  color: #5e5a54;
}

.free-text-validation-error {
  color: #b23e3e;
}

.free-text-submit {
  border-radius: 8px;
  border: 1px solid #b7a88c;
  padding: 8px 14px;
  font-size: 14px;
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.free-text-submit-enabled {
  background: #f8f3ea;
  color: #2d2a24;
}

.free-text-submit-enabled:hover {
  border-color: #b78c4a;
  background: #faf7f2;
  box-shadow: 0 3px 10px rgba(45, 42, 36, 0.08);
}

.free-text-submit-enabled:active {
  transform: scale(0.98);
}

.free-text-submit-disabled {
  cursor: not-allowed;
  border-color: #d9d0c0;
  background: #ece5d8;
  color: #8b857c;
}

@media (max-width: 640px) {
  .free-text-foot {
    flex-direction: column;
    align-items: stretch;
    gap: 6px;
  }

  .free-text-validation {
    width: 100%;
  }

  .free-text-submit {
    width: 100%;
    justify-content: center;
    text-align: center;
  }
}
</style>
