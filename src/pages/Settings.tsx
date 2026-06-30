import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  clearDataSelective,
  getStorageSize,
  saveSettings,
  type StorageSizeInfo,
} from "../api/tauri";
import { LoadingState } from "../components/StateViews";
import type { AppSettings, ModelConnectionStatus } from "../lib/types";
import { useInvalidateSettings, useSettings } from "../hooks/useSettings";

type OperationalSettings = Pick<
  AppSettings,
  | "capture_interval_secs"
  | "idle_timeout_secs"
  | "max_daily_cost_yuan"
  | "auto_start"
  | "notify_on_report"
  | "auto_cleanup_cache_days"
>;

function connectionLabel(status: ModelConnectionStatus) {
  if (status.success === true) return "已验证";
  if (status.success === false) return "测试失败";
  return "尚未验证";
}

function formatTestedAt(value: string | null) {
  if (!value) return "尚未进行实时调用";
  return `上次测试：${new Date(value).toLocaleString("zh-CN", { hour12: false })}`;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export default function Settings() {
  const navigate = useNavigate();
  const { data: settings, isLoading } = useSettings();
  const invalidateSettings = useInvalidateSettings();
  const [form, setForm] = useState<OperationalSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [storageSize, setStorageSize] = useState<StorageSizeInfo | null>(null);
  const [clearCacheChecked, setClearCacheChecked] = useState(true);
  const [clearDbChecked, setClearDbChecked] = useState(false);
  const [theme, setTheme] = useState<"light" | "dark" | "system">("system");

  useEffect(() => {
    const savedTheme =
      (localStorage.getItem("somreport-theme") as any) || "system";
    setTheme(savedTheme);
  }, []);

  const handleThemeChange = (newTheme: "light" | "dark" | "system") => {
    setTheme(newTheme);
    localStorage.setItem("somreport-theme", newTheme);
    let resolved: "light" | "dark" = "dark";
    if (newTheme === "system") {
      const isLight = window.matchMedia(
        "(prefers-color-scheme: light)",
      ).matches;
      resolved = isLight ? "light" : "dark";
    } else {
      resolved = newTheme;
    }
    document.documentElement.setAttribute("data-theme", resolved);
  };

  const fetchStorageSize = async () => {
    try {
      const size = await getStorageSize();
      setStorageSize(size);
    } catch (e) {
      console.error("Failed to load storage size", e);
    }
  };

  useEffect(() => {
    fetchStorageSize();
  }, []);

  useEffect(() => {
    if (!settings || form) return;
    setForm({
      capture_interval_secs: settings.capture_interval_secs,
      idle_timeout_secs: settings.idle_timeout_secs,
      max_daily_cost_yuan: settings.max_daily_cost_yuan,
      auto_start: settings.auto_start,
      notify_on_report: settings.notify_on_report,
      auto_cleanup_cache_days: settings.auto_cleanup_cache_days,
    });
  }, [settings, form]);

  if (isLoading || !settings || !form) {
    return (
      <div className="settings-page">
        <LoadingState message="正在读取设置…" />
      </div>
    );
  }

  const activeSettings: AppSettings = settings;
  const operationalForm: OperationalSettings = form;

  const hasChanges = Object.entries(operationalForm).some(
    ([key, value]) =>
      activeSettings[key as keyof OperationalSettings] !== value,
  );

  const hasInvalidFields =
    Number.isNaN(operationalForm.capture_interval_secs) ||
    Number.isNaN(operationalForm.idle_timeout_secs) ||
    Number.isNaN(operationalForm.max_daily_cost_yuan) ||
    Number.isNaN(operationalForm.auto_cleanup_cache_days);

  async function saveOperationalSettings() {
    setSaving(true);
    setError(null);
    try {
      await saveSettings({ ...activeSettings, ...operationalForm });
      await invalidateSettings();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "保存设置失败");
    } finally {
      setSaving(false);
    }
  }

  async function handleSelectiveClear() {
    setSaving(true);
    setError(null);
    try {
      await clearDataSelective(clearCacheChecked, clearDbChecked);
      setConfirmClear(false);
      await fetchStorageSize();
      if (clearDbChecked) {
        setForm(null);
        await invalidateSettings();
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "清理数据失败");
    } finally {
      setSaving(false);
    }
  }

  const modelCards = [
    {
      role: "vision" as const,
      eyebrow: "截图分析",
      title: "视觉模型",
      config: settings.vision_provider,
      status: settings.vision_connection,
    },
    {
      role: "text" as const,
      eyebrow: "报告生成",
      title: "文本模型",
      config: settings.text_provider,
      status: settings.text_connection,
    },
  ];

  return (
    <div className="settings-page settings-dashboard">
      <header className="settings-header settings-dashboard-header">
        <div>
          <p className="settings-kicker">SYSTEM SETUP</p>
          <h2>设置</h2>
          <p>模型配置独立管理；每次测试都是真实 API 调用。</p>
        </div>
      </header>

      <section className="settings-section">
        <div className="settings-section-heading">
          <div>
            <p className="settings-kicker">MODEL CONNECTIONS</p>
            <h3>模型连接</h3>
          </div>
          <span className="settings-section-note">
            配置、保存、测试在同一处完成
          </span>
        </div>
        <div className="model-role-grid">
          {modelCards.map(({ role, eyebrow, title, config, status }) => (
            <article className="model-role-card" key={role}>
              <div className="model-role-card-topline">
                <span>{eyebrow}</span>
                <span
                  className={`connection-badge ${status.success === true ? "connected" : status.success === false ? "failed" : "pending"}`}
                >
                  {connectionLabel(status)}
                </span>
              </div>
              <h4>{title}</h4>
              <p className="model-role-model">{config.model}</p>
              <div className="model-role-proof">
                <span>{formatTestedAt(status.tested_at)}</span>
                {status.message && (
                  <span title={status.message}>{status.message}</span>
                )}
              </div>
              <button
                className="btn-sm btn-primary"
                onClick={() => navigate(`/settings/model/${role}`)}
              >
                配置并测试 <span aria-hidden="true">→</span>
              </button>
            </article>
          ))}
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-heading">
          <div>
            <p className="settings-kicker">CAPTURE BEHAVIOR</p>
            <h3>采集与提醒</h3>
          </div>
        </div>
        <div className="settings-card settings-operational-card">
          <div className="settings-input-grid">
            <label>
              截图间隔（秒）
              <input
                type="number"
                min={5}
                max={3600}
                value={
                  Number.isNaN(operationalForm.capture_interval_secs)
                    ? ""
                    : operationalForm.capture_interval_secs
                }
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    capture_interval_secs:
                      event.target.value === ""
                        ? NaN
                        : Number(event.target.value),
                  })
                }
              />
            </label>
            <label>
              空闲超时（秒）
              <input
                type="number"
                min={30}
                max={86400}
                value={
                  Number.isNaN(operationalForm.idle_timeout_secs)
                    ? ""
                    : operationalForm.idle_timeout_secs
                }
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    idle_timeout_secs:
                      event.target.value === ""
                        ? NaN
                        : Number(event.target.value),
                  })
                }
              />
            </label>
            <label>
              每日预算（元）
              <input
                type="number"
                min={0}
                step="0.01"
                value={
                  Number.isNaN(operationalForm.max_daily_cost_yuan)
                    ? ""
                    : operationalForm.max_daily_cost_yuan
                }
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    max_daily_cost_yuan:
                      event.target.value === ""
                        ? NaN
                        : Number(event.target.value),
                  })
                }
              />
            </label>
            <label>
              自动清理截图缓存（天，0 为不清理）
              <input
                type="number"
                min={0}
                value={
                  Number.isNaN(operationalForm.auto_cleanup_cache_days)
                    ? ""
                    : operationalForm.auto_cleanup_cache_days
                }
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    auto_cleanup_cache_days:
                      event.target.value === ""
                        ? NaN
                        : Number(event.target.value),
                  })
                }
              />
            </label>
          </div>
          <div className="settings-switch-list">
            <label className="settings-switch-row">
              <span>
                <strong>启动时开始录制</strong>
                <small>应用启动后自动进入录制状态</small>
              </span>
              <input
                type="checkbox"
                checked={operationalForm.auto_start}
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    auto_start: event.target.checked,
                  })
                }
              />
            </label>
            <label className="settings-switch-row">
              <span>
                <strong>报告完成时提醒</strong>
                <small>生成日报或周报后显示系统通知</small>
              </span>
              <input
                type="checkbox"
                checked={operationalForm.notify_on_report}
                onChange={(event) =>
                  setForm({
                    ...operationalForm,
                    notify_on_report: event.target.checked,
                  })
                }
              />
            </label>
          </div>
          <div className="settings-save-row">
            {error && <p className="settings-inline-error">{error}</p>}
            <button
              className="btn-sm btn-primary"
              disabled={!hasChanges || saving || hasInvalidFields}
              onClick={saveOperationalSettings}
            >
              {saving ? "保存中…" : "保存采集设置"}
            </button>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-heading">
          <div>
            <p className="settings-kicker">APPEARANCE</p>
            <h3>外观与主题</h3>
          </div>
        </div>
        <div className="settings-card">
          <div className="settings-input-grid">
            <label>
              选择主题颜色
              <select
                value={theme}
                onChange={(e) => handleThemeChange(e.target.value as any)}
              >
                <option value="system">跟随系统</option>
                <option value="light">浅色主题 (Light)</option>
                <option value="dark">深色主题 (Dark)</option>
              </select>
            </label>
          </div>
        </div>
      </section>

      <section className="settings-section settings-danger">
        <div className="settings-section-heading">
          <div>
            <p className="settings-kicker">LOCAL STORAGE</p>
            <h3>存储空间与数据清理</h3>
          </div>
        </div>
        <div className="settings-card settings-danger-card">
          <div className="storage-stats-grid">
            <div className="storage-stat-item">
              <span className="storage-stat-label">
                数据库占用（活动与配置）
              </span>
              <strong className="storage-stat-value">
                {storageSize
                  ? formatBytes(storageSize.db_size_bytes)
                  : "正在读取…"}
              </strong>
            </div>
            <div className="storage-stat-item">
              <span className="storage-stat-label">临时截图与缓存文件</span>
              <strong className="storage-stat-value">
                {storageSize
                  ? formatBytes(storageSize.cache_size_bytes)
                  : "正在读取…"}
              </strong>
            </div>
          </div>

          <hr className="settings-divider" />

          <div className="settings-danger-row">
            <div style={{ flex: 1 }}>
              <strong>选择性清理本地数据</strong>
              <p
                style={{
                  margin: "0.25rem 0 0.75rem",
                  fontSize: "0.8rem",
                  color: "var(--color-text-muted)",
                }}
              >
                可以单独选择清理截图缓存或完全抹除数据库记录：
              </p>

              <div className="clear-options-grid">
                <label className="clear-option-checkbox">
                  <input
                    type="checkbox"
                    checked={clearCacheChecked}
                    onChange={(e) => setClearCacheChecked(e.target.checked)}
                  />
                  <span>清理临时图片与截图缓存</span>
                </label>
                <label className="clear-option-checkbox">
                  <input
                    type="checkbox"
                    checked={clearDbChecked}
                    onChange={(e) => setClearDbChecked(e.target.checked)}
                  />
                  <span className="warning-text">
                    抹除数据库数据（活动记录、报告、配置等）
                  </span>
                </label>
              </div>
            </div>

            <div className="clear-actions">
              {!confirmClear ? (
                <button
                  className="btn-sm btn-danger"
                  disabled={!clearCacheChecked && !clearDbChecked}
                  onClick={() => setConfirmClear(true)}
                >
                  清理选中项目
                </button>
              ) : (
                <div className="settings-danger-confirm">
                  <button
                    className="btn-sm btn-danger"
                    disabled={saving}
                    onClick={handleSelectiveClear}
                  >
                    确认清理
                  </button>
                  <button
                    className="btn-sm"
                    disabled={saving}
                    onClick={() => setConfirmClear(false)}
                  >
                    取消
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
