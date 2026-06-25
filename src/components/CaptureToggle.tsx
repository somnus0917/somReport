import { useRecordingStore } from "../stores/recording";

export default function CaptureToggle() {
  const { recordingState, start, pause, stop } = useRecordingStore();

  if (recordingState === "stopped") {
    return (
      <button className="btn-sm btn-primary capture-action" onClick={start}>
        开始
      </button>
    );
  }

  return (
    <div className="capture-group">
      {recordingState === "recording" ? (
        <button className="btn-sm capture-action" onClick={pause}>
          暂停
        </button>
      ) : (
        <button className="btn-sm btn-primary capture-action" onClick={start}>
          继续
        </button>
      )}
      <button className="btn-sm btn-danger capture-action" onClick={stop}>
        停止
      </button>
    </div>
  );
}
