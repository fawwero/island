// store/settings.ts — клиентский стор настроек (Zustand) поверх IPC-контракта.
// Кэширует settings_all, подписывается на settings:changed (единый поток истины — Rust).

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { SettingsChanged, AppErrorPayload } from "../types";
import { isAppErrorPayload } from "../types";

export type SettingsMap = Record<string, unknown>;

interface SettingsState {
  values: SettingsMap;
  loaded: boolean;
  load: () => Promise<void>;
  set: (key: string, value: unknown) => Promise<boolean>;
  subscribeChanges: () => Promise<void>;
}

export const useSettings = create<SettingsState>((set, get) => ({
  values: {},
  loaded: false,

  async load() {
    try {
      const all = await invoke<SettingsMap>("settings_all");
      set({ values: all, loaded: true });
    } catch (e) {
      console.error("settings_all failed", describeError(e));
      set({ loaded: true });
    }
  },

  async set(key, value) {
    // оптимистично не меняем: Rust — источник истины, дождёмся события
    return invoke<boolean>("settings_set", { key, value });
  },

  async subscribeChanges() {
    await listen<SettingsChanged>("settings:changed", (ev) => {
      const { key, value } = ev.payload;
      if (key === "*") {
        // сброс к умолчаниям — полная перезагрузка кэша
        void get().load();
        return;
      }
      set((s) => ({ values: { ...s.values, [key]: value } }));
    });
  },
}));

export function describeError(e: unknown): string {
  if (isAppErrorPayload(e)) {
    const p = e as AppErrorPayload;
    return `${p.code}: ${p.messageKey} ${JSON.stringify(p.params)}`;
  }
  return String(e);
}
