import { useRecordingStore } from '../stores/recording';

const LABELS: Record<string, string> = {
  stopped: 'Stopped',
  recording: 'Recording',
  paused: 'Paused',
};

export default function StatusBadge() {
  const recordingState = useRecordingStore((s) => s.recordingState);

  return (
    <span className={`status-badge status-${recordingState}`}>
      {LABELS[recordingState]}
    </span>
  );
}
