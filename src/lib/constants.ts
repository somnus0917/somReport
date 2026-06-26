import type { AppSettings, Category, ProviderConfig } from "./types";

export const CATEGORIES: { value: Category; label: string }[] = [
  { value: "development", label: "开发" },
  { value: "meeting", label: "会议" },
  { value: "communication", label: "沟通" },
  { value: "documentation", label: "文档" },
  { value: "research", label: "调研" },
  { value: "design", label: "设计" },
  { value: "other", label: "其他" },
];

export const CATEGORY_COLORS: Record<Category, string> = {
  development: "#4f46e5",
  meeting: "#f59e0b",
  communication: "#10b981",
  documentation: "#8b5cf6",
  research: "#3b82f6",
  design: "#ec4899",
  other: "#6b7280",
};

export const TEMPLATES = [
  { id: "default", label: "默认", description: "按类别分组的简洁日报" },
  { id: "detailed", label: "详细", description: "包含置信度分数的完整报告" },
  { id: "executive", label: "摘要", description: "简短的高层级总结" },
];

export const PROVIDERS = [
  { id: "openai", label: "OpenAI" },
  { id: "qwen", label: "通义千问 (DashScope)" },
  { id: "anthropic", label: "Anthropic" },
];

export function providerDefaults(
  role: "vision" | "text",
  provider: string,
): Pick<
  ProviderConfig,
  | "api_url"
  | "model"
  | "api_key_env_var"
  | "api_key"
  | "input_cost_per_million_yuan"
  | "output_cost_per_million_yuan"
> {
  if (provider === "anthropic") {
    return {
      api_url: "https://api.anthropic.com",
      model: "claude-sonnet-4-20250514",
      api_key_env_var: "ANTHROPIC_API_KEY",
      api_key: null,
      input_cost_per_million_yuan: 3,
      output_cost_per_million_yuan: 15,
    };
  }
  if (provider === "qwen") {
    return {
      api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
      model: role === "vision" ? "qwen3.5-flash" : "qwen3.5-flash",
      api_key_env_var: "QWEN_API_KEY",
      api_key: null,
      input_cost_per_million_yuan: 0.2,
      output_cost_per_million_yuan: 1.2,
    };
  }
  return {
    api_url: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    api_key_env_var: "OPENAI_API_KEY",
    api_key: null,
    input_cost_per_million_yuan: 0.15,
    output_cost_per_million_yuan: 0.6,
  };
}

export const DEFAULT_SETTINGS: AppSettings = {
  vision_provider: {
    name: "qwen",
    api_key_env_var: "QWEN_API_KEY",
    api_key: null,
    api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen3.5-flash",
    input_cost_per_million_yuan: 0.2,
    output_cost_per_million_yuan: 1.2,
  },
  text_provider: {
    name: "qwen",
    api_key_env_var: "QWEN_API_KEY",
    api_key: null,
    api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen3.5-plus",
    input_cost_per_million_yuan: 0.2,
    output_cost_per_million_yuan: 1.2,
  },
  capture_interval_secs: 30,
  idle_timeout_secs: 300,
  max_daily_cost_yuan: 5,
  auto_start: false,
  notify_on_report: true,
  data_retention_days: 30,
  vision_connection: {
    success: null,
    tested_at: null,
    message: null,
  },
  text_connection: {
    success: null,
    tested_at: null,
    message: null,
  },
};
