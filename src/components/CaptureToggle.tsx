import { useRecordingStore } from '../stores/recording';

export default function CaptureToggle() {
  const { recordingState, start, pause, stop } = useRecordingStore();

  if (recordingState === 'stopped') {
    return (
      <button className="capture-btn capture-start" onClick={start}>
        Start
      </button>
    );
  }

  return (
    <div className="capture-group">
      {recordingState === 'recording' ? (
        <button className="capture-btn capture-pause" onClick={pause}>
          Pause
        </button>
      ) : (
        <button className="capture-btn capture-start" onClick={start}>
          Resume
        </button>
      )}
      <button className="capture-btn capture-stop" onClick={stop}>
        Stop
      </button>
    </div>
  );
}
