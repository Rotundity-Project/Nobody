<template>
  <div class="ink-character-panel max-h-[70vh] overflow-y-auto rounded-2xl p-5">
    <h3 class="mb-4 text-xl font-display text-[var(--ink-title-color)]">
      角色信息
    </h3>

    <div
      v-if="character"
      class="space-y-4"
    >
      <section class="char-divider border-b pb-4">
        <p class="char-label text-sm">
          姓名
        </p>
        <p class="char-value text-lg font-medium">
          {{ character.name }}
        </p>
      </section>

      <section>
        <p class="char-label text-sm">
          修为境界
        </p>
        <p class="char-value font-medium">
          {{ realmLabel }}
        </p>
        <p class="char-meta text-xs">
          等级 {{ character.stats.cultivation_realm.level }}.{{ character.stats.cultivation_realm.sub_level }}
        </p>
      </section>

      <section>
        <p class="char-label text-sm">
          灵根
        </p>
        <div class="root-row mt-1">
          <div
            v-for="item in rootElements"
            :key="item.element"
            class="root-item"
          >
            <span
              class="root-icon"
              :class="item.colorClass"
              aria-hidden="true"
            >
              <svg
                v-if="item.element === Element.Earth"
                viewBox="0 0 24 24"
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M3 18h18L16 8h-8L3 18Z"
                />
              </svg>
              <svg
                v-else-if="item.element === Element.Metal"
                viewBox="0 0 24 24"
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
              >
                <circle
                  cx="12"
                  cy="12"
                  r="6.5"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 5.5v13M5.5 12h13"
                />
              </svg>
              <svg
                v-else-if="item.element === Element.Wood"
                viewBox="0 0 24 24"
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 20V8"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 10c3.5 0 5-2 5-4-3 0-5 1.8-5 4Z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 13c-3.5 0-5-2-5-4 3 0 5 1.8 5 4Z"
                />
              </svg>
              <svg
                v-else-if="item.element === Element.Water"
                viewBox="0 0 24 24"
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 4c3.6 4.2 5.5 7 5.5 9.5A5.5 5.5 0 0 1 12 19a5.5 5.5 0 0 1-5.5-5.5C6.5 11 8.4 8.2 12 4Z"
                />
              </svg>
              <svg
                v-else
                viewBox="0 0 24 24"
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 4c2.5 2 4.5 4.2 4.5 6.8 0 3.4-2.5 5.8-4.5 9.2-2-3.4-4.5-5.8-4.5-9.2C7.5 8.2 9.5 6 12 4Z"
                />
              </svg>
            </span>
            <span
              class="root-name"
              :class="item.colorClass"
            >{{ item.label }}</span>
          </div>
          <span class="char-subtle text-sm">{{ rootTypeLabel }}</span>
          <span
            class="grade-badge rounded px-2 py-0.5 text-xs font-medium"
            :class="rootGradeClass(character.stats.spiritual_root.grade)"
          >
            {{ gradeLabel }}
          </span>
        </div>
        <p class="char-meta text-xs">
          亲和度 {{ affinityLabel }}
        </p>
        <p class="char-meta text-xs">
          天赋提示：{{ gradeHint }}
        </p>
      </section>

      <section>
        <p class="char-label text-sm">
          寿元
        </p>
        <p class="char-value font-medium">
          {{ character.stats.lifespan.current_age }} / {{ character.stats.lifespan.max_age }}
        </p>
        <div class="char-lifespan-track mt-1 h-2 w-full rounded-full">
          <div
            class="h-2 rounded-full transition-all duration-300"
            :class="lifespanBarClass(character.stats.lifespan)"
            :style="{ width: `${lifespanPercent(character.stats.lifespan)}%` }"
          />
        </div>
      </section>

      <section>
        <p class="char-label text-sm">
          战斗力
        </p>
        <p class="char-value font-medium">
          {{ character.stats.combat_power.toLocaleString() }}
        </p>
      </section>

      <section v-if="character.combat_status">
        <p class="char-label text-sm">
          战后状态
        </p>
        <p class="char-value text-sm">
          伤势 {{ character.combat_status.injury_level }} /
          声望 {{ character.combat_status.reputation }} /
          仇恨 {{ character.combat_status.enmity }} /
          气机紊乱 {{ character.combat_status.qi_deviation ?? 0 }}
        </p>
      </section>

      <section v-if="socialProfileItems.length > 0">
        <p class="char-label text-sm">
          关系画像
        </p>
        <div class="char-subtle grid grid-cols-2 gap-2 text-xs">
          <div
            v-for="item in socialProfileItems"
            :key="item.label"
            class="char-info-card rounded border p-2"
          >
            <p class="char-label">
              {{ item.label }}
            </p>
            <p class="mt-0.5">
              {{ item.value }}
            </p>
          </div>
        </div>
      </section>

      <section v-if="personalityTags.length > 0">
        <p class="char-label text-sm">
          人格标签
        </p>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="tag in personalityTags"
            :key="tag"
            class="char-personality-tag rounded px-2 py-0.5 text-xs"
          >
            {{ tag }}
          </span>
        </div>
      </section>

      <section v-if="techniqueGroups.length > 0">
        <p class="char-label text-sm">
          功法流派
        </p>
        <div class="space-y-2">
          <div
            v-for="group in techniqueGroups"
            :key="group.style"
            class="char-info-card rounded border p-2"
          >
            <p class="char-meta text-xs">
              {{ group.style }}（{{ group.items.length }}）
            </p>
            <p class="char-subtle text-xs">
              {{ group.items.join(' / ') }}
            </p>
          </div>
        </div>
      </section>

      <section v-if="recentGrowthLog.length > 0">
        <p class="char-label text-sm">
          最近成长记录
        </p>
        <ul class="char-meta space-y-1 text-xs">
          <li
            v-for="(entry, index) in recentGrowthLog"
            :key="`${index}-${entry}`"
          >
            {{ entry }}
          </li>
        </ul>
      </section>

      <section>
        <p class="char-label text-sm">
          位置
        </p>
        <p class="char-value font-medium">
          {{ locationLabel }}
        </p>
      </section>
    </div>

    <div
      v-else
      class="char-label text-center"
    >
      <p>暂无角色数据</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Character, Lifespan, SocialProfile } from '../types/game';
