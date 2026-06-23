import { useRecordingStore } from '../stores/recording';

export default function BudgetIndicator() {
  const { stats, dailyCostCents } = useRecordingStore();

  return (
    <div className="budget-indicator">
      <span className="budget-time">
        {stats.work_minutes}m / {stats.total_minutes}m
      </span>
      <span className="budget-cost">${(dailyCostCents / 100).toFixed(2)}</span>
      <span className="budget-count">{stats.activity_count} activities</span>
    </div>
  );
}
