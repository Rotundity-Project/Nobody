import { computed } from 'vue';
import { type useGameStore } from '../stores/gameStore';
import { buildLocationLabelMap, formatLocationLabel } from '../shared/locationLabel';

type RootElement = 'Fire' | 'Water' | 'Wood' | 'Metal' | 'Earth';

const realmStageLabel = (subLevel?: number): string => {
  if (!subLevel || subLevel <= 1) return '初期';
  if (subLevel === 2) return '中期';
  if (subLevel === 3) return '后期';
  return '圆满';
};

const rootElementNameMap: Record<RootElement, string> = {
  Fire: '火',
  Water: '水',
  Wood: '木',
  Metal: '金',
  Earth: '土',
};

const rootElementClassMap: Record<RootElement, string> = {
  Fire: 'runtime-root-fire',
  Water: 'runtime-root-water',
  Wood: 'runtime-root-wood',
  Metal: 'runtime-root-metal',
  Earth: 'runtime-root-earth',
};

export const useRuntimeStatusMetrics = ({
  gameStore,
}: {
  gameStore: ReturnType<typeof useGameStore>;
}) => {
  const currentChapterTitle = computed(
    () => gameStore.plotState?.current_chapter?.title || gameStore.currentScene?.name || '第一章',
  );

  const playerRealmLabel = computed(() => {
    const realm = gameStore.playerCharacter?.stats?.cultivation_realm;
    if (!realm) {
      return '凡人';
    }
    return `${realm.name}${realmStageLabel(realm.sub_level)}（${realm.level}-${realm.sub_level}）`;
  });

  const chapterProgressLabel = computed(() => {
    const chapter = gameStore.plotState?.current_chapter;
    if (!chapter) {
      return '0 / 无';
    }
    return `${chapter.index} / ${chapter.title || '未命名章节'}`;
  });

  const chapterIndexLabel = computed(() => {
    const chapter = gameStore.plotState?.current_chapter;
    const idx = chapter?.index ?? 1;
    const cn = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
    const suffix = idx >= 0 && idx < cn.length ? cn[idx] : String(idx);
    return `第${suffix}章`;
  });

  const chapterNameLabel = computed(() => {
    const raw = currentChapterTitle.value.trim();
    const chapterOnlyPattern = /^第[零一二三四五六七八九十百千万\d]+章$/u;
    const stripped = raw
      .replace(/^第[零一二三四五六七八九十百千万\d]+章[\s·、:：-]*/u, '')
      .trim();
    if (stripped.length > 0 && !chapterOnlyPattern.test(stripped)) {
      return stripped;
    }
    const sceneName = (gameStore.currentScene?.name ?? '').trim();
    if (sceneName.length > 0 && !chapterOnlyPattern.test(sceneName)) {
      return sceneName;
    }
    return '未命名章节';
  });

  const locationNameMap = computed(() =>
    buildLocationLabelMap(
      (gameStore.gameState?.script?.world_setting?.locations ?? []).map((loc) => ({
        id: loc.id,
        name: loc.name,
      })),
    ),
  );

  const currentLocationLabel = computed(() =>
    formatLocationLabel(
      gameStore.playerCharacter?.location || gameStore.currentScene?.location,
      locationNameMap.value,
    ),
  );

  const sceneHeadlineLabel = computed(() => {
    const chapterName = chapterNameLabel.value.trim();
    if (chapterName.length > 0 && !/^第[零一二三四五六七八九十百千万\d]+章$/u.test(chapterName)) {
      return chapterName;
    }
    const location = currentLocationLabel.value.trim();
    return location.length > 0 ? location : '无名之地';
  });

  const showSceneGlyph = computed(() => /[堂殿阁楼宫门院府台塔]/u.test(sceneHeadlineLabel.value));

  const chapterInteractionLabel = computed(() => {
    const chapter = gameStore.plotState?.current_chapter;
    if (!chapter) {
      return '0 / 0-0';
    }
    const min = gameStore.plotState?.settings?.min_interactions_per_chapter ?? 0;
    const max = gameStore.plotState?.settings?.max_interactions_per_chapter ?? 0;
    return `${chapter.interaction_count} / ${min}-${max}`;
  });

  const gameTimeLabel = computed(() => {
    const time = gameStore.gameState?.game_time;
    if (!time) {
      return '第0年 · 第0月 · 第0日';
    }
    return `第${time.year}年 · 第${time.month}月 · 第${time.day}日`;
  });

  const spiritStoneLabel = computed(() => {
    const inventory = gameStore.playerCharacter?.inventory ?? [];
    if (inventory.length === 0) {
      return '0';
    }
    const parsed = inventory.reduce((sum, item) => {
      const text = String(item ?? '');
      if (!/灵石|spirit\s*stone/i.test(text)) {
        return sum;
      }
      const num = text.match(/(\d+)/);
      return sum + (num ? Number(num[1]) : 1);
    }, 0);
    if (parsed > 0) {
      return parsed.toLocaleString();
    }
    return inventory.length.toLocaleString();
  });

  const characterCreationDurationLabel = computed(() => {
    const ms = gameStore.lastInitializationDurationMs;
    if (typeof ms !== 'number' || !Number.isFinite(ms) || ms <= 0) {
      return '';
    }
    const secs = ms / 1000;
    return `${secs.toFixed(2)}s`;
  });

  const playerRootLabel = computed(() => {
    const root = gameStore.playerCharacter?.stats?.spiritual_root;
    if (!root) {
      return '灵根未显';
    }
    const elements = (root.elements?.length ? root.elements : [root.element]).map((value) => String(value));
    const mapped = elements.map((value) => rootElementNameMap[value as RootElement] ?? value);
    if (mapped.length === 1) {
      return `${mapped[0]}灵根`;
    }
    if (mapped.length === 2) {
      return `${mapped.join('')}双灵根`;
    }
    if (mapped.length === 3) {
      return `${mapped.join('')}三灵根`;
    }
    return `${mapped.join('/')}杂灵根`;
  });

  const playerRootElements = computed(() => {
    const root = gameStore.playerCharacter?.stats?.spiritual_root;
    if (!root) return [];
    const values = (root.elements?.length ? root.elements : [root.element])
      .map((value) => String(value))
      .filter((value): value is RootElement =>
        value === 'Fire' || value === 'Water' || value === 'Wood' || value === 'Metal' || value === 'Earth');

    return values.map((element) => ({
      element,
      label: rootElementNameMap[element],
      colorClass: rootElementClassMap[element],
    }));
  });

  const playerRootTypeLabel = computed(() => {
    const count = playerRootElements.value.length;
    if (count <= 1) return '灵根';
    if (count === 2) return '双灵根';
    if (count === 3) return '三灵根';
    return '杂灵根';
  });

  const worldLocationList = computed(() =>
    (gameStore.gameState?.script?.world_setting?.locations ?? []).map((loc) => ({
      id: loc.id,
      name: loc.name,
      spiritual_energy: loc.spiritual_energy,
    })),
  );

  const recentCombatReview = computed(() => {
    const events = gameStore.gameState?.event_history ?? [];
    return events
      .filter((event) =>
        event.event_type === 'combat_explanation'
        || event.event_type === 'encounter'
        || event.event_type === 'combat')
      .slice(-6)
      .reverse()
      .map((event) => `[t=${event.timestamp}] ${event.description}`);
  });

  return {
    currentChapterTitle,
    chapterProgressLabel,
    chapterIndexLabel,
    chapterNameLabel,
    sceneHeadlineLabel,
    showSceneGlyph,
    chapterInteractionLabel,
    gameTimeLabel,
    spiritStoneLabel,
    characterCreationDurationLabel,
    playerRealmLabel,
    playerRootLabel,
    playerRootElements,
    playerRootTypeLabel,
    currentLocationLabel,
    worldLocationList,
    recentCombatReview,
  };
};
