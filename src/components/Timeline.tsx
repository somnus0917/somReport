import { useState, useMemo } from 'react';
import type { Activity, Category } from '../lib/types';
import { CATEGORY_COLORS, CATEGORIES } from '../lib/constants';
import ActivityCard from './ActivityCard';
import { elapsedSeconds } from '../lib/datetime';

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
    return sum + elapsedSeconds(a.started_at, a.ended_at) / 60;
  }, 0);
}

export default function Timeline({ activities }: Props) {
  const [disabledCategories, setDisabledCategories] = useState<Set<Category>>(new Set());

  const categoriesInActivities = useMemo(() => {
    return Array.from(new Set(activities.map((a) => a.category)));
  }, [activities]);

  if (activities.length === 0) {
    return <div className="timeline-empty">今天还没有记录任何活动。</div>;
  }

  const toggleCategory = (cat: Category) => {
    setDisabledCategories((prev) => {
      const next = new Set(prev);
      if (next.has(cat)) {
        next.delete(cat);
      } else {
        next.add(cat);
      }
      return next;
    });
  };

  const handleSelectAll = () => {
    setDisabledCategories(new Set());
  };

  const handleSelectNone = () => {
    setDisabledCategories(new Set(categoriesInActivities));
  };

  const grouped = groupByCategory(activities);
  const failed = activities.filter((a) => a.confidence < 0.3);

  const filteredActivities = activities.filter(
    (a) => !disabledCategories.has(a.category)
  );

  return (
    <div className="timeline">
      {failed.length > 0 && (
        <div className="timeline-warning">
          {failed.length} 个活动置信度过低
        </div>
      )}

      <div className="timeline-filter-section">
        <div className="filter-header">
          <span className="filter-title">分类筛选</span>
          <div className="filter-actions">
            <button onClick={handleSelectAll} className="filter-action-btn">
              全选
            </button>
            <span className="filter-separator">/</span>
            <button onClick={handleSelectNone} className="filter-action-btn">
              清空
            </button>
          </div>
        </div>

        <div className="timeline-summary">
          {[...grouped.entries()].map(([cat, acts]) => {
            const isDisabled = disabledCategories.has(cat);
            const color = CATEGORY_COLORS[cat];
            return (
              <button
                key={cat}
                onClick={() => toggleCategory(cat)}
                className={`category-chip-btn ${isDisabled ? 'disabled' : 'enabled'}`}
                style={{
                  borderColor: isDisabled ? 'var(--color-border)' : color,
                  backgroundColor: isDisabled ? 'transparent' : `${color}15`,
                  color: isDisabled ? 'var(--color-text-muted)' : 'var(--color-text)',
                }}
              >
                <span
                  className="category-dot"
                  style={{
                    backgroundColor: isDisabled ? 'var(--color-text-dim)' : color,
                  }}
                />
                {CATEGORIES.find((c) => c.value === cat)?.label || cat}:{' '}
                {Math.round(categoryTotalMinutes(acts))}分钟
              </button>
            );
          })}
        </div>
      </div>

      <div className="timeline-list">
        {filteredActivities.length === 0 ? (
          <div className="timeline-empty-filtered">
            没有与筛选条件匹配的活动。
          </div>
        ) : (
          filteredActivities.map((a) => (
            <ActivityCard key={a.id} activity={a} />
          ))
        )}
      </div>
    </div>
  );
}
