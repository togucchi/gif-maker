const ja = {
  "app.open": "動画を開く",
  "drop.title": "動画をここにドロップ",
  "drop.sub": "またはクリックしてファイルを選択（MP4 / MOV）",
  "drop.release": "離して読み込む",
  "drop.steps": "① 動画を読み込む → ② 区間を選ぶ → ③ 設定する → ④ GIF を作成",
  loading: "読み込み中…",
  "tab.video": "動画",
  "tab.gif": "GIF",
  "gif.empty": "「GIF を作成」を押すと、ここに仕上がりが表示されます",
  "gif.stale": "設定が変更されています。もう一度作成すると反映されます。",
  "player.play": "再生 (Space)",
  "player.pause": "一時停止 (Space)",
  "player.loopRange": "区間をループ",
  "proxy.making": "この動画はそのまま再生できないため、プレビュー用に変換中… {p}%",
  "timeline.start": "開始",
  "timeline.end": "終了",
  "timeline.setStart": "現在位置を開始に (I)",
  "timeline.setEnd": "現在位置を終了に (O)",
  "timeline.length": "長さ",
  "timeline.fit": "全体表示",
  "timeline.hint": "ホイールでズーム / Shift+ホイールで移動",
  "settings.title": "GIF の設定",
  "settings.preset": "プリセット",
  "preset.light": "軽量",
  "preset.standard": "標準",
  "preset.high": "高画質",
  "settings.width": "幅",
  "settings.outSize": "出力サイズ {w} × {h}",
  "settings.fps": "フレームレート",
  "settings.quality": "品質",
  "settings.qualityHint": "下げるほどファイルが小さくなります",
  "settings.loop": "ループ",
  "loop.forever": "くり返す",
  "loop.once": "1 回だけ",
  "settings.speed": "再生速度",
  "settings.source": "元動画 {w} × {h} · {fps} fps · {codec}",
  "export.estimating": "容量を計算中…",
  "export.estimate": "推定 約 {size}",
  "export.frames": "{n} フレーム",
  "export.create": "GIF を作成",
  "export.recreate": "もう一度作成",
  "export.creating": "作成中… {done} / {total}",
  "export.cancel": "キャンセル",
  "export.save": "保存…",
  "export.saved": "保存しました: {path}",
  "export.result": "完成 {size}",
  "export.longWarn": "区間が長いため、作成に時間がかかり容量も大きくなります",
  "error.title": "エラー",
} as const;

export type Key = keyof typeof ja;

const en: Record<Key, string> = {
  "app.open": "Open video",
  "drop.title": "Drop a video here",
  "drop.sub": "or click to choose a file (MP4 / MOV)",
  "drop.release": "Release to open",
  "drop.steps": "① Open a video → ② Pick a range → ③ Adjust settings → ④ Create GIF",
  loading: "Loading…",
  "tab.video": "Video",
  "tab.gif": "GIF",
  "gif.empty": "Press “Create GIF” to see the result here",
  "gif.stale": "Settings have changed. Create again to apply them.",
  "player.play": "Play (Space)",
  "player.pause": "Pause (Space)",
  "player.loopRange": "Loop selection",
  "proxy.making": "This video can’t be played directly. Preparing a preview… {p}%",
  "timeline.start": "Start",
  "timeline.end": "End",
  "timeline.setStart": "Set start to playhead (I)",
  "timeline.setEnd": "Set end to playhead (O)",
  "timeline.length": "Length",
  "timeline.fit": "Fit",
  "timeline.hint": "Wheel to zoom / Shift+wheel to pan",
  "settings.title": "GIF settings",
  "settings.preset": "Preset",
  "preset.light": "Small",
  "preset.standard": "Standard",
  "preset.high": "High quality",
  "settings.width": "Width",
  "settings.outSize": "Output {w} × {h}",
  "settings.fps": "Frame rate",
  "settings.quality": "Quality",
  "settings.qualityHint": "Lower means a smaller file",
  "settings.loop": "Loop",
  "loop.forever": "Forever",
  "loop.once": "Play once",
  "settings.speed": "Speed",
  "settings.source": "Source {w} × {h} · {fps} fps · {codec}",
  "export.estimating": "Estimating size…",
  "export.estimate": "Estimated ~{size}",
  "export.frames": "{n} frames",
  "export.create": "Create GIF",
  "export.recreate": "Create again",
  "export.creating": "Creating… {done} / {total}",
  "export.cancel": "Cancel",
  "export.save": "Save…",
  "export.saved": "Saved: {path}",
  "export.result": "Done {size}",
  "export.longWarn": "Long selection: encoding will take a while and the file will be large",
  "error.title": "Error",
};

export type Locale = "ja" | "en";
const dicts: Record<Locale, Record<Key, string>> = { ja, en };
const STORAGE_KEY = "gif-maker.locale";

function initialLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "ja" || saved === "en") return saved;
  } catch {
    /* storage unavailable */
  }
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

let current = $state<Locale>(initialLocale());

export const i18n = {
  get locale() {
    return current;
  },
  set locale(v: Locale) {
    current = v;
    document.documentElement.lang = v;
    try {
      localStorage.setItem(STORAGE_KEY, v);
    } catch {
      /* ignore */
    }
  },
};

export function t(key: Key, vars?: Record<string, string | number>): string {
  let s = dicts[current][key];
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
}