import { Element, Grade } from '../types/game';
import { formatLocationLabel } from '../shared/locationLabel';

const props = defineProps<{
  character: Character | null;
}>();

const rootLabelMap: Record<Element, string> = {
  [Element.Fire]: '火',
  [Element.Water]: '水',
  [Element.Wood]: '木',
  [Element.Metal]: '金',
  [Element.Earth]: '土',
};

const rootColorClassMap: Record<Element, string> = {
  [Element.Fire]: 'root-fire',
  [Element.Water]: 'root-water',
  [Element.Wood]: 'root-wood',
  [Element.Metal]: 'root-metal',
  [Element.Earth]: 'root-earth',
};

const rootElements = computed(() => {
  if (!props.character) return [];
  const root = props.character.stats.spiritual_root;
  const elements = root.elements && root.elements.length > 0 ? root.elements : [root.element];
  return elements.map((item) => ({
    element: item,
    label: rootLabelMap[item] ?? '未知',
    colorClass: rootColorClassMap[item] ?? '',
  }));
});

const rootTypeLabel = computed(() => {
  const count = rootElements.value.length;
  if (count <= 1) return '灵根';
  if (count === 2) return '双灵根';
  if (count === 3) return '三灵根';
  return '杂灵根';
});

const gradeLabel = computed(() => {
  if (!props.character) return '';
  const mapping: Record<Grade, string> = {
    [Grade.Heavenly]: '单灵根',
    [Grade.Double]: '双灵根',
    [Grade.Triple]: '三灵根',
    [Grade.Pseudo]: '杂灵根',
  };
  return mapping[props.character.stats.spiritual_root.grade] ?? '未知';
});

const gradeHint = computed(() => {
  if (!props.character) return '';
  switch (props.character.stats.spiritual_root.grade) {
    case Grade.Heavenly:
      return '修炼效率极高，适合冲击高阶境界';
    case Grade.Double:
      return '资质良好，修炼与战斗成长较平衡';
    case Grade.Triple:
      return '中等资质，适合稳扎稳打路线';
    case Grade.Pseudo:
      return '基础资质一般，需要更多机缘支撑';
    default:
      return '暂无评估';
  }
});

const affinityLabel = computed(() => {
  if (!props.character) return '';
  const affinity = props.character.stats.spiritual_root.affinity;
  const pct = affinity <= 1 ? Math.round(affinity * 100) : Math.round(affinity);
  return `${pct}%`;
});

const realmLabel = computed(() => {
  if (!props.character) return '';
  const raw = props.character.stats.cultivation_realm.name;
  const mapping: Record<string, string> = {
    'Qi Condensation': '炼气',
    'Foundation Establishment': '筑基',
    'Golden Core': '金丹',
    'Nascent Soul': '元婴',
  };
  return mapping[raw] ?? raw;
});

const locationLabel = computed(() => {
  if (!props.character) return '';
  return formatLocationLabel(props.character.location);
});

