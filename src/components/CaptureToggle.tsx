import { useRecordingStore } from '../stores/recording';

export default function CaptureToggle() {
  const { recordingState, start, pause, stop, error } = useRecordingStore();

  if (recordingState === 'stopped') {
    return (
      <div>
        <button className="capture-btn capture-start" onClick={start}>
          开始
        </button>
        {error && <p className="capture-error">{error}</p>}
      </div>
    );
  }

  return (
    <div className="capture-group">
      {recordingState === 'recording' ? (
        <button className="capture-btn capture-pause" onClick={pause}>
          暂停
        </button>
      ) : (
        <button className="capture-btn capture-start" onClick={start}>
          继续
        </button>
      )}
      <button className="capture-btn capture-stop" onClick={stop}>
        停止
      </button>
    </div>
  );
}
