<template>
  <div
    v-if="visible"
    class="free-text-panel"
  >
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
  border: 1px solid var(--ink-border-strong);
  background: var(--ink-paper);
  padding: 12px 16px;
  color: var(--ink-text-primary);
  line-height: 1.65;
  outline: none;
  transition: border-color 180ms ease, box-shadow 180ms ease, background-color 180ms ease;
}

.free-text-input::placeholder {
  color: var(--free-text-placeholder);
}

.free-text-input:focus {
  border-color: var(--ink-title-color);
  box-shadow: 0 0 0 2px var(--free-text-focus-ring);
  background: var(--ink-paper-elevated);
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
  color: var(--ink-text-muted);
}

.free-text-validation-error {
  color: var(--ink-accent-main);
}

.free-text-submit {
  border-radius: 8px;
  border: 1px solid var(--ink-border-accent);
  padding: 8px 14px;
  font-size: 14px;
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.free-text-submit-enabled {
  background: var(--ink-paper-elevated);
  color: var(--ink-text-primary);
}

.free-text-submit-enabled:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
  box-shadow: 0 3px 10px var(--ink-action-shadow-hover);
}

.free-text-submit-enabled:active {
  transform: scale(0.98);
}

.free-text-submit-disabled {
  cursor: not-allowed;
  border-color: var(--ink-border-soft);
  background: var(--ink-card-bg-muted);
  color: var(--free-text-disabled-text);
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
