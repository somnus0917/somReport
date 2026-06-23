import { create } from 'zustand';
import type { AppSettings } from '../lib/types';
import { DEFAULT_SETTINGS } from '../lib/constants';
import {
  getSettings,
  saveSettings,
  saveProviderKey,
  testProviderKey,
  clearAllData,
} from '../api/tauri';

type TestStatus = 'idle' | 'testing' | 'success' | 'fail';

interface SettingsStore {
  settings: AppSettings;
  loading: boolean;
  saving: boolean;
  dirty: boolean;
  testing: Record<string, TestStatus>;
  keyInputs: Record<string, string>;

  fetchSettings: () => Promise<void>;
  updateField: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
  updateProvider: (
    which: 'vision_provider' | 'text_provider',
    field: keyof AppSettings['vision_provider'],
    value: string | null,
  ) => void;
  setKeyInput: (service: string, value: string) => void;
  save: () => Promise<void>;
  saveKey: (service: string) => Promise<void>;
  testKey: (service: string) => Promise<void>;
  clearData: () => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: { ...DEFAULT_SETTINGS },
  loading: true,
  saving: false,
  dirty: false,
  testing: {},
  keyInputs: {},

  fetchSettings: async () => {
    const settings = await getSettings();
    set({ settings, loading: false, dirty: false });
  },

  updateField: (key, value) => {
    set((s) => ({ settings: { ...s.settings, [key]: value }, dirty: true }));
  },

  updateProvider: (which, field, value) => {
    set((s) => ({
      settings: {
        ...s.settings,
        [which]: { ...s.settings[which], [field]: value },
      },
      dirty: true,
    }));
  },

  setKeyInput: (service, value) => {
    set((s) => ({ keyInputs: { ...s.keyInputs, [service]: value } }));
  },

  save: async () => {
    set({ saving: true });
    try {
      await saveSettings(get().settings);
      set({ dirty: false });
    } finally {
      set({ saving: false });
    }
  },

  saveKey: async (service) => {
    const key = get().keyInputs[service];
    if (!key) return;
    await saveProviderKey(service, key);
    set((s) => ({
      keyInputs: { ...s.keyInputs, [service]: '' },
    }));
  },

  testKey: async (service) => {
    set((s) => ({ testing: { ...s.testing, [service]: 'testing' } }));
    try {
      const ok = await testProviderKey(service);
      set((s) => ({ testing: { ...s.testing, [service]: ok ? 'success' : 'fail' } }));
    } catch {
      set((s) => ({ testing: { ...s.testing, [service]: 'fail' } }));
    }
  },

  clearData: async () => {
    await clearAllData();
  },
}));
