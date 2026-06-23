import { useState } from 'react';
import type { Activity, Category } from '../lib/types';
import { CATEGORIES, CATEGORY_COLORS } from '../lib/constants';
import { useRecordingStore } from '../stores/recording';

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function formatDuration(start: string, end: string): string {
  const ms = new Date(end).getTime() - new Date(start).getTime();
  const mins = Math.round(ms / 60000);
  if (mins < 60) return `${mins}m`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h}h ${m}m` : `${h}h`;
}

interface Props {
  activity: Activity;
}

export default function ActivityCard({ activity }: Props) {
  const { updateActivityItem, deleteActivityItem } = useRecordingStore();
  const [editing, setEditing] = useState(false);
  const [summary, setSummary] = useState(activity.summary);
  const [category, setCategory] = useState(activity.category);

  const save = async () => {
    await updateActivityItem({
      id: activity.id,
      summary,
      category,
    });
    setEditing(false);
  };

  const toggleWork = async () => {
    await updateActivityItem({
      id: activity.id,
      is_work_related: !activity.is_work_related,
    });
  };

  return (
    <div className={`activity-card ${!activity.is_work_related ? 'non-work' : ''}`}>
      <div className="activity-header">
        <span className="activity-time">
          {formatTime(activity.started_at)} &ndash; {formatTime(activity.ended_at)}
        </span>
        <span className="activity-duration">
          {formatDuration(activity.started_at, activity.ended_at)}
        </span>
        <span
          className="activity-category-badge"
          style={{ backgroundColor: CATEGORY_COLORS[activity.category] }}
        >
          {activity.category}
        </span>
      </div>

      {editing ? (
        <div className="activity-edit">
          <input
            type="text"
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            className="activity-summary-input"
          />
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value as Category)}
            className="activity-category-select"
          >
            {CATEGORIES.map((c) => (
              <option key={c.value} value={c.value}>
                {c.label}
              </option>
            ))}
          </select>
          <div className="activity-edit-actions">
            <button onClick={save} className="btn-sm btn-primary">Save</button>
            <button onClick={() => { setEditing(false); setSummary(activity.summary); setCategory(activity.category); }} className="btn-sm">Cancel</button>
          </div>
        </div>
      ) : (
        <p className="activity-summary">{activity.summary}</p>
      )}

      <div className="activity-actions">
        <button onClick={() => setEditing(true)} className="btn-sm">Edit</button>
        <button onClick={toggleWork} className="btn-sm">
          {activity.is_work_related ? 'Mark Non-Work' : 'Mark Work'}
        </button>
        <button onClick={() => deleteActivityItem(activity.id)} className="btn-sm btn-danger">
          Delete
        </button>
      </div>
    </div>
  );
}
