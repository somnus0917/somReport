export type Category =
  | 'development'
  | 'meeting'
  | 'communication'
  | 'documentation'
  | 'research'
  | 'design'
  | 'other';

export const CATEGORY_LABELS: Record<Category, string> = {
  development: '开发',
  meeting: '会议',
  communication: '沟通',
  documentation: '文档',
  research: '研究',
  design: '设计',
  other: '其他',
};

export type RecordingState = 'stopped' | 'recording' | 'paused';

export type PeriodType = 'daily' | 'weekly' | 'custom';

export interface Activity {
  id: string;
  job_id: string;
  started_at: string;
  ended_at: string;
  category: Category;
  summary: string;
  detail: string | null;
  confidence: number;
  is_work_related: boolean;
  source: string;
  edited_at: string | null;
  deleted_at: string | null;
}

export interface Report {
  id: string;
  period_type: PeriodType;
  period_start: string;
  period_end: string;
  template_id: string | null;
  title: string;
  content_markdown: string;
  model: string | null;
  prompt_version: string | null;
  created_at: string;
  updated_at: string;
}

export interface TodayStats {
  total_minutes: number;
  work_minutes: number;
  activity_count: number;
}

export interface DailyUsage {
  input_tokens: number;
  output_tokens: number;
  estimated_cost_yuan: number;
}

export interface ProviderConfig {
  name: string;
  api_key_env_var: string | null;
  api_key: string | null;
  api_url: string;
  model: string;
  input_cost_per_million_yuan: number;
  output_cost_per_million_yuan: number;
}

export interface AppSettings {
  vision_provider: ProviderConfig;
  text_provider: ProviderConfig;
  capture_interval_secs: number;
  idle_timeout_secs: number;
  max_daily_cost_yuan: number;
  auto_start: boolean;
  notify_on_report: boolean;
  data_retention_days: number;
  vision_connection: ModelConnectionStatus;
  text_connection: ModelConnectionStatus;
}

export interface ModelConnectionStatus {
  success: boolean | null;
  tested_at: string | null;
  message: string | null;
}

export interface UpdateActivityRequest {
  id: string;
  summary?: string;
  detail?: string | null;
  category?: string;
  is_work_related?: boolean;
  confidence?: number;
}
