import type { ConfigStatus, ConnectionTestReport, DashboardResponse, EventThreshold } from '../types';

export interface ConfigurePayload {
  sheet_id: string;
  sheet_name?: string;
  fixture_path?: string;
}

export async function getConfigStatus(): Promise<ConfigStatus> {
  const response = await fetch('/api/config/status');
  if (!response.ok) throw new Error('無法讀取設定狀態');
  return response.json() as Promise<ConfigStatus>;
}

/// 新增自訂事件關鍵字。後端固定以預設顯示標準 70–140 補入，閾值改由
/// 「血糖標準值」分頁（`updateEventThresholds`）設定；此端點只管理關鍵字名稱。
/// 同 label 為冪等，不重設既有閾值。
export async function addCustomEvent(payload: { label: string }): Promise<ConfigStatus> {
  const response = await fetch('/api/custom-events', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || '新增事件關鍵字失敗');
  }
  return response.json() as Promise<ConfigStatus>;
}

/// 刪除指定 label 的自訂事件關鍵字（含其顯示標準）。
export async function deleteCustomEvent(label: string): Promise<ConfigStatus> {
  const response = await fetch(`/api/custom-events/${encodeURIComponent(label)}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || '刪除事件關鍵字失敗');
  }
  return response.json() as Promise<ConfigStatus>;
}

/// 批次覆寫全部事件的前端顯示標準範圍。須提交完整集合（6 內建 + 全部現存自訂事件）。
/// 驗證：label 非空、屬內建或現存自訂、不重複、low/high 20–600、low < high。
export async function updateEventThresholds(payload: {
  event_thresholds: EventThreshold[];
}): Promise<ConfigStatus> {
  const response = await fetch('/api/event-thresholds', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    const bodyText = await response.clone().text();
    let message = bodyText || `儲存血糖標準值失敗 (${response.status})`;
    try {
      const parsed = JSON.parse(bodyText) as { message?: string };
      message = parsed.message || message;
    } catch {
      // 回應非 JSON 時保留原始文字。
    }
    throw new Error(message);
  }
  return response.json() as Promise<ConfigStatus>;
}

export async function configureDashboard(payload: ConfigurePayload): Promise<ConfigStatus> {
  const response = await fetch('/api/configure', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || '儲存 Google Sheet 設定失敗');
  }
  return getConfigStatus();
}

export async function testConnection(): Promise<ConnectionTestReport> {
  const response = await fetch('/api/config/test-connection');
  if (!response.ok) {
    const bodyText = await response.clone().text();
    let message = bodyText || `測試連線失敗 (${response.status})`;
    try {
      const parsed = JSON.parse(bodyText) as { message?: string };
      message = parsed.message || message;
    } catch {
      // Keep the raw body text when the response is not JSON.
    }
    throw new Error(message);
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
  if (!response.ok) throw new Error('無法取得 Dashboard 資料');
  return response.json() as Promise<DashboardResponse>;
}

export async function syncDashboard(): Promise<DashboardResponse> {
  const response = await fetch('/api/sync', { method: 'POST' });
  if (!response.ok) throw new Error('同步 Google Sheet 失敗');
  return response.json() as Promise<DashboardResponse>;
}

export function exportUrl(selection: PeriodSelection, event?: string, search?: string): string {
  const params = new URLSearchParams();
  appendPeriodParams(params, selection);
  if (event) params.set('event', event);
  if (search) params.set('search', search);
  return `/api/records/export.csv?${params}`;
}
