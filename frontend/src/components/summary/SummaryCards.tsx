import type { Summary } from '../../types';

function value(value: number | null, suffix = '') { return value === null ? '—' : `${value}${suffix}`; }

export function SummaryCards({ summary }: { summary: Summary }) {
  return <section className="summary-grid" aria-label="血糖摘要">
    <article className="metric-card blue"><span className="metric-icon">◉</span><div><b>平均血糖</b><strong>{value(summary.average_mg_dl)} <small>mg/dL</small></strong><span>目前選取區間</span></div></article>
    <article className="metric-card purple"><span className="metric-icon">♧</span><div><b>糖化血色素（估算）</b><strong>{value(summary.estimated_hba1c_percent, ' %')}</strong><span>eAG {value(summary.estimated_average_glucose_mg_dl)} mg/dL</span></div></article>
    <article className="metric-card green"><span className="metric-icon">◎</span><div><b>TIR（參考範圍內）</b><strong>{value(summary.in_reference_percent, ' %')}</strong><span>高 {value(summary.high_percent, '%')}　低 {value(summary.low_percent, '%')}</span></div></article>
  </section>;
}
