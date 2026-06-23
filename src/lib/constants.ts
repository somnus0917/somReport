import type { AppSettings, Category } from './types';

export const CATEGORIES: { value: Category; label: string }[] = [
  { value: 'development', label: 'Development' },
  { value: 'meeting', label: 'Meeting' },
  { value: 'communication', label: 'Communication' },
  { value: 'documentation', label: 'Documentation' },
  { value: 'research', label: 'Research' },
  { value: 'design', label: 'Design' },
  { value: 'other', label: 'Other' },
];

export const CATEGORY_COLORS: Record<Category, string> = {
  development: '#4f46e5',
  meeting: '#f59e0b',
  communication: '#10b981',
  documentation: '#8b5cf6',
  research: '#3b82f6',
  design: '#ec4899',
  other: '#6b7280',
};

export const TEMPLATES = [
  { id: 'default', label: 'Default', description: 'Concise daily report grouped by category' },
  { id: 'detailed', label: 'Detailed', description: 'Full breakdown with confidence scores' },
  { id: 'executive', label: 'Executive', description: 'Brief high-level summary' },
];

export const PROVIDERS = [
  { id: 'openai', label: 'OpenAI' },
  { id: 'anthropic', label: 'Anthropic' },
];

export const DEFAULT_SETTINGS: AppSettings = {
  vision_provider: {
    name: 'openai',
    api_key_env_var: 'OPENAI_API_KEY',
    api_key: null,
    api_url: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini',
  },
  text_provider: {
    name: 'openai',
    api_key_env_var: 'OPENAI_API_KEY',
    api_key: null,
    api_url: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini',
  },
  capture_interval_secs: 30,
  idle_timeout_secs: 300,
  max_daily_cost_cents: 500,
  auto_start: false,
  notify_on_report: true,
};
