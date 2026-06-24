import type { DailyUsage, TodayStats } from '../lib/types';

interface Props {
  stats: TodayStats;
  usage: DailyUsage;
}

function formatEstimatedCost(cents: number) {
  if (cents === 0) return '0 分';
  if (cents < 1) return `${cents.toFixed(4)} 分`;
  return `${(cents / 100).toFixed(4)}（${cents.toFixed(2)} 分）`;
}

export default function BudgetIndicator({ stats, usage }: Props) {
  return (
    <div className="budget-indicator">
      <span className="budget-time">
        {stats.work_minutes}分钟 / {stats.total_minutes}分钟
      </span>
      <span className="budget-cost">估算 {formatEstimatedCost(usage.estimated_cost_cents)}</span>
      <span className="budget-count">{usage.input_tokens + usage.output_tokens} tokens</span>
      <span className="budget-count">{stats.activity_count} 个活动</span>
    </div>
  );
}
