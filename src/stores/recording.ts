import { create } from 'zustand';
import type { Activity, RecordingState, TodayStats } from '../lib/types';
import {
  getToday,
  getRecordingState,
  getDailyUsage,
  startRecording,
  pauseRecording,
  stopRecording,
  updateActivity,
  deleteActivity,
  onRecordingStatus,
  onActivityCreated,
} from '../api/tauri';

interface RecordingStore {
  recordingState: RecordingState;
  activities: Activity[];
  stats: TodayStats;
  dailyCostCents: number;
  loading: boolean;
  error: string | null;

  fetchToday: () => Promise<void>;
  fetchRecordingState: () => Promise<void>;
  fetchDailyUsage: () => Promise<void>;
  start: () => Promise<void>;
  pause: () => Promise<void>;
  stop: () => Promise<void>;
  updateActivityItem: (req: Parameters<typeof updateActivity>[0]) => Promise<void>;
  deleteActivityItem: (id: string) => Promise<void>;
  subscribe: () => () => void;
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  recordingState: 'stopped',
  activities: [],
  stats: { total_minutes: 0, work_minutes: 0, activity_count: 0 },
  dailyCostCents: 0,
  loading: true,
  error: null,

  fetchToday: async () => {
    const [activities, stats] = await getToday();
    set({ activities, stats });
  },

  fetchRecordingState: async () => {
    const recordingState = await getRecordingState();
    set({ recordingState });
  },

  fetchDailyUsage: async () => {
    const dailyCostCents = await getDailyUsage();
    set({ dailyCostCents });
  },

  start: async () => {
    try {
      await startRecording();
      set({ error: null });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Unable to start recording.' });
    }
  },

  pause: async () => {
    await pauseRecording();
  },

  stop: async () => {
    await stopRecording();
  },

  updateActivityItem: async (req) => {
    await updateActivity(req);
    await get().fetchToday();
  },

  deleteActivityItem: async (id) => {
    await deleteActivity(id);
    await get().fetchToday();
  },

  subscribe: () => {
    const unlistenStatus = onRecordingStatus((recordingState) => {
      set({ recordingState });
    });

    const unlistenActivity = onActivityCreated((activity) => {
      set((state) => ({
        activities: [activity, ...state.activities],
        stats: { ...state.stats, activity_count: state.stats.activity_count + 1 },
      }));
      void get().fetchToday();
      void get().fetchDailyUsage();
    });

    return () => {
      unlistenStatus();
      unlistenActivity();
    };
  },
}));
