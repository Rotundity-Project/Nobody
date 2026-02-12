import { describe, expect, it } from 'vitest';
import { buildNovelExportFilename, shouldShowNovelExporter } from './novelExporter';

describe('novelExporter utils', () => {
  it('shows exporter only when game is running', () => {
    expect(shouldShowNovelExporter(true)).toBe(true);
    expect(shouldShowNovelExporter(false)).toBe(false);
  });

  it('builds export filenames with fallback', () => {
    expect(buildNovelExportFilename('Journey of Immortal')).toBe(
      'Journey_of_Immortal.txt',
    );
    expect(buildNovelExportFilename('***')).toBe('修仙旅程记录.txt');
  });
});
