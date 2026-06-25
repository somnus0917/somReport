interface Props {
  message?: string;
}

export function LoadingState({ message = '加载中...' }: Props) {
  return (
    <div className="state-container state-loading">
      <div className="state-spinner" />
      <p>{message}</p>
    </div>
  );
}

export function ErrorState({
  message = '加载失败',
  onRetry,
}: Props & { onRetry?: () => void }) {
  return (
    <div className="state-container state-error">
      <p>{message}</p>
      {onRetry && (
        <button className="btn btn-secondary btn-sm" onClick={onRetry}>
          重试
        </button>
      )}
    </div>
  );
}

export function EmptyState({ message = '暂无数据' }: Props) {
  return (
    <div className="state-container state-empty">
      <p>{message}</p>
    </div>
  );
}
