import { useRecordingStore } from '../stores/recording';

const LABELS: Record<string, string> = {
  stopped: '已停止',
  recording: '录制中',
  paused: '已暂停',
};

export default function StatusBadge() {
  const recordingState = useRecordingStore((s) => s.recordingState);

  return (
    <span className={`status-badge status-${recordingState}`}>
      {LABELS[recordingState]}
    </span>
  );
}
