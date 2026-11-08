import { invoke } from "@tauri-apps/api/core";

export interface VideoInfo {
  path: string;
  fileName: string;
  fileSize: number;
  duration: number;
  width: number;
  height: number;
  fps: number;
  codec: string;
}

export interface GifParams {
  path: string;
  start: number;
  end: number;
  srcWidth: number;
  srcHeight: number;
  width: number;
  fps: number;
  quality: number;
  loopForever: boolean;
  speed: number;
}

export interface Estimate {
  bytes: number;
  frames: number;
  width: number;
  height: number;
  exact: boolean;
}

export interface EncodeResult {
  path: string;
  bytes: number;
  frames: number;
}

export interface EncodeProgress {
  done: number;
  total: number;
}

export const api = {
  probeVideo: (path: string) => invoke<VideoInfo>("probe_video", { path }),
  generateThumbnails: (path: string, duration: number, count: number) =>
    invoke<string[]>("generate_thumbnails", { path, duration, count }),
  prepareProxy: (path: string, duration: number) =>
    invoke<string>("prepare_proxy", { path, duration }),
  estimateSize: (params: GifParams) => invoke<Estimate | null>("estimate_size", { params }),
  encodeGif: (params: GifParams) => invoke<EncodeResult>("encode_gif", { params }),
  cancelEncode: () => invoke<void>("cancel_encode"),
  saveGif: (src: string, dest: string) => invoke<void>("save_gif", { src, dest }),
};

/** Must match `GifParams::output_size` in Rust. */
export function outputSize(srcW: number, srcH: number, width: number): [number, number] {
  const w = Math.min(4096, Math.max(16, Math.round(width)));
  const h = srcW === 0 ? w : Math.max(1, Math.round((w * srcH) / srcW));
  return [w, h];
}
