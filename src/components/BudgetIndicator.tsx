import type { DailyUsage, TodayStats } from '../lib/types';
import { useSettings } from '../hooks/useSettings';

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
  const { data: settings } = useSettings();
  const maxCost = settings?.max_daily_cost_yuan ?? 5;
  const currentCost = usage.estimated_cost_yuan;
  const percent = Math.min((currentCost / maxCost) * 100, 100);
  const isHighCost = percent >= 80;

  return (
    <div className="budget-panel" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '6px' }}>
      <div className="budget-indicator" style={{ display: 'flex', gap: 'var(--spacing-md)', fontSize: 'var(--font-size-md)', color: 'var(--color-text-muted)', alignItems: 'center' }}>
        <span className="budget-time">
          {stats.work_minutes}分钟 / {stats.total_minutes}分钟
        </span>
        <span className="budget-cost" style={{ color: isHighCost ? 'var(--color-error-light)' : 'var(--color-text)', fontWeight: 600 }}>
          估算 {formatEstimatedCost(currentCost)} / {maxCost} 元
        </span>
        <span className="budget-count">{usage.input_tokens + usage.output_tokens} tokens</span>
        <span className="budget-count">{stats.activity_count} 个活动</span>
      </div>

      <div className="budget-progress-container" style={{ width: '220px', height: '6px', backgroundColor: 'rgba(255, 255, 255, 0.06)', borderRadius: '3px', overflow: 'hidden' }}>
        <div
          className="budget-progress-bar"
          style={{
            width: `${percent}%`,
            height: '100%',
            backgroundColor: isHighCost ? 'var(--color-error)' : 'var(--color-primary)',
            transition: 'width 0.3s ease',
          }}
        />
      </div>

      {isHighCost && (
        <span className="budget-warning" style={{ fontSize: '11px', color: 'var(--color-error-light)', fontWeight: 500 }}>
          ⚠️ 费用已达今日预算的 {Math.round(percent)}%
        </span>
      )}
    </div>
  );
}
