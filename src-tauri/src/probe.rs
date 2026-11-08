//! Reads basic stream info by parsing `ffmpeg -i` output (no ffprobe needed).

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

use crate::ffmpeg;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub duration: f64,
    /// Display width/height (rotation already applied).
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("valid regex"))
}

pub fn probe(path: &str) -> Result<VideoInfo> {
    let output = ffmpeg::command()
        .arg("-i")
        .arg(path)
        .output()
        .context("ffmpeg を起動できません (ffmpeg not found)")?;
    // `ffmpeg -i` without an output always exits non-zero; info is on stderr.
    let text = String::from_utf8_lossy(&output.stderr);
    let mut info = parse(&text)?;
    info.path = path.to_string();
    info.file_name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    info.file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    Ok(info)
}

fn parse(text: &str) -> Result<VideoInfo> {
    static DURATION: OnceLock<Regex> = OnceLock::new();
    static VIDEO: OnceLock<Regex> = OnceLock::new();
    static SIZE: OnceLock<Regex> = OnceLock::new();
    static FPS: OnceLock<Regex> = OnceLock::new();
    static TBR: OnceLock<Regex> = OnceLock::new();
    static ROTATION: OnceLock<Regex> = OnceLock::new();

    let duration = re(&DURATION, r"Duration:\s*(\d+):(\d+):(\d+(?:\.\d+)?)")
        .captures(text)
        .map(|c| {
            let h: f64 = c[1].parse().unwrap_or(0.0);
            let m: f64 = c[2].parse().unwrap_or(0.0);
            let s: f64 = c[3].parse().unwrap_or(0.0);
            h * 3600.0 + m * 60.0 + s
        })
        .ok_or_else(|| anyhow!("動画の長さを取得できません (unsupported file)"))?;

    let line = text
        .lines()
        .find(|l| re(&VIDEO, r"Stream #\d+:\d+.*: Video: ").is_match(l))
        .ok_or_else(|| anyhow!("映像ストリームが見つかりません (no video stream)"))?;

    let codec = line
        .split("Video: ")
        .nth(1)
        .and_then(|s| s.split(|c: char| c == ' ' || c == ',').next())
        .unwrap_or("unknown")
        .to_string();

    let size = re(&SIZE, r", (\d{2,5})x(\d{2,5})")
        .captures(line)
        .ok_or_else(|| anyhow!("解像度を取得できません (unknown resolution)"))?;
    let mut width: u32 = size[1].parse()?;
    let mut height: u32 = size[2].parse()?;

    let fps = re(&FPS, r"([\d.]+)\s*fps")
        .captures(line)
        .or_else(|| re(&TBR, r"([\d.]+)\s*tbr").captures(line))
        .and_then(|c| c[1].parse::<f64>().ok())
        .filter(|f| *f > 0.0)
        .unwrap_or(30.0);

    // e.g. "displaymatrix: rotation of -90.00 degrees" (ffmpeg autorotates on decode)
    if let Some(c) = re(&ROTATION, r"rotation of (-?[\d.]+) degrees").captures(text) {
        let deg: f64 = c[1].parse().unwrap_or(0.0);
        if ((deg.abs() / 90.0).round() as i64) % 2 == 1 {
            std::mem::swap(&mut width, &mut height);
        }
    }

    Ok(VideoInfo {
        path: String::new(),
        file_name: String::new(),
        file_size: 0,
        duration,
        width,
        height,
        fps,
        codec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_output() {
        let text = r#"
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'rec.mp4':
  Duration: 00:12:03.52, start: 0.000000, bitrate: 2345 kb/s
  Stream #0:0[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 2560x1440 [SAR 1:1 DAR 16:9], 2200 kb/s, 59.94 fps, 60 tbr, 90k tbn (default)
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 128 kb/s (default)
"#;
        let info = parse(text).unwrap();
        assert!((info.duration - 723.52).abs() < 1e-6);
        assert_eq!((info.width, info.height), (2560, 1440));
        assert!((info.fps - 59.94).abs() < 1e-6);
        assert_eq!(info.codec, "h264");
    }

    #[test]
    fn swaps_dimensions_when_rotated() {
        let text = r#"
  Duration: 00:00:10.00, start: 0.000000, bitrate: 1000 kb/s
  Stream #0:0: Video: hevc (Main) (hvc1 / 0x31637668), yuv420p(tv), 1920x1080, 30 fps, 30 tbr
      Side data:
        displaymatrix: rotation of -90.00 degrees
"#;
        let info = parse(text).unwrap();
        assert_eq!((info.width, info.height), (1080, 1920));
        assert_eq!(info.codec, "hevc");
    }
}
