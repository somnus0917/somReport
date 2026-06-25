import { useEffect, useState, type MouseEvent } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { Activity, RecordingState } from '../lib/types';
import { CATEGORIES } from '../lib/constants';
import {
  getRecordingState,
  getToday,
  onActivityCreated,
  onRecordingStatus,
  showMainWindow,
  startRecording,
  pauseRecording,
  stopRecording,
} from '../api/tauri';

const STATE_LABEL: Record<RecordingState, string> = {
  recording: '正在记录',
  paused: '已暂停',
  stopped: '未开始',
};

export default function FloatingWidget() {
  const [state, setState] = useState<RecordingState>('stopped');
  const [latestActivity, setLatestActivity] = useState<Activity | null>(null);

  function startDragging(event: MouseEvent<HTMLDivElement>) {
    if ((event.target as HTMLElement).closest('button')) return;
    void getCurrentWindow().startDragging();
  }

  useEffect(() => {
    document.body.classList.add('floating-window');
    void getRecordingState().then(setState).catch(() => undefined);
    void getToday()
      .then(([activities]) => setLatestActivity(activities[activities.length - 1] ?? null))
      .catch(() => undefined);
    const unlistenState = onRecordingStatus(setState);
    const unlistenActivity = onActivityCreated(setLatestActivity);
    return () => {
      document.body.classList.remove('floating-window');
      unlistenState();
      unlistenActivity();
    };
  }, []);

  function handlePrimaryAction() {
    if (state === 'recording') {
      void pauseRecording().catch(() => undefined);
    } else {
      void startRecording().catch(() => undefined);
    }
  }

  const primaryLabel = state === 'recording' ? '暂停' : state === 'paused' ? '继续' : '开始';

  return (
    <div className="floating-widget" onMouseDown={startDragging}>
      <div className="floating-widget-status">
        <span className={`floating-state-dot ${state}`} aria-hidden="true" />
        <span>{STATE_LABEL[state]}</span>
      </div>
      <div className="floating-widget-controls">
        <button className="fw-btn" onClick={handlePrimaryAction}>
          {primaryLabel}
        </button>
        <button
          className="fw-btn"
          onClick={() => void stopRecording().catch(() => undefined)}
          disabled={state === 'stopped'}
        >
          停止
        </button>
        <button className="fw-btn fw-btn-open" onClick={() => void showMainWindow()}>
          打开
        </button>
      </div>
      <div className="floating-widget-content">
        <p>{latestActivity?.summary ?? '等待下一次识别…'}</p>
        <span>{latestActivity ? (CATEGORIES.find(c => c.value === latestActivity.category)?.label || latestActivity.category) : '日报助手'}</span>
      </div>
    </div>
  );
}
