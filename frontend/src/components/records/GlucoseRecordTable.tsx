import { classifyByThreshold } from '../../lib/classify';
import type { DashboardTableRow, EventThreshold } from '../../types';

function cell(value: string | null) {
  return value === null ? <span className="type-error">Type Error</span> : value;
}

export function GlucoseRecordTable({ rows, eventThresholds, search, onSearch, onExport }: { rows: DashboardTableRow[]; eventThresholds: EventThreshold[]; search: string; onSearch: (value: string) => void; onExport: () => void }) {
  return <section className="panel records-panel" aria-label="血糖紀錄">
    <header className="panel-title"><h2>◉ 血糖紀錄</h2><div className="table-tools"><input aria-label="搜尋紀錄" value={search} onChange={(event: { target: { value: string } }) => onSearch(event.target.value)} placeholder="搜尋…" /><button onClick={onExport}>⇩ 匯出 CSV</button></div></header>
    {rows.length === 0 ? <div className="empty">沒有符合搜尋條件的紀錄</div> : <div className="table-scroll"><table><thead><tr><th>血糖量測日期時間</th><th>事件</th><th>量測血糖值 (mg/dL)</th><th>備註1</th><th>備註2</th></tr></thead><tbody>{rows.map((row) => {
      const glucoseValue = row.glucose_mg_dl === null ? null : Number(row.glucose_mg_dl);
      const status = row.event !== null && glucoseValue !== null && !Number.isNaN(glucoseValue) ? classifyByThreshold(row.event, glucoseValue, eventThresholds) : null;
      return <tr key={row.source_row_number}><td>{cell(row.measured_at)}</td><td>{cell(row.event)}</td><td>{row.glucose_mg_dl === null ? <span className="type-error">Type Error</span> : <span className={`glucose ${status}`}>{row.glucose_mg_dl}</span>}</td><td>{row.remark_1 || '-'}</td><td>{row.remark_2 || '-'}</td></tr>;
    })}</tbody></table></div>}
  </section>;
}