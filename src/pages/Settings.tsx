import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { cleanupLocalStorage, clearAllData, saveSettings } from '../api/tauri';
import { LoadingState } from '../components/StateViews';
import type { AppSettings, ModelConnectionStatus } from '../lib/types';
import { useInvalidateSettings, useSettings } from '../hooks/useSettings';

type OperationalSettings = Pick<
  AppSettings,
  'capture_interval_secs' | 'idle_timeout_secs' | 'max_daily_cost_cents' | 'auto_start' | 'notify_on_report' | 'data_retention_days'
>;

function connectionLabel(status: ModelConnectionStatus) {
  if (status.success === true) return '已验证';
  if (status.success === false) return '测试失败';
  return '尚未验证';
}

function formatTestedAt(value: string | null) {
  if (!value) return '尚未进行实时调用';
  return `上次测试：${new Date(value).toLocaleString('zh-CN', { hour12: false })}`;
}

export default function Settings() {
  const navigate = useNavigate();
  const { data: settings, isLoading } = useSettings();
  const invalidateSettings = useInvalidateSettings();
  const [form, setForm] = useState<OperationalSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!settings || form) return;
    setForm({
      capture_interval_secs: settings.capture_interval_secs,
      idle_timeout_secs: settings.idle_timeout_secs,
      max_daily_cost_cents: settings.max_daily_cost_cents,
      auto_start: settings.auto_start,
      notify_on_report: settings.notify_on_report,
      data_retention_days: settings.data_retention_days,
    });
  }, [settings, form]);

  if (isLoading || !settings || !form) {
    return <div className="settings-page"><LoadingState message="正在读取设置…" /></div>;
  }

  const activeSettings: AppSettings = settings;
  const operationalForm: OperationalSettings = form;

  const hasChanges = Object.entries(operationalForm).some(([key, value]) => activeSettings[key as keyof OperationalSettings] !== value);

  async function saveOperationalSettings() {
    setSaving(true);
    setError(null);
    try {
      await saveSettings({ ...activeSettings, ...operationalForm });
      await invalidateSettings();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : '保存设置失败');
    } finally {
      setSaving(false);
    }
  }

  async function clearData() {
    setSaving(true);
    setError(null);
    try {
      await clearAllData();
      setConfirmClear(false);
      setForm(null);
      await invalidateSettings();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : '清除数据失败');
    } finally {
      setSaving(false);
    }
  }

  async function cleanupStorage() {
    setCleaning(true);
    setError(null);
    try {
      await cleanupLocalStorage(operationalForm.data_retention_days);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : '清理本地存储失败');
    } finally {
      setCleaning(false);
    }
  }

  const modelCards = [
    {
      role: 'vision' as const,
      eyebrow: '截图分析',
      title: '视觉模型',
      config: settings.vision_provider,
      status: settings.vision_connection,
    },
    {
      role: 'text' as const,
      eyebrow: '报告生成',
      title: '文本模型',
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
          <span className="settings-section-note">配置、保存、测试在同一处完成</span>
        </div>
        <div className="model-role-grid">
          {modelCards.map(({ role, eyebrow, title, config, status }) => (
            <article className="model-role-card" key={role}>
              <div className="model-role-card-topline">
                <span>{eyebrow}</span>
                <span className={`connection-badge ${status.success === true ? 'connected' : status.success === false ? 'failed' : 'pending'}`}>
                  {connectionLabel(status)}
                </span>
              </div>
              <h4>{title}</h4>
              <p className="model-role-model">{config.model}</p>
              <p className="model-role-provider">{config.name} · {config.api_url.replace(/^https?:\/\//, '')}</p>
              <div className="model-role-proof">
                <span>{formatTestedAt(status.tested_at)}</span>
                {status.message && <span title={status.message}>{status.message}</span>}
              </div>
              <button className="btn-sm model-role-action" onClick={() => navigate(`/settings/model/${role}`)}>
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
                value={operationalForm.capture_interval_secs}
                onChange={(event) => setForm({ ...operationalForm, capture_interval_secs: Number(event.target.value) })}
              />
            </label>
            <label>
              空闲超时（秒）
              <input
                type="number"
                min={30}
                max={86400}
                value={operationalForm.idle_timeout_secs}
                onChange={(event) => setForm({ ...operationalForm, idle_timeout_secs: Number(event.target.value) })}
              />
            </label>
            <label>
              每日预算（分）
              <input
                type="number"
                min={0}
                value={operationalForm.max_daily_cost_cents}
                onChange={(event) => setForm({ ...operationalForm, max_daily_cost_cents: Number(event.target.value) })}
              />
            </label>
            <label>
              数据保留（天，0 为不自动清理）
              <input
                type="number"
                min={0}
                value={operationalForm.data_retention_days}
                onChange={(event) => setForm({ ...operationalForm, data_retention_days: Number(event.target.value) })}
              />
            </label>
          </div>
          <div className="settings-switch-list">
            <label className="settings-switch-row">
              <span><strong>启动时开始录制</strong><small>应用启动后自动进入录制状态</small></span>
              <input type="checkbox" checked={operationalForm.auto_start} onChange={(event) => setForm({ ...operationalForm, auto_start: event.target.checked })} />
            </label>
            <label className="settings-switch-row">
              <span><strong>报告完成时提醒</strong><small>生成日报或周报后显示系统通知</small></span>
              <input type="checkbox" checked={operationalForm.notify_on_report} onChange={(event) => setForm({ ...operationalForm, notify_on_report: event.target.checked })} />
            </label>
          </div>
          <div className="settings-save-row">
            {error && <p className="settings-inline-error">{error}</p>}
            <button className="btn-sm btn-primary" disabled={!hasChanges || saving} onClick={saveOperationalSettings}>
              {saving ? '保存中…' : '保存采集设置'}
            </button>
            <button className="btn-sm" disabled={cleaning} onClick={cleanupStorage}>
              {cleaning ? '清理中…' : '立即清理缓存'}
            </button>
          </div>
        </div>
      </section>

      <section className="settings-section settings-danger">
        <div className="settings-section-heading">
          <div>
            <p className="settings-kicker">LOCAL DATA</p>
            <h3>危险操作</h3>
          </div>
        </div>
        <div className="settings-card settings-danger-card">
          <div className="settings-danger-row">
            <div>
              <strong>清除所有本地数据</strong>
              <p>活动、报告、截图缓存和所有本地设置都会被删除；环境变量不会被应用修改。</p>
            </div>
            {!confirmClear ? (
              <button className="btn-sm btn-danger" onClick={() => setConfirmClear(true)}>清除数据</button>
            ) : (
              <div className="settings-danger-confirm">
                <button className="btn-sm btn-danger" disabled={saving} onClick={clearData}>确认清除</button>
                <button className="btn-sm" disabled={saving} onClick={() => setConfirmClear(false)}>取消</button>
              </div>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
