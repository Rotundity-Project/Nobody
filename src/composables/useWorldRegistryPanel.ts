import { computed, ref, watch } from 'vue';
import { type useGameStore } from '../stores/gameStore';

export type RegistryTable =
  | 'characters'
  | 'map_nodes'
  | 'map_edges'
  | 'techniques'
  | 'inventory_items'
  | 'factions'
  | 'story_state'
  | 'world_facts';

const WORLD_REGISTRY_PATCH_TEMPLATE = `{
  "world_facts": [
    {
      "fact_id": "fact_manual_1",
      "subject": "player",
      "predicate": "goal",
      "object": "secure_supplies"
    }
  ]
}`;

const WORLD_REGISTRY_DEFAULT_KEY_MAP: Record<RegistryTable, string> = {
  characters: 'character_id',
  map_nodes: 'location_id',
  map_edges: 'from_id',
  techniques: 'technique_id',
  inventory_items: 'item_id',
  factions: 'faction_id',
  story_state: 'chapter_index',
  world_facts: 'fact_id',
};

const WORLD_REGISTRY_TABLE_OPTIONS: readonly RegistryTable[] = [
  'characters',
  'map_nodes',
  'map_edges',
  'techniques',
  'inventory_items',
  'factions',
  'story_state',
  'world_facts',
] as const;

const buildMinimalRowTemplateForTable = (
  table: RegistryTable,
  playerLocation: string,
): Record<string, unknown> => {
  switch (table) {
    case 'characters':
      return {
        character_id: 'char_manual_1',
        name: 'NewCharacter',
        role: 'npc',
        realm_stage: 'Qi',
        realm_substage: 0,
        location_id: playerLocation,
      };
    case 'map_nodes':
      return {
        location_id: 'loc_manual_1',
        name: 'NewLocation',
        description: 'newly discovered location',
        spiritual_density: 0.5,
      };
    case 'map_edges':
      return {
        from_id: playerLocation,
        to_id: 'loc_manual_1',
        travel_days: 1,
        travel_risk: 0,
      };
    case 'techniques':
      return {
        technique_id: 'tech_manual_1',
        name: 'ManualTechnique',
        description: 'generated from panel template',
        owner_character_id: 'player',
      };
    case 'inventory_items':
      return {
        item_id: 'item_manual_1',
        owner_character_id: 'player',
        name: 'ManualItem',
        item_type: 'material',
        quantity: 1,
        effect_desc: 'no effect',
      };
    case 'factions':
      return {
        faction_id: 'faction_manual_1',
        name: 'ManualFaction',
        description: 'new faction',
      };
    case 'story_state':
      return {
        chapter_index: 1,
        chapter_goal: 'clear immediate objective',
        current_arc: 'manual_arc',
        pending_conflicts: ['resource_shortage'],
      };
    default:
      return {
        fact_id: 'fact_manual_3',
        subject: 'player',
        predicate: 'intent',
        object: 'explore',
      };
  }
};

