import { api } from "../api";
import type { CompressionChoice, CompressionSpec, JobPlan, OutputMode, PresetCatalog } from "../types";
import { settings } from "./settings.svelte";

export type ExportTab = "share" | "quality" | "custom";

export const DEFAULT_CUSTOM: CompressionSpec = {
  codec: "h264", rateControl: "crf", crf: 23, targetBitrateBps: 5_000_000, targetSizeBytes: 50 * 1024 * 1024,
  speed: "medium", audioCodec: "aac", audioBitrateKbps: 128, pixelFormat: "yuv420p", container: null,
};

/** Compression + output choices, and the latest plan from the backend. */
class ExportStore {
  catalog = $state<PresetCatalog | null>(null);
  tab = $state<ExportTab>("quality");
  reencode = $state(true);
  platformId = $state<string | null>(null);
  qualityId = $state<string | null>(null);
  custom = $state<CompressionSpec>({ ...DEFAULT_CUSTOM });
  mode = $state<OutputMode>("everything");
  outputDir = $state<string | null>(null);

  plan = $state<JobPlan | null>(null);
  planError = $state<string | null>(null);

  /** False while the Share tab is open with no destination picked. */
  get ready() { return !(this.reencode && this.tab === "share" && !this.platformId); }

  get choice(): CompressionChoice {
    if (!this.reencode) return { kind: "off" };
    if (this.tab === "share" && this.platformId) return { kind: "platform", platformId: this.platformId };
    if (this.tab === "custom") return { kind: "custom", spec: $state.snapshot(this.custom) as CompressionSpec };
    return { kind: "quality", presetId: this.qualityId ?? this.catalog?.defaultQualityId ?? "" };
  }

  async loadCatalog() {
    if (this.catalog) return;
    try {
      this.catalog = await api.presets();
      this.qualityId ??= this.catalog.defaultQualityId;
    } catch (e) {
      console.warn("presets failed", e);
    }
  }

  /** Restores the last choice saved in settings. */
  restore() {
    const s = settings.value;
    this.outputDir = s.lastOutputDir;
    const c = s.defaultCompression;
    if (!c) return;
    if (c.kind === "off") this.reencode = false;
    else if (c.kind === "platform") { this.tab = "share"; this.platformId = c.platformId; }
    else if (c.kind === "quality") { this.tab = "quality"; this.qualityId = c.presetId; }
    else { this.tab = "custom"; this.custom = { ...DEFAULT_CUSTOM, ...c.spec }; }
  }

  persist() {
    settings.update({ defaultCompression: this.choice, lastOutputDir: this.outputDir });
  }

  selectPlatform(id: string) { this.platformId = id; this.tab = "share"; this.reencode = true; this.persist(); }
  selectQuality(id: string) { this.qualityId = id; this.tab = "quality"; this.reencode = true; this.persist(); }
  patchCustom(p: Partial<CompressionSpec>) { this.custom = { ...this.custom, ...p }; this.persist(); }

  /** Seed the custom tab from whatever is currently selected so switching tabs feels continuous. */
  seedCustomFromSelection() {
    const c = this.catalog;
    if (!c) return;
    const src = this.tab === "share"
      ? c.platforms.find((p) => p.id === this.platformId)?.spec
      : c.quality.find((q) => q.id === this.qualityId)?.spec;
    if (src) this.custom = { ...this.custom, ...src };
  }
}

export const exporter = new ExportStore();
