import { describe, expect, it } from 'vitest';
import { ActionType } from '../types/game';
import type { PlayerOption } from '../types/game';
import {
  createFreeTextAction,
  createOptionAction,
  toggleInputMode,
  validateFreeTextInput,
} from './playerInput';

describe('playerInput utils', () => {
  it('toggles input mode', () => {
    expect(toggleInputMode('options')).toBe('freeText');
    expect(toggleInputMode('freeText')).toBe('options');
  });

  it('builds free text payload', () => {
    const action = createFreeTextAction(' explore the forest ');
    expect(action.action_type).toBe(ActionType.FreeText);
    expect(action.content).toBe('explore the forest');
    expect(action.selected_option_id).toBeNull();

    const validation = validateFreeTextInput('explore the forest');
    expect(validation.valid).toBe(true);
  });

  it('builds option payload', () => {
    const option: PlayerOption = {
      id: 2,
      description: 'Cultivate in meditation room',
      requirements: [],
      action: { Cultivate: null },
    };

    const action = createOptionAction(option);
    expect(action.action_type).toBe(ActionType.SelectedOption);
    expect(action.selected_option_id).toBe(2);
    expect(action.content).toBe(option.description);
  });
});
