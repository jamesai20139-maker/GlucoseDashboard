export type EventName = string;
export type Classification = 'Low' | 'InRange' | 'High';

/// 自訂事件關鍵字（含使用者指定的閾值）。
export interface CustomEventConfig {
  label: string;
  low_threshold: number;
  high_threshold: number;
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
  custom_events: CustomEventConfig[];
}

export interface ConnectionTestReport {
  status: string;
  ok: boolean;
  sheet_id: string | null;
  sheet_gid: string | null;
  sheet_name: string | null;
  url: string | null;
  http_status: number | null;
  record_count: number | null;
  issue_count: number | null;
  message: string;
  detail: string | null;
}
