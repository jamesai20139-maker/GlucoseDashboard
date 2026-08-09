import { describe, it, expect } from 'vitest';
import { classifyByThreshold } from './classify';
import type { EventThreshold } from '../types';

/// 趨勢圖事件節點與血糖紀錄表格共用同一個 classifyByThreshold。
/// 此測試確保「節點上色」與「紀錄上色」邏輯一致：
/// 超過上限 → 'high'（紅）、低於下限 → 'low'（黃）、其餘 → 'normal'（綠）。
/// 不使用截圖,純邏輯驗證。
const THRESHOLDS: EventThreshold[] = [
  { label: '空腹血糖', low: 70, high: 100 },
  { label: '午餐後', low: 70, high: 140 },
];

describe('classifyByThreshold', () => {
  it('低於下限分類為 low', () => {
    expect(classifyByThreshold('空腹血糖', 65, THRESHOLDS)).toBe('low');
  });

  it('上限臨界值分類為 high（>= 即偏高）', () => {
    expect(classifyByThreshold('空腹血糖', 100, THRESHOLDS)).toBe('high');
  });

  it('範圍內分類為 normal', () => {
    expect(classifyByThreshold('空腹血糖', 85, THRESHOLDS)).toBe('normal');
  });

  it('不同事件套用各自標準', () => {
    // 同一數值 100：空腹血糖為偏高、午餐後為正常 → 事件節點顏色依事件不同而不同。
    expect(classifyByThreshold('空腹血糖', 100, THRESHOLDS)).toBe('high');
    expect(classifyByThreshold('午餐後', 100, THRESHOLDS)).toBe('normal');
  });

  it('無對應事件閾值時視為 normal', () => {
    expect(classifyByThreshold('未知事件', 200, THRESHOLDS)).toBe('normal');
  });

  it('下限臨界值仍為 normal（< low 才偏低）', () => {
    expect(classifyByThreshold('空腹血糖', 70, THRESHOLDS)).toBe('normal');
    expect(classifyByThreshold('空腹血糖', 69, THRESHOLDS)).toBe('low');
  });
});