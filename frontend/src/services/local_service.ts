import type { DashboardResponse } from '../types';

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
