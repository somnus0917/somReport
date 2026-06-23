export type Category =
  | 'development'
  | 'meeting'
  | 'communication'
  | 'documentation'
  | 'research'
  | 'design'
  | 'other';

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

export interface ProviderConfig {
  name: string;
  api_key_env_var: string | null;
  api_key: string | null;
  api_url: string;
  model: string;
}

export interface AppSettings {
  vision_provider: ProviderConfig;
  text_provider: ProviderConfig;
  capture_interval_secs: number;
  idle_timeout_secs: number;
  max_daily_cost_cents: number;
  auto_start: boolean;
  notify_on_report: boolean;
}

export interface UpdateActivityRequest {
  id: string;
  summary?: string;
  detail?: string | null;
  category?: string;
  is_work_related?: boolean;
  confidence?: number;
}
