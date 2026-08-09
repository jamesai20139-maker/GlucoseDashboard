import { classifyByThreshold } from '../../lib/classify';
import type { EventThreshold, GlucoseRecord } from '../../types';

export function GlucoseTrendChart({ records, eventThresholds = [] }: { records: GlucoseRecord[]; eventThresholds?: EventThreshold[] }) {
  const points = records.slice().sort((a, b) => a.measured_at.localeCompare(b.measured_at));
  const max = Math.max(200, ...points.map((record) => record.glucose_mg_dl));
  return <section className="panel chart-panel" aria-label="血糖趨勢圖">
    <header className="panel-title"><h2>♧ 血糖趨勢圖</h2></header>
    {points.length === 0 ? <div className="empty">此區間沒有有效血糖紀錄</div> : <div className="chart-wrap">
      <div className="chart-zones"><span className="high-zone" /><span className="normal-zone" /><span className="low-zone" /></div>
      <svg viewBox="0 0 900 300" role="img" aria-label="血糖趨勢">
        <line x1="40" y1="250" x2="860" y2="250" className="axis" />
        <line x1="40" y1="80" x2="860" y2="80" className="threshold" />
        <polyline fill="none" points={points.map((record, index) => `${50 + index * (800 / Math.max(1, points.length - 1))},${250 - (record.glucose_mg_dl / max) * 190}`).join(' ')} className="trend-line" />
        {points.map((record, index) => { const x = 50 + index * (800 / Math.max(1, points.length - 1)); const y = 250 - (record.glucose_mg_dl / max) * 190; const status = classifyByThreshold(record.event, record.glucose_mg_dl, eventThresholds); return <circle key={`${record.source_row_number}-${record.measured_at}`} cx={x} cy={y} r="6" className={`point ${status}`}><title>{record.measured_at} {record.event} {record.glucose_mg_dl} mg/dL</title></circle>; })}
      </svg>
      <div className="legend"><span><i className="swatch high" />偏高（依事件標準）</span><span><i className="swatch normal" />參考範圍</span><span><i className="swatch low" />偏低（依事件標準）</span><span><i className="dot-alert" />異常警示</span></div>
    </div>}
  </section>;
}