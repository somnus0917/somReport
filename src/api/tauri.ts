import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { safeInvoke } from '../lib/tauri';
import type {
  Activity,
  AppSettings,
  DailyUsage,
  ProviderConfig,
  RecordingState,
  Report,
  TodayStats,
  UpdateActivityRequest,
} from '../lib/types';

export async function getToday(): Promise<[Activity[], TodayStats]> {
  return safeInvoke('get_today', undefined, '获取今日数据失败');
}

export async function updateActivity(request: UpdateActivityRequest): Promise<void> {
  return safeInvoke('update_activity', { request }, '更新活动失败');
}

export async function deleteActivity(id: string): Promise<void> {
  return safeInvoke('delete_activity', { id }, '删除活动失败');
}

export async function generateReport(
  periodType: string,
  periodStart: string,
  templateId?: string,
): Promise<Report> {
  return safeInvoke('generate_report', {
    periodType,
    periodStart,
    templateId: templateId ?? null,
  }, '生成报告失败');
}

export async function listReports(periodType?: string): Promise<Report[]> {
  return safeInvoke('list_reports', { periodType: periodType ?? null }, '获取报告列表失败');
}

export async function startRecording(): Promise<void> {
  return safeInvoke('start_recording', undefined, '开始录制失败');
}

export async function pauseRecording(): Promise<void> {
  return safeInvoke('pause_recording', undefined, '暂停录制失败');
}

export async function stopRecording(): Promise<void> {
  return safeInvoke('stop_recording', undefined, '停止录制失败');
}

export async function getRecordingState(): Promise<RecordingState> {
  return safeInvoke('get_recording_state', undefined, '获取录制状态失败');
}

export async function showMainWindow(): Promise<void> {
  return safeInvoke('show_main_window', undefined, '打开主窗口失败');
}

export async function getSettings(): Promise<AppSettings> {
  return safeInvoke('get_settings', undefined, '获取设置失败');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return safeInvoke('save_settings', { settings }, '保存设置失败');
}

export async function clearAllData(): Promise<void> {
  return safeInvoke('clear_all_data', undefined, '清除数据失败');
}

export async function cleanupLocalStorage(retentionDays: number): Promise<void> {
  return safeInvoke('cleanup_local_storage', { retentionDays }, '清理本地存储失败');
}

export async function getDailyUsage(): Promise<DailyUsage> {
  return safeInvoke('get_daily_usage', undefined, '获取使用量失败');
}

export interface TestResult {
  success: boolean;
  message: string;
  response: string | null;
}

export async function testModelConnection(
  providerType: 'vision' | 'text',
  config: ProviderConfig,
): Promise<TestResult> {
  return safeInvoke('test_model_connection', { providerType, config }, '测试模型连接失败');
}

export async function saveModelConfig(
  providerType: 'vision' | 'text',
  config: ProviderConfig,
): Promise<AppSettings> {
  return safeInvoke(
    'save_model_config',
    { providerType, config },
    '保存模型配置失败',
  );
}

export async function saveAndTestModel(
  providerType: 'vision' | 'text',
  config: ProviderConfig,
): Promise<TestResult> {
  return safeInvoke(
    'save_and_test_model',
    { providerType, config },
    '保存或测试模型失败',
  );
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
