import { create } from 'zustand';
import { queryClient } from '../lib/queryClient';
import type { RecordingState } from '../lib/types';
import {
  startRecording,
  pauseRecording,
  stopRecording,
  onRecordingStatus,
  onActivityCreated,
} from '../api/tauri';

interface RecordingUIStore {
  recordingState: RecordingState;
  error: string | null;

  start: () => Promise<void>;
  pause: () => Promise<void>;
  stop: () => Promise<void>;
  subscribe: () => () => void;
  clearError: () => void;
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
    await pauseRecording();
  },

  stop: async () => {
    await stopRecording();
  },

  subscribe: () => {
    const unlistenStatus = onRecordingStatus((recordingState) => {
      set({ recordingState });
    });

    const unlistenActivity = onActivityCreated(() => {
      queryClient.invalidateQueries({ queryKey: ['today'] });
    });

    return () => {
      unlistenStatus();
      unlistenActivity();
    };
  },

  clearError: () => set({ error: null }),
}));
