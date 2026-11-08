/** 75.5 -> "01:15.50", 3725.1 -> "1:02:05.10" */
export function formatTime(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) sec = 0;
  const cs = Math.round(sec * 100);
  const h = Math.floor(cs / 360000);
  const m = Math.floor((cs % 360000) / 6000);
  const s = Math.floor((cs % 6000) / 100);
  const c = cs % 100;
  const pad = (n: number) => String(n).padStart(2, "0");
  const tail = `${pad(m)}:${pad(s)}.${pad(c)}`;
  return h > 0 ? `${h}:${tail}` : tail;
}

/** Accepts "75.5", "1:15.5", "1:02:05.1". Returns null if invalid. */
export function parseTime(text: string): number | null {
  const parts = text.trim().split(":");
  if (parts.length === 0 || parts.length > 3) return null;
  let total = 0;
  for (const p of parts) {
    if (!/^\d*(\.\d*)?$/.test(p) || p === "" || p === ".") return null;
    total = total * 60 + parseFloat(p);
  }
  return Number.isFinite(total) ? total : null;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

export function formatDuration(sec: number): string {
  return sec < 60 ? `${sec.toFixed(1)}s` : formatTime(sec);
}
