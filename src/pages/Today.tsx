import { useEffect } from 'react';
import { useRecordingStore } from '../stores/recording';
import { useToday, useDailyUsage } from '../hooks/useRecording';
import CaptureToggle from '../components/CaptureToggle';
import StatusBadge from '../components/StatusBadge';
import BudgetIndicator from '../components/BudgetIndicator';
import Timeline from '../components/Timeline';
import { LoadingState } from '../components/StateViews';

export default function Today() {
  const { subscribe } = useRecordingStore();
  const { data: todayData, isLoading: todayLoading } = useToday();
  const { data: dailyUsage } = useDailyUsage();

  useEffect(() => {
    const unsub = subscribe();
    return unsub;
  }, [subscribe]);

  const activities = todayData?.[0] ?? [];
  const stats = todayData?.[1] ?? { total_minutes: 0, work_minutes: 0, activity_count: 0 };
  const usage = dailyUsage ?? { input_tokens: 0, output_tokens: 0, estimated_cost_cents: 0 };

  return (
    <div className="today-page">
      <header className="today-header">
        <div className="today-header-left">
          <CaptureToggle />
          <StatusBadge />
        </div>
        <BudgetIndicator stats={stats} usage={usage} />
      </header>
      <main className="today-main">
        {todayLoading ? <LoadingState /> : <Timeline activities={activities} />}
      </main>
    </div>
  );
}
