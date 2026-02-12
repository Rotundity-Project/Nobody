export function shouldShowNovelExporter(isGameRunning: boolean): boolean {
  return isGameRunning;
}

export function buildNovelExportFilename(title: string): string {
  const normalized = title
    .trim()
    .replace(/[^\u4e00-\u9fa5a-zA-Z0-9 _-]/g, '')
    .replace(/\s+/g, '_');
  const safeTitle = normalized.length > 0 ? normalized : '修仙旅程记录';
  return `${safeTitle}.txt`;
}
