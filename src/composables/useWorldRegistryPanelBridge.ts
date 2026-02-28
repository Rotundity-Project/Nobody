import { computed } from 'vue';
import { type RegistryTable, type useWorldRegistryPanel } from './useWorldRegistryPanel';

export const useWorldRegistryPanelBridge = ({
  panel,
}: {
  panel: ReturnType<typeof useWorldRegistryPanel>;
}) => {
  const panelProps = computed(() => ({
    sessionLabel: panel.worldRegistrySessionLabel.value,
    sourceLabel: panel.worldRegistrySourceLabel.value,
    counts: panel.worldRegistryCounts.value,
    preview: panel.worldRegistryPreview.value,
    patchInput: panel.worldRegistryPatchInput.value,
    patchSubmitting: panel.worldRegistryPatchSubmitting.value,
    patchError: panel.worldRegistryPatchError.value,
    selectedTable: panel.worldRegistrySelectedTable.value,
    tableOptions: panel.worldRegistryTableOptions,
    rowInput: panel.worldRegistryRowInput.value,
    rowError: panel.worldRegistryRowError.value,
    selectedIndex: panel.worldRegistrySelectedIndex.value,
    keyField: panel.worldRegistryKeyField.value,
    rowItems: panel.worldRegistrySelectedTableRows.value,
    rowItemsPaged: panel.worldRegistrySelectedTableRowsPaged.value,
    canPrevPage: panel.worldRegistryRowPage.value > 0,
    canNextPage:
      (panel.worldRegistryRowPage.value + 1) * panel.worldRegistryRowPageSize
      < panel.worldRegistrySelectedTableRows.value.length,
  }));

  const panelListeners = {
    refresh: panel.refreshWorldRegistryPanel,
    'reset-template': panel.resetWorldRegistryPatchTemplate,
    'apply-patch': panel.applyWorldRegistryPatchFromPanel,
    'append-row': panel.appendRowToRegistryTable,
    'load-first-template': panel.loadSelectedTableFirstRowTemplate,
    'load-minimal-template': panel.loadMinimalRowTemplateForSelectedTable,
    'load-row-by-index': panel.loadSelectedTableRowByIndex,
    'replace-row': panel.replaceRowInRegistryTable,
    'delete-row': panel.deleteRowInRegistryTable,
    'upsert-by-key': panel.upsertRowByKeyInRegistryTable,
    'prev-page': () => {
      panel.worldRegistryRowPage.value = Math.max(0, panel.worldRegistryRowPage.value - 1);
    },
    'next-page': () => {
      panel.worldRegistryRowPage.value += 1;
    },
    'update:patch-input': (value: string) => {
      panel.worldRegistryPatchInput.value = value;
    },
    'update:selected-table': (value: RegistryTable) => {
      panel.worldRegistrySelectedTable.value = value;
    },
    'update:row-input': (value: string) => {
      panel.worldRegistryRowInput.value = value;
    },
    'update:selected-index': (value: number) => {
      panel.worldRegistrySelectedIndex.value = value;
    },
    'update:key-field': (value: string) => {
      panel.worldRegistryKeyField.value = value;
    },
  };

  return {
    worldRegistryPanelProps: panelProps,
    worldRegistryPanelListeners: panelListeners,
  };
};
