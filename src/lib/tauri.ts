import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

export async function safeInvoke<T>(
  cmd: string,
  args?: InvokeArgs,
  errorMsg = '操作失败，请重试'
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    console.error(`[invoke:${cmd}]`, e);
    const detail = e instanceof Error ? e.message : String(e);
    throw new Error(detail ? `${errorMsg}：${detail}` : errorMsg);
  }
}
