import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { createReadStream, statSync } from "node:fs";
import { spawn } from "node:child_process";
import { resolve } from "node:path";

// Dev-only: serves ./dev-media (gitignored sample clip for the browser mock) at /dev/*.
// Not a publicDir, so nothing from it ever lands in dist/.
function devMedia(): Plugin {
  const root = resolve(import.meta.dirname, "dev-media");
  const ffmpeg = process.env.VC_FFMPEG
    ?? "ffmpeg"; // from PATH
  const types: Record<string, string> = {
    mp4: "video/mp4", jpg: "image/jpeg", m3u8: "application/vnd.apple.mpegurl", ts: "video/mp2t",
  };
  return {
    name: "videocrop-dev-media",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use("/dev", (req, res, next) => {
        const name = decodeURIComponent((req.url ?? "/").split("?")[0]).replace(/^\/+/, "");
        if (!name || name.includes("..")) return next();
        if (name === "frame") {
          // Mimics the backend's /{sid}/frame?t=&w=&fast= endpoint with a real ffmpeg grab.
          const q = new URLSearchParams((req.url ?? "").split("?")[1] ?? "");
          const t = Math.max(0, Number(q.get("t") ?? 0));
          const w = Number(q.get("w") ?? 0);
          const fast = q.get("fast") !== "0";
          const src = resolve(root, "sample.mp4");
          const args = ["-hide_banner", "-loglevel", "error"];
          if (fast) args.push("-ss", String(t), "-i", src);
          else args.push("-i", src, "-ss", String(t));
          args.push("-frames:v", "1");
          if (w > 0) args.push("-vf", `scale=${w}:-2`);
          args.push("-f", "image2pipe", "-c:v", "mjpeg", "-q:v", "4", "pipe:1");
          const p = spawn(ffmpeg, args, { windowsHide: true });
          res.setHeader("Content-Type", "image/jpeg");
          res.setHeader("Cache-Control", "no-store");
          p.stdout.pipe(res);
          p.on("error", () => { res.statusCode = 500; res.end(); });
          res.on("close", () => { if (p.exitCode === null) p.kill(); });
          return;
        }
        const file = resolve(root, name);
        let size: number;
        try { size = statSync(file).size; } catch { return next(); }
        const type = types[name.split(".").pop() ?? ""] ?? "application/octet-stream";
        res.setHeader("Accept-Ranges", "bytes");
        res.setHeader("Content-Type", type);
        const m = /bytes=(\d*)-(\d*)/.exec(req.headers.range ?? "");
        if (m) {
          const start = m[1] ? +m[1] : 0;
          const end = m[2] ? Math.min(+m[2], size - 1) : size - 1;
          res.statusCode = 206;
          res.setHeader("Content-Range", `bytes ${start}-${end}/${size}`);
          res.setHeader("Content-Length", end - start + 1);
          createReadStream(file, { start, end }).pipe(res);
        } else {
          res.setHeader("Content-Length", size);
          createReadStream(file).pipe(res);
        }
      });
    },
  };
}

// Tauri expects a fixed port and no screen-clearing so Rust errors stay visible.
export default defineConfig({
  plugins: [svelte(), devMedia()],
  clearScreen: false,
  publicDir: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "chrome120",
    cssMinify: true,
    // hls.js is lazy-loaded only when the stream path is needed, so it lands in its own chunk.
    chunkSizeWarningLimit: 600,
  },
});
