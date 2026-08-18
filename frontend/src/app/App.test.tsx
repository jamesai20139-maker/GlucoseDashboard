// @vitest-environment happy-dom
/// <reference types="node" />
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import App from './App';

/// App 設定 modal 唯讀化與工作表名欄位驗證。
///
/// 背景：自 schema 4 起「事件關鍵字設定」與「血糖標準值」改由 Google Sheet
/// 兩個工作表即時衍生，不再於本機編輯。此測試確保：
///   1. Google Sheet 設定分頁含「事件關鍵字工作表名稱」「血糖標準值工作表名稱」
///      兩欄位且預設值正確。
///   2. 事件關鍵字設定分頁為唯讀（無新增表單/刪除按鈕）。
///   3. 血糖標準值分頁為唯讀（無 number input/儲存鈕）。

const CONFIG_STATUS = {
  configured: true,
  credential_store: 'stub',
  schema_version: 4,
  sheet_id: 'FAKE',
  sheet_gid: '0',
  sheet_name: 'Sheet1',
  fixture_path: null,
  last_successful_sync_at: null,
  event_keywords_sheet_name: '事件關鍵字設定',
  glucose_standards_sheet_name: '血糖標準值設定',
  custom_events: [{ label: '飲食測試', low_threshold: 70, high_threshold: 139 }],
  event_thresholds: [
    { label: '空腹血糖', low: 70, high: 100 },
    { label: '午餐前', low: 70, high: 101 },
    { label: '午餐後', low: 70, high: 140 },
    { label: '晚餐前', low: 70, high: 101 },
    { label: '晚餐後', low: 70, high: 140 },
    { label: '睡前', low: 70, high: 140 },
    { label: '飲食測試', low: 70, high: 139 },
  ],
};

const DASHBOARD = {
  summary: {
    record_count: 1, average_mg_dl: 88, minimum_mg_dl: 88, maximum_mg_dl: 88,
    estimated_hba1c_percent: 4.7, estimated_average_glucose_mg_dl: 88,
    in_reference_percent: 100, low_percent: 0, high_percent: 0,
  },
  records: [{ source_row_number: 1, measured_at: '2026/07/07 06:30', event: '空腹血糖', glucose_mg_dl: 88, remark_1: '', remark_2: '' }],
  table_rows: [{ source_row_number: 1, measured_at: '2026/07/07 06:30', event: '空腹血糖', glucose_mg_dl: '88', remark_1: '', remark_2: '' }],
  issues: [],
  status: 'succeeded',
  last_successful_sync_at: '2026-07-07T06:30:00Z',
  custom_events: CONFIG_STATUS.custom_events,
  event_thresholds: CONFIG_STATUS.event_thresholds,
};

function jsonResponse(body: unknown): Response {
  return { ok: true, json: () => Promise.resolve(body), text: () => Promise.resolve('') } as unknown as Response;
}

function mockFetch(url: string): Response {
  if (url.startsWith('/api/config/status')) return jsonResponse(CONFIG_STATUS);
  if (url.startsWith('/api/dashboard')) return jsonResponse(DASHBOARD);
  if (url.startsWith('/api/sync')) {
    return jsonResponse({ status: 'succeeded', records: [], issues: [], last_successful_sync_at: 'x', custom_events: [], event_thresholds: [] });
  }
  return jsonResponse({});
}

/// 開啟設定 modal 並回傳 dialog 元素，供 `within` 限定查詢範圍
/// （避免「事件關鍵字設定」「血糖標準值」「空腹血糖」等文字在 sidebar/
/// 主介面也出現造成多匹配）。
async function openSettingsModal(): Promise<HTMLElement> {
  await waitFor(() => expect(screen.getByLabelText('設定')).toBeTruthy());
  fireEvent.click(screen.getByLabelText('設定'));
  return await screen.findByRole('dialog');
}

describe('App 設定 modal（唯讀化 + 工作表名欄位）', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn((url: string) => mockFetch(url)));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('Google Sheet 設定分頁含兩個工作表名稱欄位且預設值正確', async () => {
    render(<App />);
    const dialog = await openSettingsModal();
    const modal = within(dialog);
    // 預設值應來自 config status；用 placeholder 定位輸入框。
    await waitFor(() => {
      expect((modal.getByPlaceholderText('事件關鍵字設定') as HTMLInputElement).value).toBe('事件關鍵字設定');
      expect((modal.getByPlaceholderText('血糖標準值設定') as HTMLInputElement).value).toBe('血糖標準值設定');
    });
  });

  it('事件關鍵字設定分頁為唯讀（無新增表單/刪除按鈕）', async () => {
    render(<App />);
    const dialog = await openSettingsModal();
    const modal = within(dialog);
    // 切到「事件關鍵字設定」分頁：在 settings-tabs nav 內找按鈕。
    const tabsNav = dialog.querySelector('.settings-tabs') as HTMLElement;
    const eventsTab = within(tabsNav).getByText('事件關鍵字設定');
    fireEvent.click(eventsTab);
    // 自訂關鍵字「飲食測試」應顯示。
    await waitFor(() => expect(modal.getByText('飲食測試')).toBeTruthy());
    // 無「新增關鍵字」按鈕、無刪除按鈕（aria-label 含「刪除」）。
    expect(modal.queryByText('新增關鍵字')).toBeNull();
    expect(modal.queryByLabelText('刪除 飲食測試')).toBeNull();
  });

  it('血糖標準值分頁為唯讀（無 number input/儲存鈕）', async () => {
    render(<App />);
    const dialog = await openSettingsModal();
    const modal = within(dialog);
    // 切到「血糖標準值」分頁。
    const tabsNav = dialog.querySelector('.settings-tabs') as HTMLElement;
    const thresholdsTab = within(tabsNav).getByText('血糖標準值');
    fireEvent.click(thresholdsTab);
    // thresholds panel 內應顯示事件標籤（threshold-label）。
    await waitFor(() => expect(modal.getByText('空腹血糖')).toBeTruthy());
    // 無「儲存標準值」按鈕、無 number input。
    expect(modal.queryByText('儲存標準值')).toBeNull();
    expect(modal.queryAllByRole('spinbutton')).toHaveLength(0);
  });
});