import { useEffect, useState } from 'react';
import { useSettingsStore } from '../stores/settings';
import { PROVIDERS } from '../lib/constants';

function ProviderSection({
  which,
  label,
}: {
  which: 'vision_provider' | 'text_provider';
  label: string;
}) {
  const provider = useSettingsStore((s) => s.settings[which]);
  const updateProvider = useSettingsStore((s) => s.updateProvider);
  const testing = useSettingsStore((s) => s.testing[provider.name] ?? 'idle');
  const keyInput = useSettingsStore((s) => s.keyInputs[provider.name] ?? '');
  const setKeyInput = useSettingsStore((s) => s.setKeyInput);
  const saveKey = useSettingsStore((s) => s.saveKey);
  const testKey = useSettingsStore((s) => s.testKey);

  return (
    <div className="settings-card">
      <h3>{label}</h3>
      <label>
        Provider
        <select
          value={provider.name}
          onChange={(e) => updateProvider(which, 'name', e.target.value)}
        >
          {PROVIDERS.map((p) => (
            <option key={p.id} value={p.id}>
              {p.label}
            </option>
          ))}
        </select>
      </label>
      <label>
        Model
        <input
          type="text"
          value={provider.model}
          onChange={(e) => updateProvider(which, 'model', e.target.value)}
        />
      </label>
      <label>
        API URL
        <input
          type="text"
          value={provider.api_url}
          onChange={(e) => updateProvider(which, 'api_url', e.target.value)}
        />
      </label>
      <div className="settings-key-row">
        <label className="settings-key-label">
          API Key
          <div className="settings-key-input-group">
            <input
              type="password"
              placeholder="Enter new key…"
              value={keyInput}
              onChange={(e) => setKeyInput(provider.name, e.target.value)}
            />
            <button className="btn-sm btn-primary" onClick={() => saveKey(provider.name)}>
              Save
            </button>
            <button
              className="btn-sm"
              onClick={() => testKey(provider.name)}
              disabled={testing === 'testing'}
            >
              {testing === 'testing'
                ? 'Testing…'
                : testing === 'success'
                  ? '✓ OK'
                  : testing === 'fail'
                    ? '✗ Fail'
                    : 'Test'}
            </button>
          </div>
        </label>
        {provider.api_key_env_var && (
          <span className="settings-key-hint">
            Env fallback: {provider.api_key_env_var}
          </span>
        )}
      </div>
    </div>
  );
}

export default function Settings() {
  const { settings, loading, saving, dirty, fetchSettings, updateField, save, clearData } =
    useSettingsStore();
  const [confirmClear, setConfirmClear] = useState(false);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  if (loading) return <div className="settings-page"><p>Loading…</p></div>;

  return (
    <div className="settings-page">
      <header className="settings-header">
        <h2>Settings</h2>
        <button className="btn-sm btn-primary" disabled={!dirty || saving} onClick={save}>
          {saving ? 'Saving…' : 'Save Changes'}
        </button>
      </header>

      <section className="settings-section">
        <h3>API Providers</h3>
        <div className="settings-provider-grid">
          <ProviderSection which="vision_provider" label="Vision (Screenshots)" />
          <ProviderSection which="text_provider" label="Text (Reports)" />
        </div>
      </section>

      <section className="settings-section">
        <h3>Capture</h3>
        <div className="settings-card">
          <label>
            Capture interval (seconds)
            <input
              type="number"
              min={5}
              value={settings.capture_interval_secs}
              onChange={(e) => updateField('capture_interval_secs', Number(e.target.value))}
            />
          </label>
          <label>
            Idle timeout (seconds)
            <input
              type="number"
              min={30}
              value={settings.idle_timeout_secs}
              onChange={(e) => updateField('idle_timeout_secs', Number(e.target.value))}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <h3>Budget</h3>
        <div className="settings-card">
          <label>
            Daily budget (cents)
            <input
              type="number"
              min={0}
              value={settings.max_daily_cost_cents}
              onChange={(e) => updateField('max_daily_cost_cents', Number(e.target.value))}
            />
          </label>
          <span className="settings-hint">
            ≈ ${(settings.max_daily_cost_cents / 100).toFixed(2)} / day
          </span>
        </div>
      </section>

      <section className="settings-section settings-danger">
        <h3>Danger Zone</h3>
        <div className="settings-card settings-danger-card">
          <div className="settings-danger-row">
            <div>
              <strong>Clear All Local Data</strong>
              <p className="settings-hint">
                Deletes all activities, reports, screenshots, and settings. This cannot be undone.
              </p>
            </div>
            {!confirmClear ? (
              <button className="btn-sm btn-danger" onClick={() => setConfirmClear(true)}>
                Clear All Data
              </button>
            ) : (
              <div className="settings-danger-confirm">
                <button
                  className="btn-sm btn-danger"
                  onClick={async () => {
                    await clearData();
                    setConfirmClear(false);
                    fetchSettings();
                  }}
                >
                  Confirm Delete
                </button>
                <button className="btn-sm" onClick={() => setConfirmClear(false)}>
                  Cancel
                </button>
              </div>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
