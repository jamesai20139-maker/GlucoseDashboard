import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DashboardLayout } from '../components/layout/DashboardLayout';
import { GlucoseRecordTable } from '../components/records/GlucoseRecordTable';
import { SummaryCards } from '../components/summary/SummaryCards';
import { GlucoseTrendChart } from '../components/trend/GlucoseTrendChart';
import { configureDashboard, exportUrl, getConfigStatus } from '../services/local_service';
import { useDashboard } from '../state/dashboard_store';
import type { ConfigStatus } from '../types';

const filters = [['', '全部'], ['空腹血糖', '空腹血糖'], ['午餐前', '午餐前'], ['午餐後', '午餐後'], ['晚餐前', '晚餐前'], ['晚餐後', '晚餐後'], ['睡前', '睡前']];

function configLabel(config: ConfigStatus | null): string {
  if (!config) return '尚未讀取設定';
  if (!config.configured) return '尚未設定 Google Sheet';
  return `已設定 ${config.sheet_name || 'Sheet1'}`;
}

export default function App() {
  const [period, setPeriod] = useState('all');
  const [event, setEvent] = useState('');
  const [search, setSearch] = useState('');
  const [config, setConfig] = useState<ConfigStatus | null>(null);
  const [configLoading, setConfigLoading] = useState(true);
  const [configError, setConfigError] = useState<string | null>(null);
  const [sheetId, setSheetId] = useState('');
  const [sheetName, setSheetName] = useState('Sheet1');
  const [fixturePath, setFixturePath] = useState('');
  const [dirty, setDirty] = useState(false);
  const dirtyRef = useRef(false);
  const [saving, setSaving] = useState(false);
  const { data, loading, error, reload } = useDashboard(period, event || undefined, search || undefined);

  const loadConfig = useCallback(async () => {
    setConfigLoading(true);
    setConfigError(null);
    try {
      const next = await getConfigStatus();
      setConfig(next);
      if (!dirtyRef.current) {
        setSheetId(next.sheet_id || '');
        setSheetName(next.sheet_name || 'Sheet1');
        setFixturePath(next.fixture_path || '');
      }
    } catch (cause) {
      setConfigError(cause instanceof Error ? cause.message : '讀取設定失敗');
    } finally {
      setConfigLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  async function handleSubmit(event: { preventDefault: () => void }) {
    event.preventDefault();
    setSaving(true);
    try {
      const next = await configureDashboard({
        sheet_id: sheetId.trim(),
        sheet_name: sheetName.trim() || 'Sheet1',
        fixture_path: fixturePath.trim() || undefined,
      });
      setConfig(next);
      setDirty(false);
      dirtyRef.current = false;
      await reload();
    } catch (cause) {
      setConfigError(cause instanceof Error ? cause.message : '儲存設定失敗');
    } finally {
      setSaving(false);
    }
  }

  const setupSteps = useMemo(() => [
    '在 Google Sheet 右上角複製網址，找到 `/d/` 與 `/edit` 之間的字串，貼到「Sheet ID」。',
    '確認工作表名稱與實際分頁一致；如果只有一張表，可先保留 `Sheet1`。',
    '檢查標題列是否完全符合專案定義，包含日期時間、事件、血糖、備註1、備註2。',
    '若要讀取真實 Google Sheet，請確認該 Sheet 對目前環境可讀取；如果要先測試，也可以填入本機 CSV 路徑。',
    '按下「儲存設定」後，使用「立即更新」重新讀取資料，確認卡片、趨勢圖和表格都有內容。',
  ], []);

  const notes = useMemo(() => [
    'Sheet ID 只能是來源網址中的識別碼，不要貼整個網址。',
    '目前同步會優先讀取本機 CSV；沒有本機檔案時，會嘗試用 Google Sheet 的 CSV 匯出端點讀取。',
    '如果沒有讀取權限、標題列不正確，或來源檔案不存在，同步會失敗並顯示錯誤。',
    '真實 Google Sheet 建議只給讀取權限，避免把可寫入的分享權限交給不必要的人。',
  ], []);

  const sidebar = <>
    <section className="side-section setup-card">
      <h3>⚙　Google Sheet 設定</h3>
      <div className="status-banner">
        <strong>{configLabel(config)}</strong>
        <span>{config?.credential_store || 'credential store 狀態讀取中'}</span>
      </div>
      {configError ? <p className="inline-error">{configError}</p> : null}
      <form className="sheet-form" onSubmit={handleSubmit}>
        <label>Sheet ID<input value={sheetId} onChange={(event: { target: { value: string } }) => { setSheetId(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="1AbC...xYz" required /></label>
        <label>工作表名稱<input value={sheetName} onChange={(event: { target: { value: string } }) => { setSheetName(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="Sheet1" /></label>
        <label>本機 CSV 路徑<input value={fixturePath} onChange={(event: { target: { value: string } }) => { setFixturePath(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="backend/tests/fixtures/valid-sheet.csv" /></label>
        <div className="form-actions">
          <button type="submit" disabled={saving || configLoading}>{saving ? '儲存中…' : '儲存設定'}</button>
          <button type="button" onClick={() => { setDirty(false); dirtyRef.current = false; void loadConfig(); }}>重新載入</button>
        </div>
        <p className="form-hint">Sheet ID 請從 Google Sheet URL 擷取。工作表名稱若未指定，預設為 `Sheet1`。</p>
      </form>
      <div className="config-meta">
        <p><span>Sheet ID</span><strong>{config?.sheet_id || '尚未設定'}</strong></p>
        <p><span>工作表</span><strong>{config?.sheet_name || 'Sheet1'}</strong></p>
        <p><span>來源</span><strong>{config?.fixture_path || '未指定本機 CSV'}</strong></p>
        <p><span>最後同步</span><strong>{config?.last_successful_sync_at ? config.last_successful_sync_at.slice(0, 16).replace('T', ' ') : '尚未同步'}</strong></p>
      </div>
    </section>
    <section className="side-section guidance-card">
      <h3>📘　設定步驟</h3>
      <ol className="step-list">
        {setupSteps.map((step) => <li key={step}>{step}</li>)}
      </ol>
    </section>
    <section className="side-section guidance-card">
      <h3>⚠　注意事項</h3>
      <ul className="note-list">
        {notes.map((note) => <li key={note}>{note}</li>)}
      </ul>
    </section>
    <section className="side-section">
      <h3>▣　時間區間</h3>
      <div className="periods">{[['all', '日'], ['week', '週'], ['month', '月'], ['quarter', '季']].map(([key, label]) => <button key={key} className={period === key ? 'active' : ''} type="button" onClick={() => setPeriod(key)}>{label}</button>)}</div>
      <label className="date-label">▣　自訂日期<input type="text" placeholder="2026/07/07 ～ 2026/07/07" /></label>
      <button className="refresh" type="button" onClick={() => void reload()}>⟳　立即更新</button>
      <p className="sync-status"><i /> {data?.last_successful_sync_at ? `資料已更新：${data.last_successful_sync_at.slice(0, 16).replace('T', ' ')}` : '等待資料同步'}</p>
    </section>
    <section className="side-section">
      <h3>⚱　篩選項目</h3>
      {filters.map(([key, label]) => <label className={`radio-row ${event === key ? 'selected' : ''}`} key={key}><input type="radio" name="event" checked={event === key} onChange={() => setEvent(key)} />{label}</label>)}
    </section>
    <section className="side-section reference">
      <h3>ⓘ　血糖標準值</h3>
      <p>● 正常人空腹 8 小時後，70～99 mg/dL</p>
      <p>● 正常人餐前（距離上一餐至少 4 小時），70～100 mg/dL</p>
      <p className="warning">● 餐後 2 小時所測量，≥140 mg/dL 視為偏高</p>
    </section>
  </>;

  const content = useMemo(() => {
    if (loading || configLoading) return <div className="state-card">正在讀取血糖資料與設定…</div>;
    if (error) return <div className="state-card error-state"><h2>同步失敗</h2><p>{error}</p><button type="button" onClick={() => void reload()}>重新嘗試</button></div>;
    if (!data) return <div className="state-card"><h2>尚未設定資料來源</h2><p>請先填寫左側的 Google Sheet 設定，再按「儲存設定」與「立即更新」。</p></div>;
    return <><SummaryCards summary={data.summary} /><GlucoseTrendChart records={data.records} /><GlucoseRecordTable records={data.records} search={search} onSearch={setSearch} onExport={() => { window.location.href = exportUrl(period, event || undefined, search || undefined); }} /></>;
  }, [configLoading, data, error, event, loading, period, reload, search]);

  return <DashboardLayout sidebar={sidebar}>{content}</DashboardLayout>;
}
