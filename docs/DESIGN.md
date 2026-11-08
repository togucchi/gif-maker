# GIF Maker 設計書

動画（MP4 / MOV）の任意区間を GIF に変換するデスクトップアプリ。

## 1. 決定事項

| 項目 | 決定 |
|---|---|
| 対象 OS | Windows (x64) / macOS (Apple Silicon のみ) |
| 技術 | Rust + Tauri 2、フロントは Svelte 5 + TypeScript + Vite |
| 公開形態 | OSS（gifski が AGPL-3.0 のため本アプリも **AGPL-3.0-or-later**） |
| 動画デコード | FFmpeg を sidecar として同梱（`ffmpeg` のみ。`ffprobe` は同梱せず `ffmpeg -i` の出力を解析） |
| GIF エンコード | gifski crate（フレームごとの最適パレット、高画質） |
| 入力 | MP4 / MOV。主に画面録画、数分〜数十分の長尺もあり |
| 出力 | GIF のみ |
| UI 言語 | 日本語 / 英語（OS 言語で初期選択、手動切替・記憶） |
| スコープ外（当面） | 目標ファイルサイズ指定、クロップ・テキスト等の編集、一括変換、他形式出力 |

## 2. アーキテクチャ

```
Frontend (Svelte, WebView)
  DropZone ─ Preview(動画/GIF タブ) ─ Timeline ─ SettingsPanel ─ ExportBar
        │ invoke()                         ▲ events (encode://progress, proxy://progress)
Backend (Rust)
  commands.rs   … Tauri コマンド定義
  ffmpeg.rs     … sidecar 解決・プロセス起動
  probe.rs      … `ffmpeg -i` の stderr を解析（尺 / 解像度 / fps / コーデック / 回転）
  thumbs.rs     … タイムライン用サムネイル（高速シーク -ss で N 枚）
  proxy.rs      … WebView で再生できない動画を低解像度 H.264 に変換
  encode.rs     … ffmpeg(rawvideo RGBA) → gifski → GIF、容量見積もり
  job.rs        … キャンセル（AtomicBool）・見積もりの世代管理
```

### 変換パイプライン

```
ffmpeg -ss {start} -t {duration} -i input -an
       -vf "setpts=PTS/{speed},fps={fps},scale={w}:{h}:flags=lanczos"
       -pix_fmt rgba -f rawvideo pipe:1
   └─ stdout を w*h*4 バイトずつ読み → gifski::Collector に投入
                               別スレッドで gifski::Writer → 一時ファイル
```

- 出力サイズは Rust 側で計算（偶数に丸め、回転メタデータを考慮）し、rawvideo のフレーム境界を確定させる。
- 進捗は gifski の `ProgressReporter` で書き出し済みフレーム数を通知。キャンセル時は reporter が `false` を返し、ffmpeg を kill。
- 出力はまずアプリのキャッシュディレクトリに作成し、GIF タブでプレビュー後「保存」で任意の場所へコピー。

### 容量見積もり

- 選択区間の中央から最大 2 秒をサンプルとして同じ設定でメモリ上にエンコード。
- `サンプルサイズ / サンプルフレーム数 × 総フレーム数` で推定。
- フロントで 400ms デバウンス。バックエンドは世代番号で古い見積もりを破棄。

### プレビュー

- 動画: asset プロトコル（`convertFileSrc`）で `<video>` 再生。開いたファイルだけを実行時に scope へ許可。
- `<video>` が error を出したら（例: Windows で HEVC 拡張なし）プロキシ（短辺 540px / H.264 ultrafast / GOP 短め）を生成して差し替え。進捗表示あり。
- GIF: キャッシュの出力ファイルを `<img>` で表示。生成後に設定を変えたら「設定が変更されています」と表示。

### 長尺動画への配慮

