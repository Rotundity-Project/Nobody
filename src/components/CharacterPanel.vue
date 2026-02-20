<template>
  <div class="panel-surface max-h-[70vh] overflow-y-auto rounded-2xl p-5">
    <h3 class="mb-4 text-xl font-display text-amber-200">角色信息</h3>

    <div v-if="character" class="space-y-4">
      <section class="border-b border-slate-700 pb-4">
        <p class="text-sm text-slate-400">姓名</p>
        <p class="text-lg font-medium text-white">{{ character.name }}</p>
      </section>

      <section>
        <p class="text-sm text-slate-400">修为境界</p>
        <p class="font-medium text-white">{{ realmLabel }}</p>
        <p class="text-xs text-slate-500">
          等级 {{ character.stats.cultivation_realm.level }}.{{ character.stats.cultivation_realm.sub_level }}
        </p>
      </section>

      <section>
        <p class="text-sm text-slate-400">灵根</p>
        <div class="flex items-center gap-2">
          <span class="font-medium text-white">{{ elementLabel }}</span>
          <span class="rounded px-2 py-0.5 text-xs font-medium" :class="rootGradeClass(character.stats.spiritual_root.grade)">
            {{ gradeLabel }}
          </span>
        </div>
        <p class="text-xs text-slate-500">亲和度 {{ affinityLabel }}</p>
        <p class="text-xs text-slate-500">天赋提示：{{ gradeHint }}</p>
      </section>

      <section>
        <p class="text-sm text-slate-400">寿元</p>
        <p class="font-medium text-white">
          {{ character.stats.lifespan.current_age }} / {{ character.stats.lifespan.max_age }}
        </p>
        <div class="mt-1 h-2 w-full rounded-full bg-slate-700">
          <div
            class="h-2 rounded-full transition-all duration-300"
            :class="lifespanBarClass(character.stats.lifespan)"
            :style="{ width: `${lifespanPercent(character.stats.lifespan)}%` }"
          />
        </div>
      </section>

      <section>
        <p class="text-sm text-slate-400">战斗力</p>
        <p class="font-medium text-white">{{ character.stats.combat_power.toLocaleString() }}</p>
      </section>

      <section v-if="character.combat_status">
        <p class="text-sm text-slate-400">战后状态</p>
        <p class="text-sm text-white">
          伤势 {{ character.combat_status.injury_level }} /
          声望 {{ character.combat_status.reputation }} /
          仇恨 {{ character.combat_status.enmity }} /
          气机紊乱 {{ character.combat_status.qi_deviation ?? 0 }}
        </p>
      </section>

      <section v-if="socialProfileItems.length > 0">
        <p class="text-sm text-slate-400">关系画像</p>
        <div class="grid grid-cols-2 gap-2 text-xs text-slate-200">
          <div
            v-for="item in socialProfileItems"
            :key="item.label"
            class="rounded border border-slate-700 bg-slate-900/40 p-2"
          >
            <p class="text-slate-400">{{ item.label }}</p>
            <p class="mt-0.5">{{ item.value }}</p>
          </div>
        </div>
      </section>

      <section v-if="personalityTags.length > 0">
        <p class="text-sm text-slate-400">人格标签</p>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="tag in personalityTags"
            :key="tag"
            class="rounded bg-indigo-700/40 px-2 py-0.5 text-xs text-indigo-100"
          >
            {{ tag }}
          </span>
        </div>
      </section>

      <section v-if="techniqueGroups.length > 0">
        <p class="text-sm text-slate-400">功法流派</p>
        <div class="space-y-2">
          <div
            v-for="group in techniqueGroups"
            :key="group.style"
            class="rounded border border-slate-700 bg-slate-900/40 p-2"
          >
            <p class="text-xs text-slate-300">{{ group.style }}（{{ group.items.length }}）</p>
            <p class="text-xs text-slate-200">{{ group.items.join(' / ') }}</p>
          </div>
        </div>
      </section>

      <section v-if="recentGrowthLog.length > 0">
        <p class="text-sm text-slate-400">最近成长记录</p>
        <ul class="space-y-1 text-xs text-slate-300">
          <li v-for="(entry, index) in recentGrowthLog" :key="`${index}-${entry}`">
            {{ entry }}
          </li>
        </ul>
      </section>

      <section>
        <p class="text-sm text-slate-400">位置</p>
        <p class="font-medium text-white">{{ locationLabel }}</p>
      </section>
    </div>

    <div v-else class="text-center text-slate-400">
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

const elementLabel = computed(() => {
  if (!props.character) return '';
  const mapping: Record<Element, string> = {
    [Element.Fire]: '火灵根',
    [Element.Water]: '水灵根',
    [Element.Wood]: '木灵根',
    [Element.Metal]: '金灵根',
    [Element.Earth]: '土灵根',
  };
  const root = props.character.stats.spiritual_root;
  const elements = root.elements && root.elements.length > 0 ? root.elements : [root.element];
  return elements.map((item) => mapping[item] ?? '未知灵根').join(' / ');
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
      return 'bg-amber-600 text-white';
    case Grade.Double:
      return 'bg-emerald-600 text-white';
    case Grade.Triple:
      return 'bg-sky-600 text-white';
    case Grade.Pseudo:
      return 'bg-slate-600 text-white';
    default:
      return 'bg-slate-600 text-white';
  }
};

const lifespanPercent = (lifespan: Lifespan): number => {
  if (lifespan.max_age <= 0) return 0;
  return Math.min(100, (lifespan.current_age / lifespan.max_age) * 100);
};

const lifespanBarClass = (lifespan: Lifespan): string => {
  const percentage = lifespanPercent(lifespan);
  if (percentage < 30) return 'bg-emerald-500';
  if (percentage < 70) return 'bg-amber-500';
  return 'bg-rose-500';
};
</script>
