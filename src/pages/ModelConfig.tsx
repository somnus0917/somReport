import { useEffect, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  saveAndTestModel,
  saveModelConfig,
  type TestResult,
} from '../api/tauri';
import { LoadingState } from '../components/StateViews';
import { PROVIDERS, providerDefaults } from '../lib/constants';
import type { ProviderConfig } from '../lib/types';
import { useInvalidateSettings, useSettings } from '../hooks/useSettings';

type ModelRole = 'vision' | 'text';

const ROLE_COPY: Record<ModelRole, { kicker: string; title: string; description: string; testLabel: string }> = {
  vision: {
    kicker: 'SCREENSHOT ANALYSIS',
    title: '视觉模型配置',
    description: '用于理解截图中的工作活动。保存并测试会发送一张小型测试图片。',
    testLabel: '保存并测试视觉模型',
  },
  text: {
    kicker: 'REPORT GENERATION',
    title: '文本模型配置',
    description: '用于将本地聚合的活动整理为日报或周报。测试会发送一条极短的文本请求。',
    testLabel: '保存并测试文本模型',
  },
};

function isRole(value: string | undefined): value is ModelRole {
  return value === 'vision' || value === 'text';
}

export default function ModelConfig() {
  const { role: rawRole } = useParams();
  const navigate = useNavigate();
  const { data: settings, isLoading } = useSettings();
  const invalidateSettings = useInvalidateSettings();
  const role = isRole(rawRole) ? rawRole : null;
  const [config, setConfig] = useState<ProviderConfig | null>(null);
  const [action, setAction] = useState<'save' | 'test' | null>(null);
  const [result, setResult] = useState<TestResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!settings || !role || config) return;
    setConfig(role === 'vision' ? settings.vision_provider : settings.text_provider);
  }, [settings, role, config]);

  if (!role) {
    return (
      <div className="settings-page model-config-page">
        <p className="settings-inline-error">未知的模型配置类型。</p>
        <Link className="btn-sm" to="/settings">返回设置</Link>
      </div>
    );
  }

  if (isLoading || !settings || !config) {
    return <div className="settings-page"><LoadingState message="正在读取模型配置…" /></div>;
  }

  const activeRole: ModelRole = role;
  const activeConfig: ProviderConfig = config;
  const copy = ROLE_COPY[activeRole];
  const persistedStatus = activeRole === 'vision' ? settings.vision_connection : settings.text_connection;
  const visibleResult = result ?? (persistedStatus.success === null ? null : {
    success: persistedStatus.success,
    message: persistedStatus.message ?? '已完成测试',
    response: null,
  });

  function updateProvider(name: string) {
    const defaults = providerDefaults(activeRole, name);
    setConfig({ ...activeConfig, name, ...defaults });
    setResult(null);
  }

  async function save(shouldTest: boolean) {
    setAction(shouldTest ? 'test' : 'save');
    setError(null);
    setResult(null);
    try {
      if (shouldTest) {
        const testResult = await saveAndTestModel(activeRole, activeConfig);
        setResult(testResult);
      } else {
        await saveModelConfig(activeRole, activeConfig);
        setResult({ success: true, message: '配置已保存，尚未进行实时测试。', response: null });
      }
      await invalidateSettings();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : '保存模型配置失败');
    } finally {
      setAction(null);
    }
  }

  return (
    <div className="settings-page model-config-page">
      <header className="model-config-header">
        <Link className="back-link" to="/settings">← 返回设置</Link>
        <p className="settings-kicker">{copy.kicker}</p>
        <h2>{copy.title}</h2>
        <p>{copy.description}</p>
      </header>

      <section className="settings-card model-config-card">
        <div className="model-config-status-row">
          <div>
            <span className="model-config-label">密钥来源</span>
            <strong>{config.api_key_env_var ?? '未设置环境变量'}</strong>
          </div>
          <div>
            <span className="model-config-label">上次验证</span>
            <strong>{persistedStatus.tested_at ? new Date(persistedStatus.tested_at).toLocaleString('zh-CN', { hour12: false }) : '从未测试'}</strong>
          </div>
        </div>

        <div className="settings-input-grid">
          <label>
            服务商
            <select value={config.name} onChange={(event) => updateProvider(event.target.value)}>
              {PROVIDERS.map((provider) => <option key={provider.id} value={provider.id}>{provider.label}</option>)}
            </select>
          </label>
          <label>
            模型名称
            <input value={config.model} onChange={(event) => setConfig({ ...config, model: event.target.value })} />
          </label>
        </div>

        <label>
          API 地址
          <input value={config.api_url} onChange={(event) => setConfig({ ...config, api_url: event.target.value })} />
        </label>

        <div className="settings-input-grid">
          <label>
            输入价格（分 / 100 万 tokens）
            <input
              type="number"
              min={0}
              step="0.01"
              value={config.input_cost_per_million_cents}
              onChange={(event) => setConfig({ ...config, input_cost_per_million_cents: Number(event.target.value) })}
            />
          </label>
          <label>
            输出价格（分 / 100 万 tokens）
            <input
              type="number"
              min={0}
              step="0.01"
              value={config.output_cost_per_million_cents}
              onChange={(event) => setConfig({ ...config, output_cost_per_million_cents: Number(event.target.value) })}
            />
          </label>
        </div>

        <div className="model-env-note">
          <strong>密钥只从环境变量读取</strong>
          <p>
            本应用不会保存 API 密钥。请在启动应用前设置 <code>{config.api_key_env_var ?? '对应环境变量'}</code>，然后使用下方按钮测试当前进程能否读取并调用模型。
          </p>
          {(config.input_cost_per_million_cents === 0 && config.output_cost_per_million_cents === 0) && (
            <p>费用估算当前为 0；请按服务商账单填写输入和输出单价后，预算与费用才会生效。</p>
          )}
        </div>

          <div className="model-config-actions">
          <button className="btn-sm btn-primary" disabled={action !== null} onClick={() => save(true)}>
            {action === 'test' ? '正在保存并测试…' : copy.testLabel}
          </button>
          <button className="btn-sm" disabled={action !== null} onClick={() => save(false)}>
            {action === 'save' ? '正在保存…' : '仅保存'}
          </button>
          <button className="btn-sm model-config-cancel" disabled={action !== null} onClick={() => navigate('/settings')}>取消</button>
        </div>
      </section>

      {(visibleResult || error) && (
        <section className={`model-test-result ${error || visibleResult?.success === false ? 'error' : 'success'}`}>
          <p className="settings-kicker">LIVE CHECK</p>
          <h3>{error ?? visibleResult?.message}</h3>
          {!error && visibleResult?.response && <pre>{visibleResult.response}</pre>}
          {!error && visibleResult?.success === false && <p>配置已保存，但该模型暂时不可用。请检查模型名、地址、密钥权限或账户额度。</p>}
        </section>
      )}
    </div>
  );
}
