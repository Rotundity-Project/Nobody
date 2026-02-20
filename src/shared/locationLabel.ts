export interface LocationLabelSource {
  id: string;
  name: string;
}

const FALLBACK_LOCATION_LABELS: Record<string, string> = {
  sect_valley: '宗门外谷',
  sect: '宗门',
  town: '城镇',
  city: '城镇',
  forest: '林地',
  stone_forest: '乱石林',
  mountain: '山脉',
};

const LOCATION_TOKEN_LABELS: Record<string, string> = {
  sect: '宗门',
  valley: '山谷',
  peak: '山峰',
  mountain: '山脉',
  cave: '洞窟',
  forest: '林地',
  stone: '石',
  river: '河',
  lake: '湖',
  town: '城镇',
  city: '城池',
  market: '市集',
  plains: '平原',
};

const normalizeLocationId = (raw: string): string =>
  raw.trim().toLowerCase().replace(/-/g, '_');

const isChineseToken = (value: string): boolean => /[\u4e00-\u9fff]/.test(value);

export const buildLocationLabelMap = (
  sources: LocationLabelSource[] = [],
): Map<string, string> => {
  const map = new Map<string, string>();
  for (const item of sources) {
    if (item.id && item.name) {
      map.set(normalizeLocationId(item.id), item.name);
    }
  }
  return map;
};

export const formatLocationLabel = (
  raw?: string | null,
  labelMap?: Map<string, string>,
): string => {
  if (!raw) {
    return '未知';
  }

  const normalized = normalizeLocationId(raw);
  const mapped = labelMap?.get(normalized);
  if (mapped) {
    return mapped;
  }

  const fallback = FALLBACK_LOCATION_LABELS[normalized];
  if (fallback) {
    return fallback;
  }

  if (normalized.includes('_')) {
    const parts = normalized
      .split('_')
      .filter((part) => part.length > 0);
    const translated = parts.map((part) => LOCATION_TOKEN_LABELS[part] ?? part);
    if (translated.some((part, idx) => part !== parts[idx])) {
      const allTranslated = translated.every((part, idx) => part !== parts[idx]);
      if (allTranslated && translated.every(isChineseToken)) {
        return translated.join('');
      }
      return translated.join(' / ');
    }
    return parts
      .map((part) => part[0].toUpperCase() + part.slice(1))
      .join(' / ');
  }

  return raw;
};
