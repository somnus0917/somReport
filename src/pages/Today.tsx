import { useEffect } from 'react';
import { useRecordingStore } from '../stores/recording';
import CaptureToggle from '../components/CaptureToggle';
import StatusBadge from '../components/StatusBadge';
import BudgetIndicator from '../components/BudgetIndicator';
import Timeline from '../components/Timeline';

export default function Today() {
  const { activities, fetchToday, fetchRecordingState, fetchDailyUsage, subscribe } =
    useRecordingStore();

  useEffect(() => {
    const unsub = subscribe();
    Promise.all([fetchToday(), fetchRecordingState(), fetchDailyUsage()]).then(() => {
      useRecordingStore.setState({ loading: false });
    });
    return unsub;
  }, [fetchToday, fetchRecordingState, fetchDailyUsage, subscribe]);

  const loadingState = useRecordingStore((s) => s.loading);

  return (
    <div className="today-page">
      <header className="today-header">
        <div className="today-header-left">
          <CaptureToggle />
          <StatusBadge />
        </div>
        <BudgetIndicator />
      </header>
      <main className="today-main">
        {loadingState ? <p>Loading...</p> : <Timeline activities={activities} />}
      </main>
    </div>
  );
}
