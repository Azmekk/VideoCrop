import { api } from "../api";
import type { Settings } from "../types";

type Theme = Settings["theme"];

const DEFAULTS: Settings = {
  theme: "system", lastOutputDir: null, defaultCompression: null, recentFiles: [], previewEncoder: null,
};

function applyTheme(t: Theme) {
  const el = document.documentElement;
  if (t === "system") delete el.dataset.theme;
  else el.dataset.theme = t;
  try { localStorage.setItem("vc-theme", t); } catch {}
}

class SettingsStore {
  value = $state<Settings>({ ...DEFAULTS });
  loaded = $state(false);
  #saveTimer: ReturnType<typeof setTimeout> | undefined;

  async load() {
    try {
      this.value = { ...DEFAULTS, ...(await api.settingsGet()) };
      applyTheme(this.value.theme);
    } catch (e) {
      console.warn("settings_get failed", e);
    } finally {
      this.loaded = true;
    }
  }

  /** Merge + persist (coalesced so slider drags don't spam the backend). */
  update(patch: Partial<Settings>) {
    this.value = { ...this.value, ...patch };
    clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(() => {
      api.settingsSet($state.snapshot(this.value) as Settings).catch((e) => console.warn("settings_set failed", e));
    }, 300);
  }

  setTheme(t: Theme) {
    applyTheme(t);
    this.update({ theme: t });
  }
}

export const settings = new SettingsStore();
