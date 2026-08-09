import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DashboardLayout } from '../components/layout/DashboardLayout';
import { GlucoseRecordTable } from '../components/records/GlucoseRecordTable';
import { SummaryCards } from '../components/summary/SummaryCards';
import { GlucoseTrendChart } from '../components/trend/GlucoseTrendChart';
import { configureDashboard, exportUrl, getConfigStatus, syncDashboard, testConnection, addCustomEvent, deleteCustomEvent, updateEventThresholds, type PeriodSelection } from '../services/local_service';
import { useDashboard } from '../state/dashboard_store';
import { useTheme } from '../state/use_theme';
import type { ConfigStatus, ConnectionTestReport, CustomEventConfig, EventThreshold } from '../types';

const BUILTIN_FILTERS: [string, string][] = [['', '全部'], ['空腹血糖', '空腹血糖'], ['午餐前', '午餐前'], ['午餐後', '午餐後'], ['晚餐前', '晚餐前'], ['晚餐後', '晚餐後'], ['睡前', '睡前']];

function configLabel(config: ConfigStatus | null): string {
  if (!config) return '尚未讀取設定';
  if (!config.configured) return '尚未設定 Google Sheet';
  return `已設定 ${config.sheet_name || 'Sheet1'}`;
}

const THIS_YEAR = new Date().getFullYear();
const THIS_MONTH = new Date().getMonth() + 1;
const THIS_QUARTER = Math.ceil(THIS_MONTH / 3);

