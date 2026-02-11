import { ActionType } from '../types/game';
import type { PlayerOption } from '../types/game';
import {
  createFreeTextAction,
  createOptionAction,
  toggleInputMode,
  validateFreeTextInput,
} from './playerInput';

function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

export function test_input_mode_toggle(): void {
  assert(toggleInputMode('options') === 'freeText', 'Options should switch to freeText mode');
  assert(toggleInputMode('freeText') === 'options', 'freeText should switch to options mode');
}

export function test_free_text_submission_payload(): void {
  const action = createFreeTextAction(' explore the forest ');
  assert(action.action_type === ActionType.FreeText, 'Action type must be FreeText');
  assert(action.content === 'explore the forest', 'Free text should be trimmed');
  assert(action.selected_option_id === null, 'Free text action should not carry option id');

  const valid = validateFreeTextInput('explore the forest');
  assert(valid.valid, 'Normal free text should be valid');
}

export function test_option_submission_payload(): void {
  const option: PlayerOption = {
    id: 2,
    description: 'Cultivate in meditation room',
    requirements: [],
    action: { Cultivate: null },
  };

  const action = createOptionAction(option);
  assert(action.action_type === ActionType.SelectedOption, 'Action type must be SelectedOption');
  assert(action.selected_option_id === 2, 'Selected option id should be copied');
  assert(action.content === option.description, 'Option description should be copied');
}
