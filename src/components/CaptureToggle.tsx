import { useState } from "react";
import { useRecordingStore } from "../stores/recording";

export default function CaptureToggle() {
  const { recordingState, start, pause, stop } = useRecordingStore();
  const [pending, setPending] = useState(false);

  const handleStart = async () => {
    setPending(true);
    try {
      await start();
    } finally {
      setPending(false);
    }
  };

  const handlePause = async () => {
    setPending(true);
    try {
      await pause();
    } finally {
      setPending(false);
    }
  };

  const handleStop = async () => {
    setPending(true);
    try {
      await stop();
    } finally {
      setPending(false);
    }
  };

  if (recordingState === "stopped") {
    return (
      <button className="btn-sm btn-primary capture-action" disabled={pending} onClick={handleStart}>
        {pending ? "启动中…" : "开始"}
      </button>
    );
  }

  return (
    <div className="capture-group">
      {recordingState === "recording" ? (
        <button className="btn-sm capture-action" disabled={pending} onClick={handlePause}>
          {pending ? "暂停中…" : "暂停"}
        </button>
      ) : (
        <button className="btn-sm btn-primary capture-action" disabled={pending} onClick={handleStart}>
          {pending ? "启动中…" : "继续"}
        </button>
      )}
      <button className="btn-sm btn-danger capture-action" disabled={pending} onClick={handleStop}>
        {pending ? "停止中…" : "停止"}
      </button>
    </div>
  );
}
