import { describe, expect, it } from 'vitest';
import { formatLocationLabel } from './locationLabel';

describe('locationLabel', () => {
  it('formats known fallback ids in Chinese', () => {
    expect(formatLocationLabel('sect_valley')).toBe('宗门外谷');
  });

  it('formats unknown compound ids with translated tokens when possible', () => {
    expect(formatLocationLabel('sect_peak')).toBe('宗门 / 山峰');
    expect(formatLocationLabel('river_town')).toBe('河 / 城镇');
  });
});
