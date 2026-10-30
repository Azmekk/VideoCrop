// Global shortcuts. Editing fields keep their own keys except Ctrl+Enter / Ctrl+O.

import { editor } from "./state/editor.svelte";
import { cut } from "./state/cut.svelte";
import { timeline } from "./state/timeline.svelte";
import { encoder } from "./state/encode.svelte";
import { chooseAndOpen, process } from "./state/pipeline.svelte";

function isTyping(t: EventTarget | null) {
  const el = t as HTMLElement | null;
  if (!el) return false;
  if (el.isContentEditable) return true;
  if (el.tagName === "TEXTAREA" || el.tagName === "SELECT") return true;
  if (el.tagName === "INPUT") {
    const type = (el as HTMLInputElement).type;
    return type !== "checkbox" && type !== "radio" && type !== "range" && type !== "button";
  }
  return false;
}

export function handleGlobalKey(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;
  if (ctrl && e.key.toLowerCase() === "o") {
    e.preventDefault();
    void chooseAndOpen();
    return;
  }
  if (ctrl && e.key === "Enter") { e.preventDefault(); process(); return; }

  const p = editor.player;
  if (!editor.video || !p || encoder.busy || editor.mode === "crop" || isTyping(e.target)) return;
  // Space always means play/pause (as in every editor); buttons still activate with Enter.
  // Arrow keys stay with focused radio groups and sliders.
  const role = (e.target as HTMLElement | null)?.getAttribute?.("role");
  if (e.key.startsWith("Arrow") && (role === "radio" || role === "slider")) return;
  if (ctrl || e.altKey) return;

  const t = editor.displayTime;
  switch (e.key) {
    case " ": p.toggle(); break;
    case "i": case "I": cut.setStart(t); break;
    case "o": case "O": cut.setEnd(t); break;
    case "ArrowLeft": p.pause(); p.seek(e.shiftKey ? t - 1 : t - 1 / editor.fps); break;
    case "ArrowRight": p.pause(); p.seek(e.shiftKey ? t + 1 : t + 1 / editor.fps); break;
    case "Home": p.seek(cut.start); break;
    case "End": p.pause(); p.seek(cut.end - 1 / editor.fps); break;
    case "+": case "=": timeline.zoomBy(1 / 1.6); break;
    case "-": case "_": timeline.zoomBy(1.6); break;
    default: return;
  }
  e.preventDefault();
}
