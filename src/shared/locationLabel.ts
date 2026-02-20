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

const normalizeLocationId = (raw: string): string =>
  raw.trim().toLowerCase().replace(/-/g, '_');

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
    return normalized
      .split('_')
      .filter((part) => part.length > 0)
      .map((part) => part[0].toUpperCase() + part.slice(1))
      .join(' / ');
  }

  return raw;
};
