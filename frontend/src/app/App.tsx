import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DashboardLayout } from '../components/layout/DashboardLayout';
import { GlucoseRecordTable } from '../components/records/GlucoseRecordTable';
import { SummaryCards } from '../components/summary/SummaryCards';
import { GlucoseTrendChart } from '../components/trend/GlucoseTrendChart';
import { configureDashboard, exportUrl, getConfigStatus, syncDashboard, testConnection, type PeriodSelection } from '../services/local_service';
import { useDashboard } from '../state/dashboard_store';
import { useTheme } from '../state/use_theme';
import type { ConfigStatus, ConnectionTestReport, CustomEventConfig, EventThreshold } from '../types';

const BUILTIN_FILTERS: [string, string][] = [['', '全部'], ['空腹血糖', '空腹血糖'], ['午餐前', '午餐前'], ['午餐後', '午餐後'], ['晚餐前', '晚餐前'], ['晚餐後', '晚餐後'], ['睡前', '睡前']];

const DEFAULT_EVENT_KEYWORDS_SHEET_NAME = '事件關鍵字設定';
const DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME = '血糖標準值設定';

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
  // 兩個設定工作表名稱（預設為內建常數）。
  const [eventKeywordsSheetName, setEventKeywordsSheetName] = useState(DEFAULT_EVENT_KEYWORDS_SHEET_NAME);
  const [glucoseStandardsSheetName, setGlucoseStandardsSheetName] = useState(DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME);
  const [dirty, setDirty] = useState(false);
  const dirtyRef = useRef(false);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [connectionResult, setConnectionResult] = useState<ConnectionTestReport | null>(null);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  // 自訂事件關鍵字與血糖標準值：自 schema 4 起改由 Google Sheet 即時衍生，
  // 不再於本機編輯。此處僅快取最近一次 DashboardResponse/SyncResponse 的值，
  // 供側邊篩選、圖/表上色與唯讀顯示使用；首次同步前為空。
  const [customEvents, setCustomEvents] = useState<CustomEventConfig[]>([]);
  const [eventThresholds, setEventThresholds] = useState<EventThreshold[]>([]);

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

  // 從 DashboardResponse 同步 Sheet 衍生的設定（事件關鍵字、血糖標準值）。
  // 每次儀表板載入都會更新；首次同步前 data 可能為 null（此時沿用既有快取）。
  useEffect(() => {
    if (data) {
      setCustomEvents(data.custom_events || []);
      setEventThresholds(data.event_thresholds || []);
      // Sheet 編輯後目前選中的自訂事件可能消失 → 回到「全部」。
      if (event && !BUILTIN_FILTERS.some(([key]) => key === event)
          && !(data.custom_events || []).some((c) => c.label === event)) {
        setEvent('');
      }
    }
  }, [data]); // eslint-disable-line react-hooks/exhaustive-deps

  // 「立即更新」：強制重新抓取 Sheet（POST /api/sync），套用回應的設定後再重載儀表板。
  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      const synced = await syncDashboard();
      // 同步回應已含最新衍生設定，立即套用（不必等 dashboard 重載）。
      setCustomEvents(synced.custom_events || []);
      setEventThresholds(synced.event_thresholds || []);
      await reload();
    } catch (cause) {
      // 同步失敗訊息會經由 useDashboard 的下次 reload 顯示；此處不額外處理。
      void cause;
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
      // custom_events/event_thresholds 為快取暫存值（首次同步前可能為空）；
      // 不在此覆蓋，改由 DashboardResponse 同步以取得最新值。
      if (!dirtyRef.current) {
        setSheetId(next.sheet_id || '');
        setSheetName(next.sheet_name || 'Sheet1');
        setFixturePath(next.fixture_path || '');
        setEventKeywordsSheetName(next.event_keywords_sheet_name || DEFAULT_EVENT_KEYWORDS_SHEET_NAME);
        setGlucoseStandardsSheetName(next.glucose_standards_sheet_name || DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME);
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

  async function handleSubmit(submitEvent: { preventDefault: () => void }) {
    submitEvent.preventDefault();
    setSaving(true);
    try {
      const next = await configureDashboard({
        sheet_id: sheetId.trim(),
        sheet_name: sheetName.trim() || 'Sheet1',
        fixture_path: fixturePath.trim() || undefined,
        event_keywords_sheet_name: eventKeywordsSheetName.trim() || undefined,
        glucose_standards_sheet_name: glucoseStandardsSheetName.trim() || undefined,
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

  // 側邊篩選項目：內建 6 個 + 使用者自訂關鍵字。
  const filters = useMemo<[string, string][]>(
    () => [...BUILTIN_FILTERS, ...customEvents.map((c) => [c.label, c.label] as [string, string])],
    [customEvents],
  );

  const setupSteps = useMemo(() => [
    '可直接貼整個 Google Sheet 網址，系統會自動擷取 Sheet ID。',
    '確認「資料工作表名稱」與實際分頁一致；如果只有一張表，可先保留 `Sheet1`。',
    '確認工作表中有兩個設定分頁：「事件關鍵字設定」與「血糖標準值設定」，名稱可在下方欄位調整。',
    '「事件關鍵字設定」分頁須為單欄，標頭為「事件關鍵字」；「血糖標準值設定」分頁須為三欄，標頭為「事件,血糖下限,血糖上限」，並含全部六個內建事件。',
    '按下「儲存設定」後，使用「立即更新」重新讀取資料，確認卡片、趨勢圖和表格都有內容。',
  ], []);

  const notes = useMemo(() => [
    '一旦設定 Google Sheet 網址，系統一律直讀 Sheet（非本機 CSV 優先）。',
    '事件關鍵字與血糖標準值改由 Google Sheet 的兩個設定分頁即時衍生，不再於本機編輯；每次載入儀表板或同步都會重新讀取。',
    '只設本機 CSV 路徑（未連結 Sheet）時，事件關鍵字為無、血糖標準值退回六個內建預設。',
    '若設定分頁缺失、空白或格式錯誤，儀表板會顯示阻斷錯誤訊息（指出哪個分頁/欄位/值有問題）。',
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
        <label>Google Sheet 網址或 ID<input value={sheetId} onChange={(e: { target: { value: string } }) => { setSheetId(e.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="https://docs.google.com/spreadsheets/d/.../edit" required /></label>
        <label>資料工作表名稱<input value={sheetName} onChange={(e: { target: { value: string } }) => { setSheetName(e.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="Sheet1" /></label>
        <label>事件關鍵字工作表名稱<input value={eventKeywordsSheetName} onChange={(e: { target: { value: string } }) => { setEventKeywordsSheetName(e.target.value); setDirty(true); dirtyRef.current = true; }} placeholder={DEFAULT_EVENT_KEYWORDS_SHEET_NAME} /></label>
        <label>血糖標準值工作表名稱<input value={glucoseStandardsSheetName} onChange={(e: { target: { value: string } }) => { setGlucoseStandardsSheetName(e.target.value); setDirty(true); dirtyRef.current = true; }} placeholder={DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME} /></label>
        <label>本機 CSV 路徑<input value={fixturePath} onChange={(e: { target: { value: string } }) => { setFixturePath(e.target.value); setDirty(true); dirtyRef.current = true; }} placeholder="backend/tests/fixtures/valid-sheet.csv" /></label>
        <div className="form-actions">
          <button type="submit" disabled={saving || configLoading}>{saving ? '儲存中…' : '儲存設定'}</button>
          <button type="button" onClick={() => { setDirty(false); dirtyRef.current = false; void loadConfig(); }}>重新載入</button>
        </div>
        <button className="connection-check" type="button" onClick={() => void handleTestConnection()} disabled={testing || configLoading}>{testing ? '測試中…' : '測試連線'}</button>
        <p className="form-hint">可直接貼整個 Google Sheet 網址，系統會自動擷取 Sheet ID。資料工作表名稱預設為 `Sheet1`；兩個設定工作表名稱預設為「事件關鍵字設定」與「血糖標準值設定」。</p>
      </form>
      {connectionError ? <div className="connection-result error"><strong>測試失敗</strong><p>{connectionError}</p></div> : null}
      {connectionResult ? <div className={`connection-result ${connectionResult.ok ? 'ok' : 'error'}`}>
        <strong>{connectionResult.ok ? '測試成功' : '測試失敗'}</strong>
        <p>資料工作表：{connectionResult.data_sheet.ok ? '✓' : '✗'} {connectionResult.data_sheet.message}</p>
        <p>事件關鍵字設定：{connectionResult.event_keywords_sheet.ok ? '✓' : '✗'} {connectionResult.event_keywords_sheet.message}</p>
        <p>血糖標準值設定：{connectionResult.glucose_standards_sheet.ok ? '✓' : '✗'} {connectionResult.glucose_standards_sheet.message}</p>
        {connectionResult.data_sheet.detail ? <p className="connection-detail">資料表：{connectionResult.data_sheet.detail}</p> : null}
        {connectionResult.event_keywords_sheet.detail ? <p className="connection-detail">關鍵字：{connectionResult.event_keywords_sheet.detail}</p> : null}
        {connectionResult.glucose_standards_sheet.detail ? <p className="connection-detail">標準值：{connectionResult.glucose_standards_sheet.detail}</p> : null}
      </div> : null}
      <div className="config-meta">
        <p><span>Sheet ID</span><strong>{config?.sheet_id || '尚未設定'}</strong></p>
        <p><span>Sheet GID</span><strong>{config?.sheet_gid || '未指定'}</strong></p>
        <p><span>資料工作表</span><strong>{config?.sheet_name || 'Sheet1'}</strong></p>
        <p><span>事件關鍵字工作表</span><strong>{config?.event_keywords_sheet_name || DEFAULT_EVENT_KEYWORDS_SHEET_NAME}</strong></p>
        <p><span>血糖標準值工作表</span><strong>{config?.glucose_standards_sheet_name || DEFAULT_GLUCOSE_STANDARDS_SHEET_NAME}</strong></p>
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

  // 事件關鍵字設定：唯讀顯示（由 Google Sheet「事件關鍵字設定」工作表衍生）。
  const eventsPanel = <>
    <section className="side-section custom-events-card">
      <h3>🏷　事件關鍵字設定</h3>
      <p className="form-hint">事件關鍵字由 Google Sheet「事件關鍵字設定」工作表維護，每次載入儀表板或同步都會重新讀取。請直接在工作表中新增或刪除關鍵字。</p>
      <ul className="custom-event-list">
        {customEvents.map((c) => <li key={c.label}><span className="custom-event-label">{c.label}</span><span className="custom-event-threshold">{c.low_threshold}～{c.high_threshold}</span></li>)}
        {customEvents.length === 0 ? <li className="custom-event-empty">尚未連結 Sheet 或工作表無自訂關鍵字</li> : null}
      </ul>
    </section>
  </>;

  // 血糖標準值：唯讀顯示（由 Google Sheet「血糖標準值設定」工作表衍生）。
  const thresholdsPanel = <>
    <section className="side-section thresholds-card">
      <h3>⚖　血糖標準值</h3>
      <p className="form-hint">血糖標準值由 Google Sheet「血糖標準值設定」工作表維護，每次載入儀表板或同步都會重新讀取。趨勢圖與表格依此上色：超過上限為紅色、範圍內為綠色、低於下限為黃色。此設定不影響摘要統計。</p>
      <ul className="threshold-list">
        {eventThresholds.map((t) => <li key={t.label}>
          <span className="threshold-label">{t.label}</span>
          <div className="threshold-row">
            <span className="threshold-value low">下限 {t.low}</span>
            <span className="range-sep">～</span>
            <span className="threshold-value high">上限 {t.high}</span>
          </div>
        </li>)}
        {eventThresholds.length === 0 ? <li className="custom-event-empty">尚未連結 Sheet，載入內建預設中…</li> : null}
      </ul>
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
    return <><SummaryCards summary={data.summary} /><GlucoseTrendChart records={data.records} eventThresholds={eventThresholds} event={event || undefined} /><GlucoseRecordTable rows={data.table_rows} eventThresholds={eventThresholds} search={search} onSearch={setSearch} onExport={() => { window.location.href = exportUrl(selection, event || undefined, search || undefined); }} /></>;
  }, [configLoading, customEvents, eventThresholds, data, error, event, loading, selection, reload, search]);

  return <DashboardLayout sidebar={sidebar} onOpenSettings={() => setSettingsOpen(true)} theme={theme} onToggleTheme={toggleTheme}>{content}{settingsOpen ? <div className="settings-overlay" role="dialog" aria-modal="true" aria-label="設定"><div className="settings-modal"><div className="settings-modal-header"><h2>⚙　設定</h2><button className="settings-close" type="button" aria-label="關閉" onClick={() => setSettingsOpen(false)}>✕</button></div><div className="settings-modal-body">{settingsPanel}</div></div></div> : null}</DashboardLayout>;
}