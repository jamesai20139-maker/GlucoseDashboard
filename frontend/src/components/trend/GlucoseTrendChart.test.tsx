// @vitest-environment happy-dom
/// <reference types="node" />
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { GlucoseTrendChart } from './GlucoseTrendChart';
import type { EventThreshold, GlucoseRecord } from '../../types';

/// 趨勢圖事件節點上色驗證（不使用截圖）。
///
/// 之所以需要這個測試：先前 commit dfb221d 雖然讓節點 className 依事件標準
/// 產出 `point high/normal/low`，但暗色模式的 CSS 特異性 bug
/// （`:root[data-theme="dark"] .point` (0,3,0) 壓過 `.point.high` (0,2,0)）
/// 使紅/綠/黃在暗色模式顯示不出來。此測試分兩層把關：
///   1. 元件 render：每個節點 circle 的 class 必須含正確狀態。
///   2. CSS 回歸：dashboard.css 的暗色模式必須有 .point.high/.normal/.low
///      覆寫，且特異性高於暗色基礎 .point 規則，否則上色會被基礎規則蓋掉。
const THRESHOLDS: EventThreshold[] = [
  { label: '空腹血糖', low: 70, high: 100 },
  { label: '午餐後', low: 70, high: 140 },
];

function record(row: number, event: string, value: number, at: string): GlucoseRecord {
  return { source_row_number: row, measured_at: at, event, glucose_mg_dl: value, remark_1: '', remark_2: '' };
}

/// 空腹血糖標準 70–100、午餐後 70–140。同一數值在不同事件下分類不同，
/// 用以驗證節點顏色「依事件標準」而非單一全段門檻。
const RECORDS: GlucoseRecord[] = [
  record(1, '空腹血糖', 110, '2026-08-01T08:00'), // >=100 → high
  record(2, '空腹血糖', 85, '2026-08-01T12:00'),  // 70..99 → normal
  record(3, '空腹血糖', 65, '2026-08-01T18:00'),  // <70 → low
  record(4, '午餐後', 110, '2026-08-02T13:00'),   // 70..139 → normal（同 110 在空腹為 high）
  record(5, '午餐後', 145, '2026-08-02T19:00'),   // >=140 → high
];

describe('GlucoseTrendChart 事件節點上色', () => {
  it('每個節點 circle 帶有依事件標準算出的 point high/normal/low class', () => {
    const { container } = render(
      <GlucoseTrendChart records={RECORDS} eventThresholds={THRESHOLDS} />,
    );
    const circles = Array.from(container.querySelectorAll('circle.point'));
    // 5 筆紀錄 → 5 個節點
    expect(circles).toHaveLength(5);
    const classes = circles.map((c) => c.getAttribute('class') ?? '');
    // 依 RECORDS 順序對應：high, normal, low, normal, high
    expect(classes[0]).toContain('high');
    expect(classes[1]).toContain('normal');
    expect(classes[2]).toContain('low');
    expect(classes[3]).toContain('normal');
    expect(classes[4]).toContain('high');
  });

  it('同一數值 110 在空腹血糖為 high、在午餐後為 normal（事件別上色）', () => {
    const { container } = render(
      <GlucoseTrendChart records={RECORDS} eventThresholds={THRESHOLDS} />,
    );
    const circles = Array.from(container.querySelectorAll('circle.point'));
    const first = circles[0].getAttribute('class') ?? '';
    const fourth = circles[3].getAttribute('class') ?? '';
    expect(first).toContain('high');
    expect(fourth).toContain('normal');
  });

  it('無對應事件閾值時節點為 normal（不誤判為 high/low）', () => {
    const records = [record(1, '未知事件', 300, '2026-08-01T08:00')];
    const { container } = render(
      <GlucoseTrendChart records={records} eventThresholds={THRESHOLDS} />,
    );
    const circle = container.querySelector('circle.point')!;
    const cls = circle.getAttribute('class') ?? '';
    expect(cls).toContain('normal');
    expect(cls).not.toContain('high');
    expect(cls).not.toContain('low');
  });
});

/// 解析選擇器的特異性 (a,b,c)。
/// 僅處理本檔案用到的選擇器片段（類、屬性、偽類、型別），足夠此回歸測試使用。
function specificity(selector: string): [number, number, number] {
  let a = 0, b = 0, c = 0;
  // ID
  const ids = selector.match(/#[\w-]+/g) ?? [];
  a += ids.length;
  // 類、屬性、偽類（:root）各 +1b
  const classes = selector.match(/\.[\w-]+/g) ?? [];
  const attrs = selector.match(/\[[^\]]+\]/g) ?? [];
  const pseudos = selector.match(/:(?!root)[\w-]+/g) ?? [];
  const rootPseudo = selector.includes(':root') ? 1 : 0;
  b += classes.length + attrs.length + pseudos.length + rootPseudo;
  // 型別選擇器
  const types = selector.match(/(^|\s|>|\+|~)[a-z][\w-]*/gi) ?? [];
  c += types.length;
  return [a, b, c];
}

function compare(a: [number, number, number], b: [number, number, number]): number {
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return 0;
}

describe('趨勢圖節點 CSS 特異性回歸（暗色模式不可蓋掉事件上色）', () => {
  const here = dirname(fileURLToPath(import.meta.url));
  const dashboardCss = readFileSync(resolve(here, '../../styles/dashboard.css'), 'utf8');

  /// 從 CSS 文字中抓出符合選擇器片段的規則，回傳其特異性；找不到時回 null。
  function specOf(selectorFragment: string): [number, number, number] | null {
    // 先移除註解，避免註解內的文字干擾選擇器比對。
    const cleaned = dashboardCss.replace(/\/\*[\s\S]*?\*\//g, '');
    // 逐筆配對「選擇器 { 宣告 }」。選擇器段不含 { }，宣告段不含巢狀 { }。
    const re = /([^{}]+)\{([^{}]*)\}/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(cleaned)) !== null) {
      const selectors = m[1].split(',').map((s) => s.trim());
      for (const s of selectors) {
        if (s === selectorFragment) return specificity(s);
      }
    }
    return null;
  }

  it('暗色模式有 .point.high 覆寫且特異性高於暗色基礎 .point 規則', () => {
    const base = specOf(':root[data-theme="dark"] .point');
    const high = specOf(':root[data-theme="dark"] .point.high');
    expect(base).not.toBeNull();
    expect(high).not.toBeNull();
    expect(compare(high!, base!)).toBeGreaterThan(0);
  });

  it('暗色模式有 .point.normal 覆寫且特異性高於暗色基礎 .point 規則', () => {
    const base = specOf(':root[data-theme="dark"] .point');
    const normal = specOf(':root[data-theme="dark"] .point.normal');
    expect(normal).not.toBeNull();
    expect(compare(normal!, base!)).toBeGreaterThan(0);
  });

  it('暗色模式有 .point.low 覆寫且特異性高於暗色基礎 .point 規則', () => {
    const base = specOf(':root[data-theme="dark"] .point');
    const low = specOf(':root[data-theme="dark"] .point.low');
    expect(low).not.toBeNull();
    expect(compare(low!, base!)).toBeGreaterThan(0);
  });
});