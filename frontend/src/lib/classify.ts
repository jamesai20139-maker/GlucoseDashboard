import type { EventThreshold } from '../types';

/// 依事件標準範圍分類血糖值，供趨勢圖與表格上色共用。
/// 規則：`value >= high` → 'high'（超過，紅）；`value < low` → 'low'（過低，黃）；
/// 其餘 → 'normal'（範圍內，綠）。找不到對應事件的閾值時視為 'normal'。
export function classifyByThreshold(
  event: string,
  value: number,
  thresholds: EventThreshold[],
): 'high' | 'normal' | 'low' {
  const threshold = thresholds.find((t) => t.label === event);
  if (!threshold) return 'normal';
  if (value >= threshold.high) return 'high';
  if (value < threshold.low) return 'low';
  return 'normal';
}