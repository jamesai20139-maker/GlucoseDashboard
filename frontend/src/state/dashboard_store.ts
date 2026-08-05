import { useCallback, useEffect, useState } from 'react';
import { getDashboard } from '../services/local_service';
import type { DashboardResponse } from '../types';

export function useDashboard(period: string, event?: string, search?: string) {
  const [data, setData] = useState<DashboardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { setData(await getDashboard(period, event, search)); }
    catch (cause) { setData(null); setError(cause instanceof Error ? cause.message : '載入失敗'); }
    finally { setLoading(false); }
  }, [period, event, search]);
  useEffect(() => { void reload(); }, [reload]);
  return { data, loading, error, reload };
}
