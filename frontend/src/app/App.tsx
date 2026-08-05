import { useMemo, useState } from 'react';
import { DashboardLayout } from '../components/layout/DashboardLayout';
import { SummaryCards } from '../components/summary/SummaryCards';
import { GlucoseTrendChart } from '../components/trend/GlucoseTrendChart';
import { GlucoseRecordTable } from '../components/records/GlucoseRecordTable';
import { exportUrl } from '../services/local_service';
import { useDashboard } from '../state/dashboard_store';

const filters = [['', '全部'], ['空腹血糖', '空腹血糖'], ['午餐前', '午餐前'], ['午餐後', '午餐後'], ['晚餐前', '晚餐前'], ['晚餐後', '晚餐後'], ['睡前', '睡前']];

export default function App() {
  const [period, setPeriod] = useState('all');
  const [event, setEvent] = useState('');
  const [search, setSearch] = useState('');
  const { data, loading, error, reload } = useDashboard(period, event || undefined, search || undefined);
  const sidebar = <>
    <section className="side-section"><h3>▣　時間區間</h3><div className="periods">{[['all', '日'], ['week', '週'], ['month', '月'], ['quarter', '季']].map(([key, label]) => <button key={key} className={period === key ? 'active' : ''} onClick={() => setPeriod(key)}>{label}</button>)}</div><label className="date-label">▣　自訂日期<input type="text" placeholder="2026/07/07 ～ 2026/07/07" /></label><button className="refresh" onClick={() => void reload()}>⟳　立即更新</button><p className="sync-status"><i /> {data?.last_successful_sync_at ? `資料已更新：${data.last_successful_sync_at.slice(0, 16).replace('T', ' ')}` : '等待資料同步'}</p></section>
    <section className="side-section"><h3>⚱　篩選項目</h3>{filters.map(([key, label]) => <label className={`radio-row ${event === key ? 'selected' : ''}`} key={key}><input type="radio" name="event" checked={event === key} onChange={() => setEvent(key)} />{label}</label>)}</section>
    <section className="side-section reference"><h3>ⓘ　血糖標準值</h3><p>● 正常人空腹 8 小時後，70～99 mg/dL</p><p>● 正常人餐前（距離上一餐至少 4 小時），70～100 mg/dL</p><p className="warning">● 餐後 2 小時所測量，≥140 mg/dL 視為偏高</p></section>
  </>;
  const content = useMemo(() => {
    if (loading) return <div className="state-card">正在讀取血糖資料…</div>;
    if (error) return <div className="state-card error-state"><h2>同步失敗</h2><p>{error}</p><button onClick={() => void reload()}>重新嘗試</button></div>;
    if (!data) return <div className="state-card"><h2>尚未設定資料來源</h2><p>請先使用 glucose-dashboard config 完成設定。</p></div>;
    return <><SummaryCards summary={data.summary} /><GlucoseTrendChart records={data.records} /><GlucoseRecordTable records={data.records} search={search} onSearch={setSearch} onExport={() => { window.location.href = exportUrl(period, event || undefined, search || undefined); }} /></>;
  }, [data, error, event, loading, period, reload, search]);
  return <DashboardLayout sidebar={sidebar}>{content}</DashboardLayout>;
}
