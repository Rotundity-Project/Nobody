export function shouldShowNovelExporter(isGameEnded: boolean): boolean {
  return isGameEnded;
}

export function buildNovelExportFilename(title: string): string {
  const normalized = title
    .trim()
    .replace(/[^a-zA-Z0-9 _-]/g, '')
    .replace(/\s+/g, '_');
  const safeTitle = normalized.length > 0 ? normalized : 'journey_record';
  return `${safeTitle}.txt`;
}
