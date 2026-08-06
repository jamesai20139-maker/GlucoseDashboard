import type { ConfigStatus, ConnectionTestReport, DashboardResponse } from '../types';

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

export async function getDashboard(period = 'all', event?: string, search?: string): Promise<DashboardResponse> {
  const params = new URLSearchParams({ period });
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

export function exportUrl(period = 'all', event?: string, search?: string): string {
  const params = new URLSearchParams({ period });
  if (event) params.set('event', event);
  if (search) params.set('search', search);
  return `/api/records/export.csv?${params}`;
}
