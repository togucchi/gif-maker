import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { api, outputSize, type EncodeProgress, type Estimate, type GifParams, type VideoInfo } from "./api";

export interface Settings {
  width: number;
  fps: number;
  quality: number;
  loopForever: boolean;
  speed: number;
}

export type PresetId = "light" | "standard" | "high";
export const PRESETS: Record<PresetId, Pick<Settings, "width" | "fps" | "quality">> = {
  light: { width: 320, fps: 10, quality: 60 },
  standard: { width: 480, fps: 15, quality: 80 },
  high: { width: 720, fps: 24, quality: 90 },
};

export const MIN_RANGE = 0.1;
export const LONG_RANGE_WARN = 30;
const THUMB_COUNT = 24;

function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}

class Project {
  info = $state<VideoInfo | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  // Playback
  videoSrc = $state("");
  video = $state<HTMLVideoElement | null>(null);
  currentTime = $state(0);
  playing = $state(false);
  loopRange = $state(true);
  proxyProgress = $state<number | null>(null);
  private triedProxy = false;

  thumbs = $state<string[]>([]);

  // Selection
  start = $state(0);
  end = $state(0);

  settings = $state<Settings>({ width: 480, fps: 15, quality: 80, loopForever: true, speed: 1 });

  // Estimate / encode
  estimate = $state<Estimate | null>(null);
  estimating = $state(false);
  encoding = $state<EncodeProgress | null>(null);
  result = $state<{ path: string; url: string; bytes: number; key: string } | null>(null);
  tab = $state<"video" | "gif">("video");
  notice = $state<string | null>(null);

  get maxWidth() {
    return this.info ? Math.max(64, this.info.width) : 1920;
  }
  get maxFps() {
    return this.info ? Math.max(1, Math.min(50, Math.round(this.info.fps))) : 30;
  }
  get rangeLength() {
    return Math.max(0, this.end - this.start);
  }
  get outSize(): [number, number] {
    const i = this.info;
    return i ? outputSize(i.width, i.height, this.settings.width) : [this.settings.width, 0];
  }
  get params(): GifParams | null {
    const i = this.info;
    if (!i) return null;
    return {
      path: i.path,
      start: this.start,
      end: this.end,
      srcWidth: i.width,
      srcHeight: i.height,
      width: this.settings.width,
      fps: this.settings.fps,
      quality: this.settings.quality,
      loopForever: this.settings.loopForever,
      speed: this.settings.speed,
    };
  }
  get paramsKey() {
    return JSON.stringify(this.params);
  }
  get resultStale() {
    return !!this.result && this.result.key !== this.paramsKey;
  }

  async load(path: string) {
    this.loading = true;
    this.error = null;
    this.notice = null;
    try {
      const info = await api.probeVideo(path);
      this.video?.pause();
      this.info = info;
      this.triedProxy = false;
      this.proxyProgress = null;
      this.videoSrc = convertFileSrc(info.path);
      this.currentTime = 0;
      this.start = 0;
      this.end = Math.min(5, info.duration);
      this.result = null;
      this.estimate = null;
      this.tab = "video";
      this.thumbs = [];
      this.applyPreset("standard");
      api
        .generateThumbnails(info.path, info.duration, THUMB_COUNT)
        .then((paths) => {
          if (this.info?.path === info.path) this.thumbs = paths.map((p) => convertFileSrc(p));
        })
        .catch(() => {});
    } catch (e) {
      this.error = errorText(e);
    } finally {
      this.loading = false;
    }
  }

  /** Called when the WebView cannot decode the file (e.g. HEVC on Windows). */
  async useProxy() {
    const info = this.info;
    if (!info || this.triedProxy) return;
    this.triedProxy = true;
    this.proxyProgress = 0;
    const unlisten = await listen<number>("proxy://progress", (e) => (this.proxyProgress = e.payload));
    try {
      const proxy = await api.prepareProxy(info.path, info.duration);
      if (this.info?.path === info.path) this.videoSrc = convertFileSrc(proxy);
    } catch (e) {
      if (this.info?.path === info.path) this.error = errorText(e);
    } finally {
      unlisten();
      this.proxyProgress = null;
    }
  }

