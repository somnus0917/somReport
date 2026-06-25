import type { DailyUsage, TodayStats } from '../lib/types';

interface Props {
  stats: TodayStats;
  usage: DailyUsage;
}

function formatEstimatedCost(yuan: number) {
  if (yuan === 0) return '0 元';
  if (yuan < 0.01) return `${yuan.toFixed(6)} 元`;
  return `${yuan.toFixed(4)} 元`;
}

export default function BudgetIndicator({ stats, usage }: Props) {
  return (
    <div className="budget-indicator">
      <span className="budget-time">
        {stats.work_minutes}分钟 / {stats.total_minutes}分钟
      </span>
      <span className="budget-cost">估算 {formatEstimatedCost(usage.estimated_cost_yuan)}</span>
      <span className="budget-count">{usage.input_tokens + usage.output_tokens} tokens</span>
      <span className="budget-count">{stats.activity_count} 个活动</span>
    </div>
  );
}