export const useWorldRegistryPanel = (
  gameStore: ReturnType<typeof useGameStore>,
  playClick: () => void,
) => {
  const worldRegistryPatchInput = ref<string>(WORLD_REGISTRY_PATCH_TEMPLATE);
  const worldRegistryPatchSubmitting = ref(false);
  const worldRegistryPatchError = ref('');
  const worldRegistrySelectedTable = ref<RegistryTable>('world_facts');
  const worldRegistryRowInput = ref<string>(
    JSON.stringify(
      {
        fact_id: 'fact_manual_2',
        subject: 'player',
        predicate: 'plan',
        object: 'visit_market',
      },
      null,
      2,
    ),
  );
  const worldRegistryRowError = ref('');
  const worldRegistrySelectedIndex = ref<number>(0);
  const worldRegistryKeyField = ref<string>(WORLD_REGISTRY_DEFAULT_KEY_MAP.world_facts);
  const worldRegistryRowPage = ref(0);
  const worldRegistryRowPageSize = 6;

  const worldRegistrySessionLabel = computed(() => {
    const sid = gameStore.worldRegistry?.session_id?.trim();
    if (!sid) return '未注册';
    return sid.length > 20 ? `${sid.slice(0, 20)}...` : sid;
  });

  const worldRegistrySourceLabel = computed(() => gameStore.worldRegistry?.source || 'unknown');

  const worldRegistryCounts = computed(() => {
    const tables = gameStore.worldRegistry?.tables;
    return {
      characters: tables?.characters?.length ?? 0,
      map_nodes: tables?.map_nodes?.length ?? 0,
      map_edges: tables?.map_edges?.length ?? 0,
      techniques: tables?.techniques?.length ?? 0,
      inventory_items: tables?.inventory_items?.length ?? 0,
      factions: tables?.factions?.length ?? 0,
      story_state: tables?.story_state?.length ?? 0,
      world_facts: tables?.world_facts?.length ?? 0,
    };
  });

  const worldRegistryPreview = computed(() => {
    if (!gameStore.worldRegistry) return '{}';
    try {
      return JSON.stringify(gameStore.worldRegistry, null, 2);
    } catch {
      return '{}';
    }
  });

  const worldRegistrySelectedTableRows = computed(() => {
    const table = worldRegistrySelectedTable.value;
    const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
    return rows.map((row, index) => {
      const key = worldRegistryKeyField.value.trim();
      const asRecord = row as Record<string, unknown>;
      const keyVal = key ? asRecord?.[key] : undefined;
      const label = keyVal !== undefined
        ? `${String(keyVal)}`
        : JSON.stringify(row).slice(0, 80);
      return { index, label };
    });
  });

  const worldRegistrySelectedTableRowsPaged = computed(() => {
    const start = worldRegistryRowPage.value * worldRegistryRowPageSize;
    return worldRegistrySelectedTableRows.value.slice(start, start + worldRegistryRowPageSize);
  });

  const refreshWorldRegistryPanel = async () => {
    playClick();
    await gameStore.refreshWorldRegistry();
  };

  const resetWorldRegistryPatchTemplate = () => {
    playClick();
    worldRegistryPatchInput.value = WORLD_REGISTRY_PATCH_TEMPLATE;
    worldRegistryPatchError.value = '';
  };

  const applyWorldRegistryPatchFromPanel = async () => {
    worldRegistryPatchError.value = '';
    let parsed: unknown;
    try {
      parsed = JSON.parse(worldRegistryPatchInput.value);
    } catch {
      worldRegistryPatchError.value = 'Patch JSON 解析失败，请检查格式。';
      return;
    }
    try {
      worldRegistryPatchSubmitting.value = true;
      await gameStore.applyWorldRegistryPatch(parsed);
    } catch (error) {
      worldRegistryPatchError.value = error instanceof Error ? error.message : String(error);
    } finally {
      worldRegistryPatchSubmitting.value = false;
    }
  };

  const appendRowToRegistryTable = async () => {
    worldRegistryRowError.value = '';
    let row: unknown;
    try {
      row = JSON.parse(worldRegistryRowInput.value);
    } catch {
      worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
      return;
    }
    if (!row || typeof row !== 'object' || Array.isArray(row)) {
      worldRegistryRowError.value = '行 JSON 必须是对象。';
      return;
    }
    const table = worldRegistrySelectedTable.value;
    const patch = { [table]: [row] };
    try {
      worldRegistryPatchSubmitting.value = true;
      await gameStore.applyWorldRegistryPatch(patch);
    } catch (error) {
      worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
    } finally {
      worldRegistryPatchSubmitting.value = false;
    }
  };

  const loadSelectedTableFirstRowTemplate = () => {
    playClick();
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
    const row = rows.length > 0 ? rows[0] : {};
    worldRegistryRowInput.value = JSON.stringify(row, null, 2);
  };

  const loadMinimalRowTemplateForSelectedTable = () => {
    playClick();
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const playerLocation = gameStore.playerCharacter?.location || 'sect';
    worldRegistryRowInput.value = JSON.stringify(
      buildMinimalRowTemplateForTable(table, playerLocation),
      null,
      2,
    );
  };

  const upsertRowByKeyInRegistryTable = async () => {
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const keyField = worldRegistryKeyField.value.trim();
    if (!keyField) {
      worldRegistryRowError.value = '主键字段不能为空。';
      return;
    }
    let row: unknown;
    try {
      row = JSON.parse(worldRegistryRowInput.value);
    } catch {
      worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
      return;
    }
    if (!row || typeof row !== 'object' || Array.isArray(row)) {
      worldRegistryRowError.value = '行 JSON 必须是对象。';
      return;
    }
    const patch = { [table]: [{ __op: 'upsert_by_key', __key_field: keyField, row }] };
    try {
      worldRegistryPatchSubmitting.value = true;
      await gameStore.applyWorldRegistryPatch(patch);
    } catch (error) {
      worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
    } finally {
      worldRegistryPatchSubmitting.value = false;
    }
  };

  const loadSelectedTableRowByIndex = () => {
    playClick();
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
    const rows = gameStore.worldRegistry?.tables?.[table] ?? [];
    if (idx >= rows.length) {
      worldRegistryRowError.value = `索引越界：${idx}，当前 ${table} 共 ${rows.length} 行。`;
      return;
    }
    worldRegistryRowInput.value = JSON.stringify(rows[idx], null, 2);
  };

  const replaceRowInRegistryTable = async () => {
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
    let row: unknown;
    try {
      row = JSON.parse(worldRegistryRowInput.value);
    } catch {
      worldRegistryRowError.value = '行 JSON 解析失败，请检查格式。';
      return;
    }
    if (!row || typeof row !== 'object' || Array.isArray(row)) {
      worldRegistryRowError.value = '行 JSON 必须是对象。';
      return;
    }
    const patch = { [table]: [{ __op: 'replace', __index: idx, row }] };
    try {
      worldRegistryPatchSubmitting.value = true;
      await gameStore.applyWorldRegistryPatch(patch);
    } catch (error) {
      worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
    } finally {
      worldRegistryPatchSubmitting.value = false;
    }
  };

  const deleteRowInRegistryTable = async () => {
    worldRegistryRowError.value = '';
    const table = worldRegistrySelectedTable.value;
    const idx = Math.max(0, Number(worldRegistrySelectedIndex.value) || 0);
    const patch = { [table]: [{ __op: 'delete', __index: idx }] };
    try {
      worldRegistryPatchSubmitting.value = true;
      await gameStore.applyWorldRegistryPatch(patch);
    } catch (error) {
      worldRegistryRowError.value = error instanceof Error ? error.message : String(error);
    } finally {
      worldRegistryPatchSubmitting.value = false;
    }
  };

  watch(worldRegistrySelectedTable, (table) => {
    worldRegistryRowPage.value = 0;
    worldRegistryKeyField.value = WORLD_REGISTRY_DEFAULT_KEY_MAP[table] ?? 'id';
    const playerLocation = gameStore.playerCharacter?.location || 'sect';
    worldRegistryRowInput.value = JSON.stringify(
      buildMinimalRowTemplateForTable(table, playerLocation),
      null,
      2,
    );
  });

  return {
    worldRegistrySessionLabel,
    worldRegistrySourceLabel,
    worldRegistryCounts,
    worldRegistryPreview,
    worldRegistryPatchInput,
    worldRegistryPatchSubmitting,
    worldRegistryPatchError,
    worldRegistrySelectedTable,
    worldRegistryTableOptions: WORLD_REGISTRY_TABLE_OPTIONS,
    worldRegistryRowInput,
    worldRegistryRowError,
    worldRegistrySelectedIndex,
    worldRegistryKeyField,
    worldRegistryRowPage,
    worldRegistryRowPageSize,
    worldRegistrySelectedTableRows,
    worldRegistrySelectedTableRowsPaged,
    refreshWorldRegistryPanel,
    resetWorldRegistryPatchTemplate,
    applyWorldRegistryPatchFromPanel,
    appendRowToRegistryTable,
    loadSelectedTableFirstRowTemplate,
    loadMinimalRowTemplateForSelectedTable,
    upsertRowByKeyInRegistryTable,
    loadSelectedTableRowByIndex,
    replaceRowInRegistryTable,
    deleteRowInRegistryTable,
  };
};