const recentGrowthLog = computed(() => {
  const list = props.character?.growth_log ?? [];
  return list.slice(-6).reverse();
});

const personalityTags = computed(() => {
  const list = props.character?.personality_tags ?? [];
  return list.slice(0, 6);
});

const classifyTechniqueStyle = (name: string): string => {
  const lower = name.toLowerCase();
  if (lower.includes('剑') || lower.includes('sword')) return '剑修';
  if (lower.includes('刀') || lower.includes('blade')) return '刀修';
  if (lower.includes('拳') || lower.includes('体') || lower.includes('body')) return '体修';
  if (lower.includes('符') || lower.includes('阵') || lower.includes('array') || lower.includes('talisman')) return '符阵';
  if (lower.includes('火') || lower.includes('雷') || lower.includes('water') || lower.includes('冰')) return '术法';
  return '杂学';
};

const techniqueGroups = computed(() => {
  const techniques = props.character?.stats.techniques ?? [];
  if (techniques.length === 0) return [];
  const grouped = new Map<string, string[]>();
  for (const tech of techniques) {
    const style = classifyTechniqueStyle(tech);
    const list = grouped.get(style) ?? [];
    list.push(tech);
    grouped.set(style, list);
  }
  return Array.from(grouped.entries()).map(([style, items]) => ({ style, items }));
});

const socialProfileItems = computed(() => {
  const profile: SocialProfile | undefined = props.character?.social_profile;
  if (!profile) return [];
  return [
    { label: '宗门亲和', value: profile.sect_affinity },
    { label: '师徒羁绊', value: profile.mentor_bond },
    { label: '宿怨值', value: profile.vendetta },
    { label: '人情值', value: profile.favor },
    { label: '阵营立场', value: profile.camp_stance },
  ].map((item) => ({ label: item.label, value: String(item.value) }));
});

const rootGradeClass = (grade: Grade): string => {
  switch (grade) {
    case Grade.Heavenly:
      return 'grade-badge-heavenly';
    case Grade.Double:
      return 'grade-badge-double';
    case Grade.Triple:
      return 'grade-badge-triple';
    case Grade.Pseudo:
      return 'grade-badge-pseudo';
    default:
      return 'grade-badge-pseudo';
  }
};

const lifespanPercent = (lifespan: Lifespan): number => {
  if (lifespan.max_age <= 0) return 0;
  return Math.min(100, (lifespan.current_age / lifespan.max_age) * 100);
};

const lifespanBarClass = (lifespan: Lifespan): string => {
  const percentage = lifespanPercent(lifespan);
  if (percentage < 30) return 'lifespan-bar-safe';
  if (percentage < 70) return 'lifespan-bar-mid';
  return 'lifespan-bar-risk';
};
</script>

<style scoped>
.ink-character-panel {
  background: var(--ink-card-bg);
  border: 1px solid var(--ink-border-soft);
  box-shadow: var(--ink-shadow-panel);
}

.char-label {
  color: var(--ink-text-muted);
}

.char-value {
  color: var(--ink-text-primary);
}

.char-meta {
  color: var(--ink-text-muted);
}

.char-subtle {
  color: var(--character-text-secondary);
}

.char-divider {
  border-color: var(--ink-border-soft);
}

.char-lifespan-track {
  background: var(--ink-card-bg-muted);
}

.char-info-card {
  border-color: var(--ink-border-soft);
  background: var(--ink-paper-elevated);
}

.grade-badge {
  color: var(--ink-text-primary);
}

.grade-badge-heavenly {
  background: var(--character-grade-heavenly-bg);
}

.grade-badge-double {
  background: var(--character-grade-double-bg);
}

.grade-badge-triple {
  background: var(--character-grade-triple-bg);
}

.grade-badge-pseudo {
  background: var(--character-grade-pseudo-bg);
}

.char-personality-tag {
  border: 1px solid var(--character-tag-border);
  background: var(--character-tag-bg);
  color: var(--character-tag-text);
}

.lifespan-bar-safe {
  background: var(--character-lifespan-safe-bg);
}

.lifespan-bar-mid {
  background: var(--character-lifespan-mid-bg);
}

.lifespan-bar-risk {
  background: var(--character-lifespan-risk-bg);
}

.root-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.root-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.root-icon {
  display: inline-flex;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
}

.root-name {
  font-size: 14px;
  line-height: 1;
}

.root-earth {
  color: var(--runtime-root-earth);
}

.root-metal {
  color: var(--ink-title-color);
}

.root-wood {
  color: var(--ink-text-ink);
}

.root-water {
  color: var(--runtime-root-water);
}

.root-fire {
  color: var(--ink-accent-main);
}
</style>
