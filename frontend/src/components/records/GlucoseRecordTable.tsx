import type { GlucoseRecord } from '../../types';

export function GlucoseRecordTable({ records, search, onSearch, onExport }: { records: GlucoseRecord[]; search: string; onSearch: (value: string) => void; onExport: () => void }) {
  return <section className="panel records-panel" aria-label="血糖紀錄">
    <header className="panel-title"><h2>◉ 血糖紀錄</h2><div className="table-tools"><input aria-label="搜尋紀錄" value={search} onChange={(event: { target: { value: string } }) => onSearch(event.target.value)} placeholder="搜尋…" /><button onClick={onExport}>⇩ 匯出 CSV</button></div></header>
    {records.length === 0 ? <div className="empty">沒有符合搜尋條件的紀錄</div> : <div className="table-scroll"><table><thead><tr><th>血糖量測日期時間</th><th>事件</th><th>量測血糖值 (mg/dL)</th><th>備註1</th><th>備註2</th></tr></thead><tbody>{records.map((record) => <tr key={`${record.source_row_number}-${record.measured_at}`}><td>{record.measured_at.replace('T', ' ').replace('Z', '').slice(0, 16)}</td><td>{record.event}</td><td><span className={record.glucose_mg_dl >= 140 || record.glucose_mg_dl < 70 ? 'glucose high' : 'glucose normal'}>{record.glucose_mg_dl}</span></td><td>{record.remark_1 || '-'}</td><td>{record.remark_2 || '-'}</td></tr>)}</tbody></table></div>}
  </section>;
}
