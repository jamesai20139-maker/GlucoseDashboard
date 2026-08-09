import { classifyByThreshold } from '../../lib/classify';
import type { EventThreshold, GlucoseRecord } from '../../types';

/// 趨勢圖座標常數。Y 軸對應血糖值：v → y = AXIS_Y - (v / scaleMax) * SCALE，
/// scaleMax 至少 200 mg/dL 並無條件進位到 50 的倍數，方便刻度標示。
/// X 軸採「固定點距 + 水平捲動」:每筆記錄固定間距 STEP,SVG 寬度隨資料筆數增長,
/// 點數多時可左右捲動查看全部資料,不再把所有點擠進固定寬度。
const AXIS_Y = 250;       // X 軸 y 座標
const TOP_Y = 60;         // 繪圖區頂部
const LEFT_PAD = 58;      // 左側留白:Y 軸標題 + 刻度
const RIGHT_PAD = 10;     // 右側留白
const SCALE = AXIS_Y - TOP_Y; // 像素/單位換算高度
const STEP = 64;          // 每筆記錄的固定 X 間距(像素)
const MIN_PLOT = 760;     // 繪圖區最小寬度,資料少時仍填滿面板
const RIGHT_BREATH = 120; // 最後一點右側留白,避免貼邊
const LOW_BG_UNIVERSAL = 70; // 通用低血糖門檻,用於「全部」模式

type Status = 'high' | 'normal' | 'low';

export function GlucoseTrendChart({ records, eventThresholds = [], event }: {
  records: GlucoseRecord[];
  eventThresholds?: EventThreshold[];
  event?: string;
}) {
  const points = records.slice().sort((a, b) => a.measured_at.localeCompare(b.measured_at));
  const n = points.length;
  const scaleMax = Math.ceil(Math.max(200, ...points.map((r) => r.glucose_mg_dl)) / 50) * 50;
  const plotWidth = Math.max(MIN_PLOT, (n - 1) * STEP + RIGHT_BREATH);
  const svgWidth = LEFT_PAD + plotWidth + RIGHT_PAD;
  const plotLeft = LEFT_PAD;
  const plotRight = plotLeft + plotWidth;
  const xAt = (i: number) => plotLeft + i * STEP;
  const yAt = (value: number) => AXIS_Y - (value / scaleMax) * SCALE;
  const clampY = (y: number) => Math.max(TOP_Y, Math.min(AXIS_Y, y));

  // 選定事件的標準值;「全部」模式不畫單一事件色帶,僅保留通用低血糖 70 線。
  const eventThreshold = event ? eventThresholds.find((t) => t.label === event) : undefined;
  const lowValue = eventThreshold ? eventThreshold.low : LOW_BG_UNIVERSAL;
  const highValue = eventThreshold ? eventThreshold.high : null;
  const lowY = clampY(yAt(lowValue));
  const highY = highValue !== null ? clampY(yAt(highValue)) : null;

  // Y 軸刻度:每 50 mg/dL 一條,從 0 到 scaleMax。
  const yTicks: number[] = [];
  for (let v = 0; v <= scaleMax; v += 50) yTicks.push(v);

  const statusOf = (record: GlucoseRecord): Status =>
    classifyByThreshold(record.event, record.glucose_mg_dl, eventThresholds);
  const statusLabel = (status: Status) => (status === 'high' ? '偏高' : status === 'low' ? '偏低' : '參考範圍');

  return <section className="panel chart-panel" aria-label="血糖趨勢圖">
    <header className="panel-title"><h2>♧ 血糖趨勢圖</h2></header>
    {n === 0 ? <div className="empty">此區間沒有有效血糖紀錄</div> : <div className="chart-wrap">
      <div className="chart-scroll">
        <svg viewBox={`0 0 ${svgWidth} 300`} width={svgWidth} height={300} role="img" aria-label="血糖趨勢">
          {/* 依事件標準值的背景色帶(僅篩選單一事件時繪製) */}
          {eventThreshold && highY !== null ? <>
            <rect x={plotLeft} y={TOP_Y} width={plotWidth} height={highY - TOP_Y} className="zone-high" />
            <rect x={plotLeft} y={highY} width={plotWidth} height={lowY - highY} className="zone-normal" />
            <rect x={plotLeft} y={lowY} width={plotWidth} height={AXIS_Y - lowY} className="zone-low" />
          </> : null}
          {/* Y 軸標題(旋轉) */}
          <text transform={`translate(14 ${(TOP_Y + AXIS_Y) / 2}) rotate(-90)`} textAnchor="middle" className="axis-title">量測血糖值 (mg/dL)</text>
          {/* Y 軸刻度與水平格線 */}
          {yTicks.map((v) => {
            const y = clampY(yAt(v));
            return <g key={`yt-${v}`}>
              <line x1={plotLeft} y1={y} x2={plotRight} y2={y} className="grid" />
              <text x={plotLeft - 6} y={y + 4} textAnchor="end" className="tick-label">{v}</text>
            </g>;
          })}
          {/* X 軸基準線 */}
          <line x1={plotLeft} y1={AXIS_Y} x2={plotRight} y2={AXIS_Y} className="axis" />
          {/* 事件高標準門檻線 */}
          {eventThreshold && highY !== null ? <>
            <line x1={plotLeft} y1={highY} x2={plotRight} y2={highY} className="threshold high" />
            <text x={plotRight - 4} y={highY - 6} textAnchor="end" className="threshold-label">{highValue} mg/dL</text>
          </> : null}
          {/* 低標準門檻線:單一事件用事件 low;「全部」模式用通用 70 */}
          <line x1={plotLeft} y1={lowY} x2={plotRight} y2={lowY} className="threshold low" />
          <text x={plotRight - 4} y={lowY - 6} textAnchor="end" className="threshold-label">{lowValue} mg/dL</text>
          {/* 趨勢折線:單一顏色連線,顏色僅標示於事件節點 */}
          <polyline fill="none" points={points.map((record, i) => `${xAt(i)},${yAt(record.glucose_mg_dl)}`).join(' ')} className="trend-line" />
          {/* 資料點 + 節點上方血糖值標籤 + X 軸日期時間標籤 */}
          {points.map((record, index) => {
            const status = statusOf(record);
            const cx = xAt(index);
            const cy = yAt(record.glucose_mg_dl);
            return <g key={`${record.source_row_number}-${record.measured_at}`}>
              <text x={cx} y={cy - 11} textAnchor="middle" className="value-label">{record.glucose_mg_dl}</text>
              <circle cx={cx} cy={cy} r="6" className={`point ${status}`}><title>{record.measured_at} {record.event} {record.glucose_mg_dl} mg/dL（{statusLabel(status)}）</title></circle>
            </g>;
          })}
        </svg>
      </div>
      <div className="legend"><span><i className="swatch high" />偏高（依事件標準）</span><span><i className="swatch normal" />參考範圍</span><span><i className="swatch low" />偏低（依事件標準）</span><span><i className="dot-alert" />異常警示</span></div>
    </div>}
  </section>;
}