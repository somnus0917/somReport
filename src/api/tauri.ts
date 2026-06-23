import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  Activity,
  AppSettings,
  RecordingState,
  Report,
  TodayStats,
  UpdateActivityRequest,
} from '../lib/types';

export async function getToday(): Promise<[Activity[], TodayStats]> {
  return invoke('get_today');
}

export async function updateActivity(request: UpdateActivityRequest): Promise<void> {
  return invoke('update_activity', { request });
}

export async function deleteActivity(id: string): Promise<void> {
  return invoke('delete_activity', { id });
}

export async function generateReport(
  periodType: string,
  periodStart: string,
  templateId?: string,
): Promise<Report> {
  return invoke('generate_report', {
    periodType,
    periodStart,
    templateId: templateId ?? null,
  });
}

export async function listReports(periodType?: string): Promise<Report[]> {
  return invoke('list_reports', { periodType: periodType ?? null });
}

export async function startRecording(): Promise<void> {
  return invoke('start_recording');
}

export async function pauseRecording(): Promise<void> {
  return invoke('pause_recording');
}

export async function stopRecording(): Promise<void> {
  return invoke('stop_recording');
}

export async function getRecordingState(): Promise<RecordingState> {
  return invoke('get_recording_state');
}

export async function saveProviderKey(service: string, key: string): Promise<void> {
  return invoke('save_provider_key', { service, key });
}

export async function testProviderKey(service: string): Promise<boolean> {
  return invoke('test_provider_key', { service });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

export async function clearAllData(): Promise<void> {
  return invoke('clear_all_data');
}

export async function getDailyUsage(): Promise<number> {
  return invoke('get_daily_usage');
}

export function onRecordingStatus(callback: (state: RecordingState) => void): UnlistenFn {
  const unlisten = listen<RecordingState>('recording-status', (event) => {
    callback(event.payload);
  });
  let unlistened = false;
  return () => {
    if (!unlistened) {
      unlistened = true;
      unlisten.then((fn) => fn());
    }
  };
}

export function onActivityCreated(callback: (activity: Activity) => void): UnlistenFn {
  const unlisten = listen<Activity>('activity-created', (event) => {
    callback(event.payload);
  });
  let unlistened = false;
  return () => {
    if (!unlistened) {
      unlistened = true;
      unlisten.then((fn) => fn());
    }
  };
}

export function onJobUpdated(
  callback: (job: { id: string; status: string }) => void,
): UnlistenFn {
  const unlisten = listen<{ id: string; status: string }>('job-updated', (event) => {
    callback(event.payload);
  });
  let unlistened = false;
  return () => {
    if (!unlistened) {
      unlistened = true;
      unlisten.then((fn) => fn());
    }
  };
}
