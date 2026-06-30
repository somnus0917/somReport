import { useMemo } from "react";
import type { Activity, Category } from "../lib/types";
import { CATEGORIES, CATEGORY_COLORS } from "../lib/constants";
import { parseDate } from "../lib/datetime";

interface Props {
  activities: Activity[];
}

interface HeatmapCell {
  category: Category;
  hour: number;
  minutes: number;
}

type ActivityWithDateAliases = Activity & {
  startedAt?: string;
  endedAt?: string;
};

const HOURS = Array.from({ length: 24 }, (_, hour) => hour);
const MS_PER_HOUR = 60 * 60 * 1000;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function parseActivityTime(activity: ActivityWithDateAliases, field: "started" | "ended"): Date {
  const value =
    field === "started"
      ? activity.started_at ?? activity.startedAt
      : activity.ended_at ?? activity.endedAt;
  return parseDate(value);
}

function normalizeCategory(category: string): Category {
  return CATEGORIES.some(({ value }) => value === category)
    ? (category as Category)
    : "other";
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const value = hex.replace("#", "");
  return {
    r: Number.parseInt(value.slice(0, 2), 16),
    g: Number.parseInt(value.slice(2, 4), 16),
    b: Number.parseInt(value.slice(4, 6), 16),
  };
}

function heatmapColor(category: Category, alpha: number): string {
  const { r, g, b } = hexToRgb(CATEGORY_COLORS[category]);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function buildHeatmap(activities: Activity[]): HeatmapCell[] {
  const totals = new Map<string, number>();

  for (const activity of activities) {
    const safeActivity = activity as ActivityWithDateAliases;
    const category = normalizeCategory(safeActivity.category);
    const startedAt = parseActivityTime(safeActivity, "started");
    const endedAt = parseActivityTime(safeActivity, "ended");
    const startMs = startedAt.getTime();
    const endMs = endedAt.getTime();

    if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
      continue;
    }

    const cursor = new Date(startedAt);
    cursor.setMinutes(0, 0, 0);

    while (cursor.getTime() < endMs) {
      const hourStart = cursor.getTime();
      const hourEnd = hourStart + MS_PER_HOUR;
      const overlapMs = Math.max(0, Math.min(endMs, hourEnd) - Math.max(startMs, hourStart));

      if (overlapMs > 0) {
        const hour = cursor.getHours();
        const key = `${category}-${hour}`;
        totals.set(key, (totals.get(key) ?? 0) + overlapMs / 60000);
      }

      cursor.setHours(cursor.getHours() + 1);
    }
  }

  return CATEGORIES.flatMap(({ value }) =>
    HOURS.map((hour) => ({
      category: value,
      hour,
      minutes: totals.get(`${value}-${hour}`) ?? 0,
    })),
  );
}

function formatHour(hour: number): string {
  return `${String(hour).padStart(2, "0")}:00`;
}

function formatMinutes(minutes: number): string {
  if (minutes <= 0) return "0分钟";
  if (minutes < 60) return `${Math.round(minutes)}分钟`;
  const hours = Math.floor(minutes / 60);
  const rest = Math.round(minutes % 60);
  return rest ? `${hours}小时${rest}分钟` : `${hours}小时`;
}

export default function ActivityHeatmap({ activities }: Props) {
  const { cells, maxMinutes, activeMinutes, busiestCell } = useMemo(() => {
    const builtCells = buildHeatmap(activities);
    const max = Math.max(...builtCells.map((cell) => cell.minutes), 0);
    const total = builtCells.reduce((sum, cell) => sum + cell.minutes, 0);
    const busiest = builtCells.reduce<HeatmapCell | null>((current, cell) => {
      if (!current || cell.minutes > current.minutes) return cell;
      return current;
    }, null);

    return {
      cells: builtCells,
      maxMinutes: max,
      activeMinutes: total,
      busiestCell: busiest && busiest.minutes > 0 ? busiest : null,
    };
  }, [activities]);

  const cellByKey = useMemo(() => {
    return new Map(cells.map((cell) => [`${cell.category}-${cell.hour}`, cell]));
  }, [cells]);

  return (
    <section className="activity-heatmap" aria-labelledby="activity-heatmap-title">
      <div className="heatmap-header">
        <div>
          <p className="heatmap-kicker">activity heatmap</p>
          <h3 id="activity-heatmap-title">时间热力图</h3>
        </div>
        <div className="heatmap-stats">
          <span>{formatMinutes(activeMinutes)}</span>
          <span>{activities.length} 个活动</span>
          {busiestCell && (
            <span>
              峰值 {formatHour(busiestCell.hour)} · {formatMinutes(busiestCell.minutes)}
            </span>
          )}
        </div>
      </div>

      <div className="heatmap-scroller" role="img" aria-label="按小时和活动分类统计的热力图">
        <div className="heatmap-grid">
          <div className="heatmap-axis-corner" />
          {HOURS.map((hour) => (
            <div key={hour} className="heatmap-hour-label">
              {hour % 3 === 0 ? String(hour).padStart(2, "0") : ""}
            </div>
          ))}

          {CATEGORIES.map(({ value, label }) => (
            <div className="heatmap-row" key={value}>
              <div className="heatmap-category-label">
                <span
                  className="heatmap-category-dot"
                  style={{ backgroundColor: CATEGORY_COLORS[value] }}
                />
                {label}
              </div>
              {HOURS.map((hour) => {
                const cell = cellByKey.get(`${value}-${hour}`);
                const minutes = cell?.minutes ?? 0;
                const intensity = maxMinutes > 0 ? clamp(minutes / maxMinutes, 0, 1) : 0;
                const alpha = minutes > 0 ? 0.18 + intensity * 0.72 : 0;

                return (
                  <div
                    key={hour}
                    className={`heatmap-cell ${minutes > 0 ? "active" : ""}`}
                    title={`${label} · ${formatHour(hour)} · ${formatMinutes(minutes)}`}
                    style={{
                      backgroundColor:
                        minutes > 0 ? heatmapColor(value, alpha) : undefined,
                    }}
                  >
                    <span>{minutes > 0 ? Math.max(1, Math.round(minutes)) : ""}</span>
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
