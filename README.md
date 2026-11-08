# GIF Maker

動画（MP4 / MOV）の好きな区間を切り取って GIF に変換するデスクトップアプリです（Windows / macOS Apple Silicon）。
A desktop app to turn a section of a video into a GIF.

- 動画と完成 GIF をその場でプレビュー
- タイムラインで区間をドラッグ選択（長尺動画はズーム / 全体バーで移動）
- 幅・フレームレート・品質・ループ・速度を設定、推定ファイルサイズを常時表示
- 日本語 / English

設計は [docs/DESIGN.md](docs/DESIGN.md) を参照。

## ダウンロード / Download

[Releases](https://github.com/togucchi/gif-maker/releases/latest) から取得してください。

| OS | ファイル |
|---|---|
| Windows (x64) | `GIF.Maker_<version>_x64-setup.exe` |
| macOS (Apple Silicon) | `GIF.Maker_<version>_aarch64.dmg` |

macOS 版は未公証のため、アプリケーションフォルダへコピー後に一度だけ次を実行してください。

```bash
xattr -cr "/Applications/GIF Maker.app"
```

## リリース手順

`package.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json` のバージョンを上げ、`v<version>` タグを push すると GitHub Actions が Windows / macOS 版をビルドして Release に公開します。

## 開発環境

- Node.js 20 以上
- Rust (stable) — https://rustup.rs
- Windows: Visual Studio Build Tools（C++）と WebView2（Windows 11 は標準搭載）
- macOS: Xcode Command Line Tools

## セットアップ

```bash
npm install
```

FFmpeg（sidecar）を取得して `src-tauri/binaries/` に配置します。

```powershell
# Windows
powershell -ExecutionPolicy Bypass -File scripts/fetch-ffmpeg.ps1
```

```bash
# macOS (Apple Silicon)
./scripts/fetch-ffmpeg.sh
```

## 実行・ビルド

```bash
npm run tauri dev      # 開発起動
npm run tauri build    # インストーラ作成（Windows: NSIS / macOS: dmg）
npm run dev            # ブラウザだけで UI 確認（バックエンドはモック）
```

テスト（実際の FFmpeg で変換まで確認する場合は `--include-ignored`）:

```bash
cd src-tauri && GIF_MAKER_FFMPEG="$PWD/binaries/ffmpeg-x86_64-pc-windows-msvc.exe" cargo test -- --include-ignored
```

ショートカット: `Space` 再生/停止、`I` / `O` 開始/終了を現在位置に、`←` `→` 1 フレーム移動（Shift で 1 秒）。

## ライセンス

AGPL-3.0-or-later（GIF エンコーダ [gifski](https://gif.ski) のライセンスに準拠）。
同梱する FFmpeg は GPL ビルドです。ソースは https://ffmpeg.org から入手できます。
