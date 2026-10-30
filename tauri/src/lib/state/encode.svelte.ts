import { api, events } from "../api";
import type { EncodeResult, EncodeStatus, JobDraft, JobPlan } from "../types";

export type EncodePhase = "idle" | "starting" | "running" | "done" | "failed" | "cancelled";

class EncodeStore {
  phase = $state<EncodePhase>("idle");
  status = $state<EncodeStatus | null>(null);
  result = $state<EncodeResult | null>(null);
  plan = $state<JobPlan | null>(null);
  error = $state<string | null>(null);
  #wired = false;

  get busy() { return this.phase === "starting" || this.phase === "running"; }

  async init() {
    if (this.#wired) return;
    this.#wired = true;
    await events.onEncodeProgress((s) => {
      if (this.busy) { this.status = s; this.phase = "running"; }
    });
    await events.onEncodeDone((r) => {
      this.result = r;
      this.phase = r.ok ? "done" : r.cancelled ? "cancelled" : "failed";
      this.error = r.error;
      if (r.cancelled) setTimeout(() => { if (this.phase === "cancelled") this.dismiss(); }, 1800);
    });
  }

  async start(draft: JobDraft) {
    if (this.busy) return;
    this.phase = "starting";
    this.status = null;
    this.result = null;
    this.error = null;
    try {
      this.plan = await api.encodeStart(draft);
    } catch (e) {
      this.phase = "failed";
      this.error = String(e);
    }
  }

  cancel() {
    if (this.busy) api.encodeCancel().catch((e) => console.warn(e));
  }

  dismiss() {
    this.phase = "idle";
    this.status = null;
    this.result = null;
    this.error = null;
  }
}

export const encoder = new EncodeStore();