- 尺の取得・サムネイル生成は入力シーク（`-ss` を `-i` の前）で全デコードしない。
- タイムラインはホイールでズーム（カーソル位置中心）、ドラッグでパン、「全体表示」でリセット。
- 再生ヘッド位置から `I` / `O` キー（またはボタン）で開始・終了を設定できる。数値入力で ±0.1 秒の微調整も可能。
- 選択区間が長い（30 秒超）場合は容量が大きくなる旨を警告（ブロックはしない）。

## 3. UI

```
┌──────────────────────────────────────────────────────────┐
│ GIF Maker                         [開く]  [日本語 | EN]  │
├───────────────────────────────────┬──────────────────────┤
│  [ 動画 | GIF ]                   │ プリセット            │
│                                   │ [軽量][標準][高画質] │
│          プレビュー               │ 幅 ━━━●━━  480 px    │
│                                   │   → 480×270          │
│  ▶  00:12.40 / 03:20.00   🔁区間  │ FPS ━━●━━━  15       │
├───────────────────────────────────┤ 品質 ━━━━●━  80      │
│ [サムネイル帯 + 選択範囲 + 再生ヘッド]│ ループ [無限 ▾]    │
│ 開始 [00:05.00] [I]  終了 [00:09.50] [O] │ 速度 [1.0x ▾]  │
├───────────────────────────────────┴──────────────────────┤
│ 推定 約 3.2 MB · 480×270 · 15fps · 68 フレーム · 4.5 秒  │
│                          [ GIF を作成 ]  [ 保存… ]       │
└──────────────────────────────────────────────────────────┘
```

- 初期状態は画面全体がドロップエリア（「動画をドロップ、またはクリックして選択」）。
- 操作の流れが 上（見る）→ 中（切り取る）→ 右（設定）→ 下（作成・保存）の順に並ぶ。
- ショートカット: `Space` 再生/停止、`I` / `O` 開始/終了設定、`←` `→` 1 フレーム移動（Shift で 1 秒）。

### 設定項目

| 項目 | 範囲 | 既定 |
|---|---|---|
| プリセット | 軽量 (320px/10fps/q60)・標準 (480px/15fps/q80)・高画質 (720px/24fps/q90) | 標準 |
| 幅 | 64 〜 元動画の幅（高さは縦横比から自動） | 480 |
| FPS | 1 〜 min(30, 元 fps) | 15 |
| 品質 | 1 〜 100（gifski quality） | 80 |
| ループ | 無限 / 1 回のみ | 無限 |
| 速度 | 0.5x / 1x / 1.5x / 2x | 1x |

## 4. Tauri コマンド

| コマンド | 内容 |
|---|---|
| `probe_video(path)` | 動画情報を返し、asset scope にファイルを許可 |
| `generate_thumbnails(path, duration, count)` | サムネイルのパス一覧 |
| `prepare_proxy(path)` | プロキシ動画のパス（`proxy://progress` で進捗） |
| `estimate_size(params)` | 推定バイト数 |
| `encode_gif(params)` | 一時 GIF のパス・サイズ（`encode://progress` で進捗） |
| `cancel_encode()` | 変換キャンセル |
| `save_gif(src, dest)` | 一時 GIF を保存先へコピー |

## 5. ビルド・配布

- `src-tauri/binaries/ffmpeg-<target-triple>[.exe]` に FFmpeg を配置（`scripts/fetch-ffmpeg.*` で取得）。
  - Windows: `x86_64-pc-windows-msvc`、macOS: `aarch64-apple-darwin`
- Windows: NSIS インストーラ。macOS: `.dmg`（署名・公証は配布時に設定）。
- 同梱 FFmpeg のライセンス表記（GPL ビルドの場合はソース入手先）を About / README に記載する。

## 6. マイルストーン

1. **MVP**（本リポジトリの初期実装）: 読み込み・プレビュー・区間選択・設定・見積もり・変換・保存・日英切替
2. 使い勝手: 設定の記憶、最近使ったファイル、変換完了通知
3. 配布: CI（GitHub Actions で Windows / macOS arm64 ビルド）、署名・公証、ライセンス表記画面
