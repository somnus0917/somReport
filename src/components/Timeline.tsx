import type { Activity, Category } from '../lib/types';
import { CATEGORY_COLORS } from '../lib/constants';
import ActivityCard from './ActivityCard';

interface Props {
  activities: Activity[];
}

function groupByCategory(activities: Activity[]): Map<Category, Activity[]> {
  const map = new Map<Category, Activity[]>();
  for (const a of activities) {
    const list = map.get(a.category) ?? [];
    list.push(a);
    map.set(a.category, list);
  }
  return map;
}

function categoryTotalMinutes(activities: Activity[]): number {
  return activities.reduce((sum, a) => {
    return sum + (new Date(a.ended_at).getTime() - new Date(a.started_at).getTime()) / 60000;
  }, 0);
}

export default function Timeline({ activities }: Props) {
  if (activities.length === 0) {
    return <div className="timeline-empty">今天还没有记录任何活动。</div>;
  }

  const grouped = groupByCategory(activities);
  const failed = activities.filter((a) => a.confidence < 0.3);

  return (
    <div className="timeline">
      {failed.length > 0 && (
        <div className="timeline-warning">
          {failed.length} 个活动置信度过低
        </div>
      )}

      <div className="timeline-summary">
        {[...grouped.entries()].map(([cat, acts]) => (
          <span
            key={cat}
            className="category-chip"
            style={{ borderColor: CATEGORY_COLORS[cat] }}
          >
            {cat}: {Math.round(categoryTotalMinutes(acts))}m
          </span>
        ))}
      </div>

      <div className="timeline-list">
        {activities.map((a) => (
          <ActivityCard key={a.id} activity={a} />
        ))}
      </div>
    </div>
  );
}
