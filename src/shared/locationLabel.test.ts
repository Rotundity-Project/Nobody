import { describe, expect, it } from 'vitest';
import { formatLocationLabel } from './locationLabel';

describe('locationLabel', () => {
  it('formats known fallback ids in Chinese', () => {
    expect(formatLocationLabel('sect_valley')).toBe('宗门外谷');
  });

  it('formats unknown compound ids with translated tokens when possible', () => {
    expect(formatLocationLabel('sect_peak')).toBe('宗门山峰');
    expect(formatLocationLabel('river_town')).toBe('河城镇');
  });

  it('keeps separator for mixed translated and unknown tokens', () => {
    expect(formatLocationLabel('sect_ruins')).toBe('宗门 / ruins');
  });

  it('formats single-token and non-underscore location ids', () => {
    expect(formatLocationLabel('valley')).toBe('山谷');
    expect(formatLocationLabel('stone-forest')).toBe('乱石林');
    expect(formatLocationLabel('river town')).toBe('河城镇');
  });
});
