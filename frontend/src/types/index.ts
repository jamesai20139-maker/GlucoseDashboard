export type EventName = '空腹血糖' | '午餐前' | '午餐後' | '晚餐前' | '晚餐後' | '睡前';
export type Classification = 'Low' | 'InRange' | 'High';

export interface GlucoseRecord {
  source_row_number: number;
  measured_at: string;
  event: EventName;
  glucose_mg_dl: number;
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
  issues: { message_zh_tw: string; code: string }[];
  status: string;
  last_successful_sync_at: string | null;
}

export interface ConfigStatus {
  configured: boolean;
  credential_store: string;
  schema_version: number;
  sheet_id: string | null;
  sheet_name: string | null;
  fixture_path: string | null;
  last_successful_sync_at: string | null;
}
