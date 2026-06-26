import { useState } from 'react';
import type { Activity, Category } from '../lib/types';
import { CATEGORIES, CATEGORY_COLORS } from '../lib/constants';
import { useUpdateActivity, useDeleteActivity } from '../hooks/useRecording';

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function formatDuration(start: string, end: string): string {
  const ms = new Date(end).getTime() - new Date(start).getTime();
  const secs = Math.max(1, Math.round(ms / 1000));
  if (secs < 60) {
    return `${secs}秒`;
  }
  const mins = Math.round(ms / 60000);
  if (mins < 60) return `${mins}分钟`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h}小时 ${m}分钟` : `${h}小时`;
}

interface Props {
  activity: Activity;
}

export default function ActivityCard({ activity }: Props) {
  const updateMutation = useUpdateActivity();
  const deleteMutation = useDeleteActivity();
  const [editing, setEditing] = useState(false);
  const [summary, setSummary] = useState(activity.summary);
  const [category, setCategory] = useState(activity.category);

  const save = async () => {
    await updateMutation.mutateAsync({
      id: activity.id,
      summary,
      category,
    });
    setEditing(false);
  };

  const toggleWork = async () => {
    await updateMutation.mutateAsync({
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
          {CATEGORIES.find(c => c.value === activity.category)?.label || activity.category}
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
            <button onClick={save} disabled={updateMutation.isPending} className="btn-sm btn-primary">
              {updateMutation.isPending ? '保存中…' : '保存'}
            </button>
            <button
              onClick={() => { setEditing(false); setSummary(activity.summary); setCategory(activity.category); }}
              disabled={updateMutation.isPending}
              className="btn-sm"
            >
              取消
            </button>
          </div>
        </div>
      ) : (
        <p className="activity-summary">{activity.summary}</p>
      )}

      <div className="activity-actions">
        <button
          onClick={() => setEditing(true)}
          disabled={updateMutation.isPending || deleteMutation.isPending}
          className="btn-sm"
        >
          编辑
        </button>
        <button
          onClick={toggleWork}
          disabled={updateMutation.isPending || deleteMutation.isPending}
          className="btn-sm"
        >
          {updateMutation.isPending ? '更新中…' : (activity.is_work_related ? '标记为非工作' : '标记为工作')}
        </button>
        <button
          onClick={() => {
            if (window.confirm('确定要删除这条活动记录吗？')) {
              deleteMutation.mutate(activity.id);
            }
          }}
          disabled={updateMutation.isPending || deleteMutation.isPending}
          className="btn-sm btn-danger"
        >
          {deleteMutation.isPending ? '删除中…' : '删除'}
        </button>
      </div>
    </div>
  );
}
