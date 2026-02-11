import { buildNovelExportFilename, shouldShowNovelExporter } from './novelExporter';

function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

export function test_exporter_shown_when_game_ends(): void {
  assert(shouldShowNovelExporter(true), 'Exporter should show when game ends');
  assert(!shouldShowNovelExporter(false), 'Exporter should hide during active game');
}

export function test_export_filename_generation(): void {
  assert(
    buildNovelExportFilename('Journey of Immortal') === 'Journey_of_Immortal.txt',
    'Filename should normalize spaces',
  );
  assert(
    buildNovelExportFilename('***') === 'journey_record.txt',
    'Filename should fallback for invalid title',
  );
}
