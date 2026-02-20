import { computed, ref } from 'vue';

export type InputMode = 'options' | 'freeText';

export type InputValidation = {
  valid: boolean;
  message: string;
};

export const useInputMode = (
  validateFreeTextInput: (text: string) => InputValidation,
) => {
  const inputMode = ref<InputMode>('options');
  const freeTextInput = ref('');

  const inputValidation = computed(() => validateFreeTextInput(freeTextInput.value));

  const setInputMode = (mode: InputMode) => {
    inputMode.value = mode;
  };

  return {
    inputMode,
    freeTextInput,
    inputValidation,
    setInputMode,
  };
};