import { computed, type ComputedRef } from 'vue';
import { type useGameStore } from '../stores/gameStore';

type WorldRegistryCounts = {
  characters: number;
  map_nodes: number;
  map_edges: number;
  techniques: number;
  inventory_items: number;
  factions: number;
  story_state: number;
  world_facts: number;
};

const asText = (value: unknown, fallback = ''): string => {
  if (value === null || value === undefined) return fallback;
  if (typeof value === 'string') {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : fallback;
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }
  try {
    return JSON.stringify(value);
  } catch {
    return fallback;
  }
};

const compactRecord = (row: Record<string, unknown>, ignoreKeys: string[] = []): string => {
  const ignored = new Set(ignoreKeys);
  const entries = Object.entries(row)
    .filter(([key, value]) => !ignored.has(key) && value !== null && value !== undefined && String(value).trim() !== '')
    .slice(0, 4)
    .map(([key, value]) => `${key}:${asText(value)}`);
  return entries.join(' | ');
};

export const useRuntimeQuickPanels = ({
  gameStore,
  worldRegistrySessionLabel,
  worldRegistrySourceLabel,
  worldRegistryCounts,
  currentLocationLabel,
  spiritStoneLabel,
}: {
  gameStore: ReturnType<typeof useGameStore>;
  worldRegistrySessionLabel: ComputedRef<string>;
  worldRegistrySourceLabel: ComputedRef<string>;
  worldRegistryCounts: ComputedRef<WorldRegistryCounts>;
  currentLocationLabel: ComputedRef<string>;
  spiritStoneLabel: ComputedRef<string>;
}) => {
  const backpackPanelItems = computed(() => {
    const tableRows = gameStore.worldRegistry?.tables?.inventory_items ?? [];
    if (tableRows.length > 0) {
      return tableRows.map((row, index) => {
        const name = asText(row.name, asText(row.item_id, `背包物品 #${index + 1}`));
        const quantity = asText(row.quantity, '');
        const owner = asText(row.owner_character_id, '');
        const effect = asText(row.effect_desc, asText(row.item_type, ''));
        const isSpiritStone = /灵石|spirit\s*stone/i.test(name);
        return {
          id: `bag-reg-${asText(row.item_id, String(index))}`,
          title: quantity ? `${name} x${quantity}` : name,
          description: effect || undefined,
          meta: owner ? `持有者：${owner}` : compactRecord(row, ['name', 'item_id', 'quantity', 'owner_character_id', 'effect_desc']),
          badge: isSpiritStone ? '灵石' : undefined,
          featured: isSpiritStone,
        };
      }).sort((a, b) => Number(b.featured) - Number(a.featured));
    }

    const inventory = gameStore.playerCharacter?.inventory ?? [];
    return inventory
      .map((item, index) => {
        const text = String(item);
        const isSpiritStone = /灵石|spirit\s*stone/i.test(text);
        return {
          id: `bag-${index}`,
          title: text,
          meta: isSpiritStone ? `灵石统计：${spiritStoneLabel.value}` : undefined,
          badge: isSpiritStone ? '灵石' : undefined,
          featured: isSpiritStone,
        };
      })
      .sort((a, b) => Number(b.featured) - Number(a.featured));
  });

  const techniquePanelItems = computed(() => {
    const registryTechniques = gameStore.worldRegistry?.tables?.techniques ?? [];
    if (registryTechniques.length > 0) {
      return registryTechniques.map((row, index) => {
        const title = asText(row.name, asText(row.technique_id, `功法 #${index + 1}`));
        const desc = asText(row.description, asText(row.effect_desc, '暂无描述'));
        const owner = asText(row.owner_character_id, '');
        const required = asText(row.required_realm_level, asText(row.required_realm, ''));
        const metaParts = [
          required ? `需求：${required}` : '',
          owner ? `归属：${owner}` : '',
        ].filter(Boolean);
        const meta = metaParts.length > 0
          ? metaParts.join(' | ')
          : compactRecord(row, ['name', 'technique_id', 'description', 'owner_character_id', 'required_realm_level', 'required_realm']);
        return {
          id: `tech-reg-${asText(row.technique_id, String(index))}`,
          title,
          description: desc,
          meta: meta || undefined,
        };
      });
    }

    const learned = gameStore.playerCharacter?.stats?.techniques ?? [];
    const worldTechniques = gameStore.gameState?.script?.world_setting?.techniques ?? [];
    const worldMap = new Map(worldTechniques.map((tech) => [tech.name, tech]));
    const fromLearned = learned.map((name, index) => {
      const mapped = worldMap.get(name);
      return {
        id: `learned-${index}-${name}`,
        title: name,
        description: mapped?.description || '已掌握功法',
        meta: mapped ? `需求境界：${mapped.required_realm_level}` : '来源：角色面板',
      };
    });
    const learnedSet = new Set(learned);
    const fromWorld = worldTechniques
      .filter((tech) => !learnedSet.has(tech.name))
      .slice(0, 24)
      .map((tech) => ({
        id: `world-tech-${tech.id}`,
        title: tech.name,
        description: tech.description,
        meta: `需求境界：${tech.required_realm_level}`,
      }));
    return [...fromLearned, ...fromWorld];
  });

  const factionPanelItems = computed(() => {
    const registryFactions = gameStore.worldRegistry?.tables?.factions ?? [];
    if (registryFactions.length > 0) {
      return registryFactions.map((row, index) => {
        const title = asText(row.name, asText(row.faction_id, `势力 #${index + 1}`));
        const description = asText(row.description, '');
        const power = asText(row.power_level, asText(row.rank, ''));
        const meta = power
          ? `势力等级：${power}`
          : compactRecord(row, ['name', 'faction_id', 'description', 'power_level', 'rank']);
        return {
          id: `faction-reg-${asText(row.faction_id, String(index))}`,
          title,
          description: description || undefined,
          meta: meta || undefined,
        };
      });
    }

    const scriptFactions = gameStore.gameState?.script?.world_setting?.factions ?? [];
    const worldFactions = Object.values(gameStore.gameState?.world_state?.factions ?? {});
    const merged = [...scriptFactions];
    for (const faction of worldFactions) {
      if (!merged.some((item) => item.id === faction.id)) {
        merged.push(faction);
      }
    }
    return merged.map((faction) => ({
      id: `faction-${faction.id}`,
      title: faction.name,
      description: faction.description,
      meta: `势力等级：${faction.power_level}`,
    }));
  });

  const worldPanelItems = computed(() => {
    const summary = [
      {
        id: 'world-session',
        title: `会话：${worldRegistrySessionLabel.value}`,
        description: `来源：${worldRegistrySourceLabel.value}`,
        featured: true,
      },
      {
        id: 'world-count-characters',
        title: `人物：${worldRegistryCounts.value.characters}`,
        meta: `地图点：${worldRegistryCounts.value.map_nodes}，地图边：${worldRegistryCounts.value.map_edges}`,
      },
      {
        id: 'world-count-assets',
        title: `功法：${worldRegistryCounts.value.techniques}，背包：${worldRegistryCounts.value.inventory_items}`,
        meta: `势力：${worldRegistryCounts.value.factions}，剧情态：${worldRegistryCounts.value.story_state}`,
      },
      {
        id: 'world-count-facts',
        title: `事实：${worldRegistryCounts.value.world_facts}`,
        meta: `当前位置：${currentLocationLabel.value}`,
      },
    ];

    const tables = gameStore.worldRegistry?.tables;
    if (!tables) return summary;

    const mappedTableItems = [
      ...tables.map_nodes.slice(0, 8).map((row, index) => ({
        id: `world-map-${asText(row.location_id, String(index))}`,
        title: `地图：${asText(row.name, asText(row.location_id, `节点 #${index + 1}`))}`,
        description: asText(row.description, ''),
        meta: compactRecord(row, ['name', 'location_id', 'description']) || undefined,
      })),
      ...tables.story_state.slice(0, 6).map((row, index) => ({
        id: `world-story-${index}`,
        title: `剧情：第${asText(row.chapter_index, String(index + 1))}章`,
        description: asText(row.chapter_goal, asText(row.current_arc, '')),
        meta: compactRecord(row, ['chapter_index', 'chapter_goal', 'current_arc']) || undefined,
      })),
      ...tables.world_facts.slice(0, 10).map((row, index) => ({
        id: `world-fact-${asText(row.fact_id, String(index))}`,
        title: `事实：${asText(row.subject, 'unknown')} ${asText(row.predicate, '')}`.trim(),
        description: asText(row.object, ''),
        meta: compactRecord(row, ['fact_id', 'subject', 'predicate', 'object']) || undefined,
      })),
    ];

    return [...summary, ...mappedTableItems];
  });

  const quickPanels = computed(() => ([
    {
      id: 'backpack' as const,
      label: '背包',
      title: '背包',
      subtitle: '当前携带物与可追踪资源',
      emptyText: '背包为空。',
      items: backpackPanelItems.value,
    },
    {
      id: 'techniques' as const,
      label: '功法',
      title: '功法',
      subtitle: '已掌握与世界可见功法',
      emptyText: '尚未获得可展示功法。',
      items: techniquePanelItems.value,
    },
    {
      id: 'factions' as const,
      label: '势力',
      title: '势力',
      subtitle: '世界中的门派与组织',
      emptyText: '暂无势力信息。',
      items: factionPanelItems.value,
    },
    {
      id: 'world' as const,
      label: '世界',
      title: '世界快照',
      subtitle: '本轮世界状态索引',
      emptyText: '暂无世界快照。',
      items: worldPanelItems.value,
    },
  ]));

  return {
    quickPanels,
  };
};
