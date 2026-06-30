import { create } from 'zustand';
import { queryClient } from '../lib/queryClient';
import type { RecordingState } from '../lib/types';
import {
  startRecording,
  pauseRecording,
  stopRecording,
  getRecordingState,
  onRecordingStatus,
  onActivityCreated,
} from '../api/tauri';
import type { Activity, TodayStats } from '../lib/types';
import { elapsedSeconds, parseDateTimestamp } from '../lib/datetime';

interface RecordingUIStore {
  recordingState: RecordingState;
  error: string | null;

  start: () => Promise<void>;
  pause: () => Promise<void>;
  stop: () => Promise<void>;
  subscribe: () => () => void;
  clearError: () => void;
}

function calculateTodayStats(activities: Activity[]): TodayStats {
  const totalSeconds = activities.reduce((sum, activity) => {
    return sum + elapsedSeconds(activity.started_at, activity.ended_at);
  }, 0);
  const workSeconds = activities
    .filter((activity) => activity.is_work_related)
    .reduce((sum, activity) => {
      return sum + elapsedSeconds(activity.started_at, activity.ended_at);
    }, 0);

  return {
    total_minutes: Math.round(totalSeconds / 60),
    work_minutes: Math.round(workSeconds / 60),
    activity_count: activities.length,
  };
}

function mergeActivity(activities: Activity[], activity: Activity): Activity[] {
  const next = activities.filter((item) => item.id !== activity.id);
  next.push(activity);
  return next.sort(
    (a, b) => parseDateTimestamp(a.started_at) - parseDateTimestamp(b.started_at),
  );
}

export const useRecordingStore = create<RecordingUIStore>((set) => ({
  recordingState: 'stopped',
  error: null,

  start: async () => {
    try {
      await startRecording();
      set({ error: null });
    } catch (error) {
      const message = error instanceof Error ? error.message : '无法开始录制';
      set({ error: message });
    }
  },

  pause: async () => {
    try {
      await pauseRecording();
      set({ error: null });
    } catch (error) {
      const message = error instanceof Error ? error.message : '无法暂停录制';
      set({ error: message });
    }
  },

  stop: async () => {
    try {
      await stopRecording();
      set({ error: null });
    } catch (error) {
      const message = error instanceof Error ? error.message : '无法停止录制';
      set({ error: message });
    }
  },

  subscribe: () => {
    getRecordingState()
      .then((state) => {
        set({ recordingState: state });
      })
      .catch((err) => {
        console.error('Failed to sync initial recording state:', err);
      });

    const unlistenStatus = onRecordingStatus((recordingState) => {
      set({ recordingState });
    });

    const unlistenActivity = onActivityCreated((activity) => {
      queryClient.setQueryData<[Activity[], TodayStats]>(['today'], (current) => {
        const activities = mergeActivity(current?.[0] ?? [], activity);
        return [activities, calculateTodayStats(activities)];
      });
      queryClient.invalidateQueries({ queryKey: ['today'] });
      queryClient.invalidateQueries({ queryKey: ['dailyUsage'] });
    });

    return () => {
      unlistenStatus();
      unlistenActivity();
    };
  },

  clearError: () => set({ error: null }),
}));
