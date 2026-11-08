// Browser-only mock of the Tauri backend so the UI can be developed with
// `npm run dev` outside the app shell. Never loaded inside Tauri.
import { mockConvertFileSrc, mockIPC, mockWindows } from "@tauri-apps/api/mocks";

export function installDevMock() {
  mockWindows("main");
  mockConvertFileSrc("windows");
  mockIPC((cmd, args) => {
    const a = args as Record<string, any>;
    switch (cmd) {
      case "probe_video":
        return {
          path: a.path,
          fileName: "screen-recording.mp4",
          fileSize: 123_456_789,
          duration: 754.3,
          width: 2560,
          height: 1440,
          fps: 60,
          codec: "h264",
        };
      case "generate_thumbnails":
        return [];
      case "estimate_size": {
        const p = a.params;
        const frames = Math.ceil(((p.end - p.start) / p.speed) * p.fps);
        const h = Math.round((p.width * p.srcHeight) / p.srcWidth);
        return { bytes: Math.round(frames * p.width * h * 0.12 * (p.quality / 100)), frames, width: p.width, height: h, exact: false };
      }
      case "plugin:dialog|open":
        return "C:/videos/screen-recording.mp4";
      default:
        return null;
    }
  }, { shouldMockEvents: true });
}
