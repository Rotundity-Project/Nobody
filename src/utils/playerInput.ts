import type { PlayerAction, PlayerOption } from '../types/game';
import { ActionType } from '../types/game';

export interface InputValidationResult {
  valid: boolean;
  message: string;
}

const MAX_FREE_TEXT_LENGTH = 200;

export function validateFreeTextInput(text: string): InputValidationResult {
  const trimmed = text.trim();
  if (!trimmed) {
    return { valid: false, message: 'Input cannot be empty.' };
  }
  if (trimmed.length > MAX_FREE_TEXT_LENGTH) {
    return {
      valid: false,
      message: `Input is too long. Max ${MAX_FREE_TEXT_LENGTH} characters.`,
    };
  }
  return {
    valid: true,
    message: `Length ${trimmed.length}/${MAX_FREE_TEXT_LENGTH}`,
  };
}

export function createOptionAction(option: PlayerOption): PlayerAction {
  return {
    action_type: ActionType.SelectedOption,
    content: option.description,
    selected_option_id: option.id,
  };
}

export function createFreeTextAction(text: string): PlayerAction {
  return {
    action_type: ActionType.FreeText,
    content: text.trim(),
    selected_option_id: null,
  };
}

export function toggleInputMode(mode: 'options' | 'freeText'): 'options' | 'freeText' {
  return mode === 'options' ? 'freeText' : 'options';
}
