//! Thumbnail strip and preview-proxy generation.

use anyhow::{bail, Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::ffmpeg;

/// Stable per-file cache key (path + size + mtime).
pub fn cache_key(path: &str) -> String {
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    if let Ok(meta) = std::fs::metadata(path) {
        meta.len().hash(&mut h);
        if let Ok(modified) = meta.modified() {
            modified.hash(&mut h);
        }
    }
    format!("{:016x}", h.finish())
}

/// Extracts `count` evenly spaced frames using fast input seeking, so long
/// recordings never need a full decode.
pub fn thumbnails(path: &str, duration: f64, count: usize, dir: &Path) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let count = count.clamp(1, 60);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let file = dir.join(format!("{i:02}.jpg"));
        if !file.is_file() {
            let t = duration * (i as f64 + 0.5) / count as f64;
            let status = ffmpeg::command()
                .args(["-loglevel", "error", "-ss", &ffmpeg::secs(t), "-i"])
                .arg(path)
                .args(["-frames:v", "1", "-vf", "scale=-2:90", "-q:v", "5", "-y"])
                .arg(&file)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .context("ffmpeg を起動できません")?;
            if !status.success() || !file.is_file() {
                continue; // a missing tile is not fatal
            }
        }
        out.push(file);
    }
    Ok(out)
}

/// Transcodes to a small H.264 file the WebView can always play.
/// `on_progress` receives 0.0..=1.0; returning `false` aborts.
pub fn proxy(
    path: &str,
    duration: f64,
    dest: &Path,
    mut on_progress: impl FnMut(f64) -> bool,
) -> Result<PathBuf> {
    if dest.is_file() {
        return Ok(dest.to_path_buf());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("part.mp4");
    let mut child = ffmpeg::command()
        .args(["-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-an",
            "-vf",
            "scale=-2:'min(540,ih)':flags=bilinear",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "30",
            "-g",
            "15",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-progress",
            "pipe:1",
            "-y",
        ])
        .arg(&tmp)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("ffmpeg を起動できません")?;

    let stdout = child.stdout.take().expect("piped stdout");
    let mut aborted = false;
    for line in BufReader::new(stdout).lines().map_while(|l| l.ok()) {
        if let Some(us) = line.strip_prefix("out_time_us=") {
            if let Ok(us) = us.trim().parse::<f64>() {
                let p = if duration > 0.0 { (us / 1e6 / duration).clamp(0.0, 1.0) } else { 0.0 };
                if !on_progress(p) {
                    aborted = true;
                    let _ = child.kill();
                    break;
                }
            }
        }
    }
    let status = child.wait()?;
    if aborted {
        let _ = std::fs::remove_file(&tmp);
        bail!("cancelled");
    }
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        bail!("プレビュー用動画の作成に失敗しました (proxy failed)");
    }
    std::fs::rename(&tmp, dest)?;
    Ok(dest.to_path_buf())
}
