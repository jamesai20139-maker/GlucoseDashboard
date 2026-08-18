export type EventName = string;
export type Classification = 'Low' | 'InRange' | 'High';

/// 自訂事件關鍵字（含使用者指定的閾值）。
export interface CustomEventConfig {
  label: string;
  low_threshold: number;
  high_threshold: number;
}

/// 單一事件的前端顯示標準範圍。趨勢圖與表格上色的唯一來源；
/// 不影響後端摘要統計（摘要仍用內建醫學標準）。
export interface EventThreshold {
  label: string;
  low: number;
  high: number;
}

export interface GlucoseRecord {
  source_row_number: number;
  measured_at: string;
  event: string;
  glucose_mg_dl: number;
  remark_1: string;
  remark_2: string;
}

export interface DashboardTableRow {
  source_row_number: number;
  measured_at: string | null;
  event: string | null;
  glucose_mg_dl: string | null;
  remark_1: string;
  remark_2: string;
}

export interface Summary {
  record_count: number;
  average_mg_dl: number | null;
  minimum_mg_dl: number | null;
  maximum_mg_dl: number | null;
  estimated_hba1c_percent: number | null;
  estimated_average_glucose_mg_dl: number | null;
  in_reference_percent: number | null;
  low_percent: number | null;
  high_percent: number | null;
}

export interface DashboardResponse {
  summary: Summary;
  records: GlucoseRecord[];
  table_rows: DashboardTableRow[];
  issues: { message_zh_tw: string; code: string }[];
  status: string;
  last_successful_sync_at: string | null;
  /// 即時衍生的自訂事件關鍵字（由「事件關鍵字設定」工作表）。
  custom_events: CustomEventConfig[];
  /// 即時衍生的血糖標準值（由「血糖標準值設定」工作表）。
  event_thresholds: EventThreshold[];
}

/// `/api/sync` 回應（與 DashboardResponse 不同：無 selection/table_rows）。
export interface SyncResponse {
  status: string;
  records: GlucoseRecord[];
  issues: { message_zh_tw: string; code: string }[];
  last_successful_sync_at: string | null;
  custom_events: CustomEventConfig[];
  event_thresholds: EventThreshold[];
}

export interface ConfigStatus {
  configured: boolean;
  credential_store: string;
  schema_version: number;
  sheet_id: string | null;
  sheet_gid: string | null;
  sheet_name: string | null;
  fixture_path: string | null;
  last_successful_sync_at: string | null;
  /// 「事件關鍵字設定」工作表名稱。
  event_keywords_sheet_name: string | null;
  /// 「血糖標準值設定」工作表名稱。
  glucose_standards_sheet_name: string | null;
  /// 即時衍生值（取自快取，首次同步前為空）；權威來源為 DashboardResponse/SyncResponse。
  custom_events: CustomEventConfig[];
  event_thresholds: EventThreshold[];
}

/// 單一工作表的連線測試報告。
export interface WorksheetConnectionReport {
  ok: boolean;
  sheet_name: string | null;
  url: string | null;
  http_status: number | null;
  row_count: number | null;
  header_valid: boolean;
  message: string;
  detail: string | null;
}

export interface ConnectionTestReport {
  status: string;
  ok: boolean;
  sheet_id: string | null;
  sheet_gid: string | null;
  data_sheet: WorksheetConnectionReport;
  event_keywords_sheet: WorksheetConnectionReport;
  glucose_standards_sheet: WorksheetConnectionReport;
}
