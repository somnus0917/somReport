import { useEffect } from "react";
import { useRecordingStore } from "../stores/recording";
import { useToday, useDailyUsage } from "../hooks/useRecording";
import CaptureToggle from "../components/CaptureToggle";
import StatusBadge from "../components/StatusBadge";
import BudgetIndicator from "../components/BudgetIndicator";
import Timeline from "../components/Timeline";
import ActivityHeatmap from "../components/ActivityHeatmap";
import { LoadingState } from "../components/StateViews";
import { showFloatingWidget } from "../api/tauri";
export default function Today() {
  const { subscribe, error } = useRecordingStore();
  const { data: todayData, isLoading: todayLoading } = useToday();
  const { data: dailyUsage } = useDailyUsage();

  useEffect(() => {
    const unsub = subscribe();
    return unsub;
  }, [subscribe]);

  const activities = todayData?.[0] ?? [];
  const stats = todayData?.[1] ?? {
    total_minutes: 0,
    work_minutes: 0,
    activity_count: 0,
  };
  const usage = dailyUsage ?? {
    input_tokens: 0,
    output_tokens: 0,
    estimated_cost_yuan: 0,
  };

  return (
    <div className="today-page">
      <header className="page-header">
        <p className="page-kicker">daily overview</p>
        <h2>今日</h2>
        <p>实时追踪工作活动，AI 自动分析截图内容。</p>
      </header>
      <header className="today-header">
        <div className="today-header-left">
          <div className="today-capture-panel">
            <div className="today-capture-row">
              <CaptureToggle />
              <StatusBadge />
              <button
                className="btn-sm"
                onClick={() => void showFloatingWidget()}
              >
                悬浮窗
              </button>
            </div>
            {error && <p className="capture-error">{error}</p>}
          </div>
        </div>
        <BudgetIndicator stats={stats} usage={usage} />
      </header>
      <main className="today-main">
        {todayLoading ? (
          <LoadingState />
        ) : (
          <>
            <ActivityHeatmap activities={activities} />
            <Timeline activities={activities} />
          </>
        )}
      </main>
    </div>
  );
}
