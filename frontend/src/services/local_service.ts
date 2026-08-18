import type {
  ConfigStatus,
  ConnectionTestReport,
  DashboardResponse,
  SyncResponse,
} from '../types';

export interface ConfigurePayload {
  sheet_id: string;
  sheet_name?: string;
  fixture_path?: string;
  /// 「事件關鍵字設定」工作表名稱（空白套預設「事件關鍵字設定」）。
  event_keywords_sheet_name?: string;
  /// 「血糖標準值設定」工作表名稱（空白套預設「血糖標準值設定」）。
  glucose_standards_sheet_name?: string;
}

/// 解析 API 錯誤回應的 JSON `{message}`，失敗時回退為 status 文字。
/// 讓 UI 能顯示後端 `AppError::Sync` 指出的哪個工作表/欄位/值出問題。
export async function parseApiError(response: Response, fallback: string): Promise<string> {
  const bodyText = await response.clone().text();
  let message = bodyText || `${fallback} (${response.status})`;
  try {
    const parsed = JSON.parse(bodyText) as { message?: string };
    if (parsed.message) message = parsed.message;
  } catch {
    // 回應非 JSON 時保留原始文字。
  }
  return message;
}

export async function getConfigStatus(): Promise<ConfigStatus> {
  const response = await fetch('/api/config/status');
  if (!response.ok) throw new Error('無法讀取設定狀態');
  return response.json() as Promise<ConfigStatus>;
}

export async function configureDashboard(payload: ConfigurePayload): Promise<ConfigStatus> {
  const response = await fetch('/api/configure', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    throw new Error(await parseApiError(response, '儲存 Google Sheet 設定失敗'));
  }
  return getConfigStatus();
}

export async function testConnection(): Promise<ConnectionTestReport> {
  const response = await fetch('/api/config/test-connection');
  if (!response.ok) {
    throw new Error(await parseApiError(response, '測試連線失敗'));
  }
  return response.json() as Promise<ConnectionTestReport>;
}

/// 時間區間細粒參數。依 `period` 帶對應欄位：
/// `all`（無參數）、`day`（start/end）、`week`（year/week）、
/// `month`（year/month）、`quarter`（year/quarter）。
export interface PeriodSelection {
  period: 'all' | 'day' | 'week' | 'month' | 'quarter';
  start?: string;   // YYYY-MM-DD（day 用）
  end?: string;     // YYYY-MM-DD（day 用）
  year?: number;    // week/month/quarter 用
  week?: number;    // week 用
  month?: number;   // month 用
  quarter?: number; // quarter 用
}

function appendPeriodParams(params: URLSearchParams, sel: PeriodSelection) {
  params.set('period', sel.period);
  if (sel.period === 'day') {
    if (sel.start) params.set('start', sel.start);
    if (sel.end) params.set('end', sel.end);
  } else if (sel.period === 'week') {
    if (sel.year) params.set('year', String(sel.year));
    if (sel.week) params.set('week', String(sel.week));
  } else if (sel.period === 'month') {
    if (sel.year) params.set('year', String(sel.year));
    if (sel.month) params.set('month', String(sel.month));
  } else if (sel.period === 'quarter') {
    if (sel.year) params.set('year', String(sel.year));
    if (sel.quarter) params.set('quarter', String(sel.quarter));
  }
}

export async function getDashboard(selection: PeriodSelection, event?: string, search?: string): Promise<DashboardResponse> {
  const params = new URLSearchParams();
  appendPeriodParams(params, selection);
  if (event) params.set('event', event);
  if (search) params.set('search', search);
  const response = await fetch(`/api/dashboard?${params}`);
  if (!response.ok) {
    throw new Error(await parseApiError(response, '無法取得 Dashboard 資料'));
  }
  return response.json() as Promise<DashboardResponse>;
}

export async function syncDashboard(): Promise<SyncResponse> {
  const response = await fetch('/api/sync', { method: 'POST' });
  if (!response.ok) {
    throw new Error(await parseApiError(response, '同步 Google Sheet 失敗'));
  }
  return response.json() as Promise<SyncResponse>;
}

export function exportUrl(selection: PeriodSelection, event?: string, search?: string): string {
  const params = new URLSearchParams();
  appendPeriodParams(params, selection);
  if (event) params.set('event', event);
  if (search) params.set('search', search);
  return `/api/records/export.csv?${params}`;
}