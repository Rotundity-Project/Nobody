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
    return { valid: false, message: '输入不能为空。' };
  }
  if (trimmed.length > MAX_FREE_TEXT_LENGTH) {
    return {
      valid: false,
      message: `输入过长，最多 ${MAX_FREE_TEXT_LENGTH} 个字符。`,
    };
  }
  return {
    valid: true,
    message: `长度 ${trimmed.length}/${MAX_FREE_TEXT_LENGTH}`,
  };
}

export function createOptionAction(option: PlayerOption): PlayerAction {
  return {
    action_type: ActionType.SelectedOption,
    content: option.description,
    selected_option_id: option.id,
    meta: null,
  };
}

export function createFreeTextAction(text: string): PlayerAction {
  return {
    action_type: ActionType.FreeText,
    content: text.trim(),
    selected_option_id: null,
    meta: null,
  };
}

export function createContinueAction(): PlayerAction {
  return {
    action_type: ActionType.FreeText,
    content: '继续',
    selected_option_id: null,
    meta: { action_kind: 'continue' },
  };
}

export function toggleInputMode(mode: 'options' | 'freeText'): 'options' | 'freeText' {
  return mode === 'options' ? 'freeText' : 'options';
}
