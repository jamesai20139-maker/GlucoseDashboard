import { useCallback, useEffect, useState } from 'react';
import { getDashboard, type PeriodSelection } from '../services/local_service';
import type { DashboardResponse } from '../types';

export function useDashboard(selection: PeriodSelection, event?: string, search?: string) {
  const [data, setData] = useState<DashboardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  // 以 JSON 字串作為依賴鍵，避免物件每次 render 都被視為變更。
  const key = JSON.stringify(selection);
  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { setData(await getDashboard(selection, event, search)); }
    catch (cause) { setData(null); setError(cause instanceof Error ? cause.message : '載入失敗'); }
    finally { setLoading(false); }
  }, [key, event, search]);
  useEffect(() => { void reload(); }, [reload]);
  return { data, loading, error, reload };
}