  applyPreset(id: PresetId) {
    const p = PRESETS[id];
    this.settings.width = Math.min(p.width, this.maxWidth);
    this.settings.fps = Math.min(p.fps, this.maxFps);
    this.settings.quality = p.quality;
  }

  activePreset(): PresetId | null {
    for (const id of Object.keys(PRESETS) as PresetId[]) {
      const p = PRESETS[id];
      if (
        this.settings.width === Math.min(p.width, this.maxWidth) &&
        this.settings.fps === Math.min(p.fps, this.maxFps) &&
        this.settings.quality === p.quality
      )
        return id;
    }
    return null;
  }

  // ---- playback -------------------------------------------------------

  seek(t: number) {
    const d = this.info?.duration ?? 0;
    t = Math.min(Math.max(0, t), d);
    this.currentTime = t;
    if (this.video) this.video.currentTime = t;
  }

  togglePlay() {
    const v = this.video;
    if (!v) return;
    if (v.paused) {
      if (this.loopRange && (this.currentTime < this.start || this.currentTime >= this.end - 0.05)) {
        this.seek(this.start);
      }
      v.play().catch(() => {});
    } else {
      v.pause();
    }
  }

  step(frames: number) {
    const fps = this.info?.fps || 30;
    this.video?.pause();
    this.seek(this.currentTime + frames / fps);
  }

  setStart(t: number) {
    const d = this.info?.duration ?? 0;
    this.start = Math.min(Math.max(0, t), d - MIN_RANGE);
    if (this.end - this.start < MIN_RANGE) this.end = Math.min(d, this.start + MIN_RANGE);
  }

  setEnd(t: number) {
    const d = this.info?.duration ?? 0;
    this.end = Math.max(Math.min(d, t), MIN_RANGE);
    if (this.end - this.start < MIN_RANGE) this.start = Math.max(0, this.end - MIN_RANGE);
  }

  /** Moves the whole selection keeping its length. */
  moveRange(newStart: number) {
    const d = this.info?.duration ?? 0;
    const len = this.rangeLength;
    const s = Math.min(Math.max(0, newStart), d - len);
    this.start = s;
    this.end = s + len;
  }

  // ---- estimate / encode -------------------------------------------------

  private estimateTimer: ReturnType<typeof setTimeout> | undefined;

  scheduleEstimate() {
    clearTimeout(this.estimateTimer);
    this.notice = null;
    const params = this.params;
    if (!params || params.end <= params.start) {
      this.estimate = null;
      return;
    }
    this.estimating = true;
    this.estimateTimer = setTimeout(async () => {
      try {
        const e = await api.estimateSize(params);
        if (e) {
          this.estimate = e;
          this.estimating = false;
        } // null = superseded by a newer request; keep spinner
      } catch {
        this.estimating = false;
      }
    }, 400);
  }

  async encode() {
    const params = this.params;
    if (!params || this.encoding) return;
    const key = this.paramsKey;
    this.error = null;
    this.notice = null;
    this.encoding = { done: 0, total: 0 };
    const unlisten = await listen<EncodeProgress>("encode://progress", (e) => {
      if (this.encoding) this.encoding = e.payload;
    });
    try {
      const r = await api.encodeGif(params);
      this.result = { path: r.path, url: convertFileSrc(r.path), bytes: r.bytes, key };
      this.video?.pause();
      this.tab = "gif";
    } catch (e) {
      const msg = errorText(e);
      if (msg !== "cancelled") this.error = msg;
    } finally {
      unlisten();
      this.encoding = null;
    }
  }

  cancelEncode() {
    api.cancelEncode().catch(() => {});
  }
}

export const project = new Project();