export default function App() {
  const [period, setPeriod] = useState<'all' | 'day' | 'week' | 'month' | 'quarter'>('all');
  const [dayStart, setDayStart] = useState('');
  const [dayEnd, setDayEnd] = useState('');
  const [selYear, setSelYear] = useState(THIS_YEAR);
  const [selWeek, setSelWeek] = useState(1);
  const [selMonth, setSelMonth] = useState(THIS_MONTH);
  const [selQuarter, setSelQuarter] = useState(THIS_QUARTER);
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
  const [testing, setTesting] = useState(false);
  const [connectionResult, setConnectionResult] = useState<ConnectionTestReport | null>(null);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [customEvents, setCustomEvents] = useState<CustomEventConfig[]>([]);
  const [newEventLabel, setNewEventLabel] = useState('');
  const [eventError, setEventError] = useState<string | null>(null);
  const [savingEvent, setSavingEvent] = useState(false);
  // 「血糖標準值」分頁：本地編輯表單與來源 state。eventThresholds 為已儲存的值，
  // 用於趨勢圖/表格上色；thresholdsDraft 為分頁內可編輯的暫存值，儲存後才套用。
  const [eventThresholds, setEventThresholds] = useState<EventThreshold[]>([]);
  const [thresholdsDraft, setThresholdsDraft] = useState<EventThreshold[]>([]);
  const [thresholdsError, setThresholdsError] = useState<string | null>(null);
  const [savingThresholds, setSavingThresholds] = useState(false);

  // 由目前時間區間選擇組裝 PeriodSelection，供 useDashboard 與匯出使用。
  const selection: PeriodSelection = useMemo(() => {
    if (period === 'day') return { period, start: dayStart, end: dayEnd };
    if (period === 'week') return { period, year: selYear, week: selWeek };
    if (period === 'month') return { period, year: selYear, month: selMonth };
    if (period === 'quarter') return { period, year: selYear, quarter: selQuarter };
    return { period: 'all' };
  }, [period, dayStart, dayEnd, selYear, selWeek, selMonth, selQuarter]);

  const { data, loading, error, reload } = useDashboard(selection, event || undefined, search || undefined);
  const [refreshing, setRefreshing] = useState(false);

  // 「立即更新」：強制重新抓取 Sheet（POST /api/sync），更新同步時間後再重載儀表板。
  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      await syncDashboard();
      await reload();
    } finally {
      setRefreshing(false);
    }
  }, [reload]);

  const loadConfig = useCallback(async () => {
    setConfigLoading(true);
    setConfigError(null);
    try {
      const next = await getConfigStatus();
      setConfig(next);
      setCustomEvents(next.custom_events || []);
      const thresholds = next.event_thresholds || [];
      setEventThresholds(thresholds);
      setThresholdsDraft(thresholds.map((t) => ({ ...t })));
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

  async function handleTestConnection() {
    setTesting(true);
    setConnectionError(null);
    try {
      const result = await testConnection();
      setConnectionResult(result);
    } catch (cause) {
      setConnectionResult(null);
      setConnectionError(cause instanceof Error ? cause.message : '測試連線失敗');
    } finally {
      setTesting(false);
    }
  }

  // 新增自訂事件關鍵字。後端以預設顯示標準 70–140 補入；成功後重載設定與儀表板，
  // 並同步 event_thresholds 讓「血糖標準值」分頁即時出現新事件。
  async function handleAddCustomEvent(submitEvent: { preventDefault: () => void }) {
    submitEvent.preventDefault();
    setEventError(null);
    setSavingEvent(true);
    try {
      const next = await addCustomEvent({ label: newEventLabel.trim() });
      setConfig(next);
      setCustomEvents(next.custom_events);
      const thresholds = next.event_thresholds || [];
      setEventThresholds(thresholds);
      setThresholdsDraft(thresholds.map((t) => ({ ...t })));
      setNewEventLabel('');
      await reload();
    } catch (cause) {
      setEventError(cause instanceof Error ? cause.message : '新增事件關鍵字失敗');
    } finally {
      setSavingEvent(false);
    }
  }

  async function handleDeleteCustomEvent(label: string) {
    setEventError(null);
    try {
      const next = await deleteCustomEvent(label);
      setConfig(next);
      setCustomEvents(next.custom_events);
      const thresholds = next.event_thresholds || [];
      setEventThresholds(thresholds);
      setThresholdsDraft(thresholds.map((t) => ({ ...t })));
      // 若目前選中的事件被刪除，回到「全部」。
      if (event === label) setEvent('');
      await reload();
    } catch (cause) {
      setEventError(cause instanceof Error ? cause.message : '刪除事件關鍵字失敗');
    }
  }

  // 儲存「血糖標準值」分頁編輯。後端要求完整集合（6 內建 + 全部現存自訂事件），
  // thresholdsDraft 已含兩者；直接送出。成功後套用至上色來源並重載儀表板。
  async function handleSaveThresholds(submitEvent: { preventDefault: () => void }) {
    submitEvent.preventDefault();
    setThresholdsError(null);
    setSavingThresholds(true);
    try {
      const next = await updateEventThresholds({ event_thresholds: thresholdsDraft });
      setConfig(next);
      const thresholds = next.event_thresholds || [];
      setEventThresholds(thresholds);
      setThresholdsDraft(thresholds.map((t) => ({ ...t })));
      await reload();
    } catch (cause) {
      setThresholdsError(cause instanceof Error ? cause.message : '儲存血糖標準值失敗');
    } finally {
      setSavingThresholds(false);
    }
  }

  // 更新 thresholdsDraft 中指定事件的下限/上限欄位。
  function updateThresholdField(label: string, field: 'low' | 'high', value: number) {
    setThresholdsDraft(thresholdsDraft.map((t) => (t.label === label ? { ...t, [field]: value } : t)));
  }

  // 側邊篩選項目：內建 6 個 + 使用者自訂關鍵字。
  const filters = useMemo<[string, string][]>(
    () => [...BUILTIN_FILTERS, ...customEvents.map((c) => [c.label, c.label] as [string, string])],
    [customEvents],
  );

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

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsTab, setSettingsTab] = useState<'sheet' | 'events' | 'thresholds'>('sheet');
  const [theme, toggleTheme] = useTheme();

  const sheetPanel = <>
    <section className="side-section setup-card">
      <h3>⚙　Google Sheet 設定</h3>
      <div className="status-banner">
        <strong>{configLabel(config)}</strong>
        <span>{config?.credential_store || 'credential store 狀態讀取中'}</span>
      </div>
      {configError ? <p className="inline-error">{configError}</p> : null}
      <form className="sheet-form" onSubmit={handleSubmit}>
        <label>Google Sheet 網址或 ID<input value={sheetId} onChange={(event: { target: { value: string } }) => { setSheetId(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="https://docs.google.com/spreadsheets/d/.../edit" required /></label>
        <label>工作表名稱<input value={sheetName} onChange={(event: { target: { value: string } }) => { setSheetName(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="Sheet1" /></label>
        <label>本機 CSV 路徑<input value={fixturePath} onChange={(event: { target: { value: string } }) => { setFixturePath(event.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="backend/tests/fixtures/valid-sheet.csv" /></label>
        <div className="form-actions">
          <button type="submit" disabled={saving || configLoading}>{saving ? '儲存中…' : '儲存設定'}</button>
          <button type="button" onClick={() => { setDirty(false); dirtyRef.current = false; void loadConfig(); }}>重新載入</button>
        </div>
        <button className="connection-check" type="button" onClick={() => void handleTestConnection()} disabled={testing || configLoading}>{testing ? '測試中…' : '測試連線'}</button>
        <p className="form-hint">可直接貼整個 Google Sheet 網址，系統會自動擷取 Sheet ID。工作表名稱若未指定，預設為 `Sheet1`。</p>
      </form>
      {connectionError ? <div className="connection-result error"><strong>測試失敗</strong><p>{connectionError}</p></div> : null}
      {connectionResult ? <div className={`connection-result ${connectionResult.ok ? 'ok' : 'error'}`}><strong>{connectionResult.ok ? '測試成功' : '測試失敗'}</strong><p>{connectionResult.message}</p><p>HTTP：{connectionResult.http_status ?? '未知'}　記錄：{connectionResult.record_count ?? '未知'}　問題：{connectionResult.issue_count ?? '未知'}</p><p>Sheet GID：{connectionResult.sheet_gid ?? '未指定'}</p><p>網址：{connectionResult.url ?? '未知'}</p>{connectionResult.detail ? <p className="connection-detail">{connectionResult.detail}</p> : null}</div> : null}
      <div className="config-meta">
        <p><span>Sheet ID</span><strong>{config?.sheet_id || '尚未設定'}</strong></p>
        <p><span>Sheet GID</span><strong>{config?.sheet_gid || '未指定'}</strong></p>
        <p><span>工作表</span><strong>{config?.sheet_name || 'Sheet1'}</strong></p>
        <p><span>來源</span><strong>{config?.fixture_path || 'Google Sheet 直讀'}</strong></p>
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
  </>;

  const eventsPanel = <>
    <section className="side-section custom-events-card">
      <h3>🏷　事件關鍵字設定</h3>
      <p className="form-hint">新增自訂事件關鍵字後，事件欄填入相同字串的列會出現在側邊「篩選項目」。每個事件的標準範圍請至「血糖標準值」分頁設定。</p>
      {eventError ? <p className="inline-error">{eventError}</p> : null}
      <ul className="custom-event-list">
        {customEvents.map((c) => <li key={c.label}><span className="custom-event-label">{c.label}</span><button type="button" className="custom-event-remove" onClick={() => void handleDeleteCustomEvent(c.label)} aria-label={`刪除 ${c.label}`}>✕</button></li>)}
        {customEvents.length === 0 ? <li className="custom-event-empty">尚未新增自訂事件關鍵字</li> : null}
      </ul>
      <form className="sheet-form" onSubmit={handleAddCustomEvent}>
        <label>關鍵字<input value={newEventLabel} onChange={(e: { target: { value: string } }) => setNewEventLabel(e.target.value)} placeholder="例如：運動後" required /></label>
        <div className="form-actions">
          <button type="submit" disabled={savingEvent}>{savingEvent ? '新增中…' : '新增關鍵字'}</button>
        </div>
        <p className="form-hint">新增後可在「血糖標準值」分頁調整該事件的正常範圍。</p>
      </form>
    </section>
  </>;

  const thresholdsPanel = <>
    <section className="side-section thresholds-card">
      <h3>⚖　血糖標準值</h3>
      <p className="form-hint">設定每個事件的正常血糖範圍。趨勢圖與血糖紀錄會依此上色：超過上限為紅色、範圍內為綠色、低於下限為黃色。此設定不影響摘要統計。</p>
      {thresholdsError ? <p className="inline-error">{thresholdsError}</p> : null}
      <form className="sheet-form" onSubmit={handleSaveThresholds}>
        <ul className="threshold-list">
          {thresholdsDraft.map((t) => <li key={t.label}>
            <span className="threshold-label">{t.label}</span>
            <div className="threshold-row">
              <label>下限<input type="number" min={20} max={600} value={t.low} onChange={(e: { target: { value: string } }) => updateThresholdField(t.label, 'low', Number(e.target.value))} /></label>
              <span className="range-sep">～</span>
              <label>上限<input type="number" min={20} max={600} value={t.high} onChange={(e: { target: { value: string } }) => updateThresholdField(t.label, 'high', Number(e.target.value))} /></label>
            </div>
          </li>)}
          {thresholdsDraft.length === 0 ? <li className="custom-event-empty">載入標準值中…</li> : null}
        </ul>
        <div className="form-actions">
          <button type="submit" disabled={savingThresholds}>{savingThresholds ? '儲存中…' : '儲存標準值'}</button>
        </div>
        <p className="form-hint">閾值須介於 20–600，且下限須小於上限。內建事件不可刪除，自訂事件請至「事件關鍵字」分頁管理。</p>
      </form>
    </section>
  </>;

  const settingsTabs: [typeof settingsTab, string, string][] = [['sheet', 'sheet', 'Google Sheet 設定'], ['events', 'events', '事件關鍵字設定'], ['thresholds', 'thresholds', '血糖標準值']];

  const settingsPanel = <>
    <nav className="settings-tabs" aria-label="設定分頁">
      {settingsTabs.map(([key, id, label]) => <button key={key} type="button" className={`settings-tab ${settingsTab === key ? 'active' : ''}`} aria-selected={settingsTab === key} onClick={() => setSettingsTab(key)}>{label}</button>)}
    </nav>
    <div className="settings-tab-content">{settingsTab === 'sheet' ? sheetPanel : settingsTab === 'events' ? eventsPanel : thresholdsPanel}</div>
  </>;

  const sidebar = <>
    <section className="side-section">
      <h3>▣　時間區間</h3>
      <div className="periods">{[['all', '全部'], ['day', '日'], ['week', '週'], ['month', '月'], ['quarter', '季']].map(([key, label]) => <button key={key} className={period === key ? 'active' : ''} type="button" onClick={() => setPeriod(key as typeof period)}>{label}</button>)}</div>
      {period === 'day' ? <div className="period-config">
        <label>起<input type="date" value={dayStart} onChange={(e: { target: { value: string } }) => setDayStart(e.target.value)} placeholder="2026/07/07" /></label>
        <span className="range-sep">～</span>
        <label>訖<input type="date" value={dayEnd} onChange={(e: { target: { value: string } }) => setDayEnd(e.target.value)} placeholder="2026/07/07" /></label>
      </div> : null}
      {period === 'week' ? <div className="period-config">
        <label>年<input type="number" min="2000" max="2100" value={selYear} onChange={(e: { target: { value: string } }) => setSelYear(Number(e.target.value))} /></label>
        <label>週<input type="number" min="1" max="53" value={selWeek} onChange={(e: { target: { value: string } }) => setSelWeek(Number(e.target.value))} /></label>
        <p className="config-hint">第 N 週以 1/1 起算，每 7 天為一週。</p>
      </div> : null}
      {period === 'month' ? <div className="period-config">
        <label>年<input type="number" min="2000" max="2100" value={selYear} onChange={(e: { target: { value: string } }) => setSelYear(Number(e.target.value))} /></label>
        <label>月<select value={selMonth} onChange={(e: { target: { value: string } }) => setSelMonth(Number(e.target.value))}>{Array.from({ length: 12 }, (_, i) => i + 1).map((m) => <option key={m} value={m}>{m} 月</option>)}</select></label>
      </div> : null}
      {period === 'quarter' ? <div className="period-config">
        <label>年<input type="number" min="2000" max="2100" value={selYear} onChange={(e: { target: { value: string } }) => setSelYear(Number(e.target.value))} /></label>
        <label>季<select value={selQuarter} onChange={(e: { target: { value: string } }) => setSelQuarter(Number(e.target.value))}>{[1, 2, 3, 4].map((q) => <option key={q} value={q}>第 {q} 季（{q === 1 ? '1-3' : q === 2 ? '4-6' : q === 3 ? '7-9' : '10-12'} 月）</option>)}</select></label>
      </div> : null}
      <button className="refresh" type="button" disabled={refreshing} onClick={() => void handleRefresh()}>{refreshing ? '⟳　更新中…' : '⟳　立即更新'}</button>
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
    if (!data) return <div className="state-card"><h2>尚未設定資料來源</h2><p>請先按右上角「設定」按鈕填寫 Google Sheet 設定，再按「儲存設定」與「立即更新」。</p></div>;
    return <><SummaryCards summary={data.summary} /><GlucoseTrendChart records={data.records} eventThresholds={eventThresholds} /><GlucoseRecordTable rows={data.table_rows} eventThresholds={eventThresholds} search={search} onSearch={setSearch} onExport={() => { window.location.href = exportUrl(selection, event || undefined, search || undefined); }} /></>;
  }, [configLoading, customEvents, eventThresholds, data, error, event, loading, selection, reload, search]);

  return <DashboardLayout sidebar={sidebar} onOpenSettings={() => setSettingsOpen(true)} theme={theme} onToggleTheme={toggleTheme}>{content}{settingsOpen ? <div className="settings-overlay" role="dialog" aria-modal="true" aria-label="設定"><div className="settings-modal"><div className="settings-modal-header"><h2>⚙　設定</h2><button className="settings-close" type="button" aria-label="關閉" onClick={() => setSettingsOpen(false)}>✕</button></div><div className="settings-modal-body">{settingsPanel}</div></div></div> : null}</DashboardLayout>;
}
