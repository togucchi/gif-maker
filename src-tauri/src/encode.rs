//! Video segment -> GIF via ffmpeg (raw RGBA frames) and gifski.

use anyhow::{anyhow, bail, Context, Result};
use gifski::progress::ProgressReporter;
use imgref::ImgVec;
use rgb::FromSlice;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::process::Stdio;

use crate::ffmpeg;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GifParams {
    pub path: String,
    pub start: f64,
    pub end: f64,
    pub src_width: u32,
    pub src_height: u32,
    pub width: u32,
    pub fps: f64,
    pub quality: u8,
    pub loop_forever: bool,
    pub speed: f64,
}

impl GifParams {
    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    /// Output size; height follows the source aspect ratio. Mirrors
    /// `outputSize()` in the frontend.
    pub fn output_size(&self) -> (u32, u32) {
        let w = self.width.clamp(16, 4096);
        let h = if self.src_width == 0 {
            w
        } else {
            ((w as f64 * self.src_height as f64 / self.src_width as f64).round() as u32).max(1)
        };
        (w, h)
    }

    pub fn speed(&self) -> f64 {
        self.speed.clamp(0.25, 4.0)
    }

    pub fn fps(&self) -> f64 {
        self.fps.clamp(1.0, 50.0)
    }

    pub fn total_frames(&self) -> usize {
        ((self.duration() / self.speed()) * self.fps()).ceil().max(1.0) as usize
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub bytes: u64,
    pub frames: usize,
    pub width: u32,
    pub height: u32,
    /// True when the sample covered the whole selection.
    pub exact: bool,
}

struct Reporter<'a, F: FnMut(usize) + Send> {
    written: usize,
    cancelled: &'a (dyn Fn() -> bool + Sync),
    on_frame: F,
}

impl<F: FnMut(usize) + Send> ProgressReporter for Reporter<'_, F> {
    fn increase(&mut self) -> bool {
        self.written += 1;
        (self.on_frame)(self.written);
        !(self.cancelled)()
    }

    fn done(&mut self, _msg: &str) {}
}

/// Encodes `[start, start+duration)` of the source. Returns written frame count.
fn encode_range<W: Write + Send>(
    p: &GifParams,
    start: f64,
    duration: f64,
    out: W,
    cancelled: &(dyn Fn() -> bool + Sync),
    on_frame: impl FnMut(usize) + Send,
) -> Result<usize> {
    let (w, h) = p.output_size();
    let fps = p.fps();
    let speed = p.speed();
    let filter = format!("setpts=PTS/{speed},fps={fps},scale={w}:{h}:flags=lanczos");

    let mut child = ffmpeg::command()
        .args(["-loglevel", "error", "-ss", &ffmpeg::secs(start), "-t", &ffmpeg::secs(duration), "-i"])
        .arg(&p.path)
        .args(["-an", "-vf", &filter, "-pix_fmt", "rgba", "-f", "rawvideo", "pipe:1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("ffmpeg を起動できません (ffmpeg not found)")?;

    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");

    let settings = gifski::Settings {
        width: Some(w),
        height: Some(h),
        quality: p.quality.clamp(1, 100),
        fast: false,
        repeat: if p.loop_forever { gifski::Repeat::Infinite } else { gifski::Repeat::Finite(0) },
        ..Default::default()
    };
    let (collector, writer) = gifski::new(settings).map_err(|e| anyhow!("gifski: {e}"))?;

    let frame_bytes = (w as usize) * (h as usize) * 4;

    let (read_result, write_result, err_text) = std::thread::scope(|s| {
        let err_thread = s.spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        });
        let writer_thread = s.spawn(move || {
            let mut reporter = Reporter { written: 0, cancelled, on_frame };
            writer.write(out, &mut reporter).map(|_| reporter.written)
        });

        let read_result: Result<usize> = (|| {
            #[allow(unused_mut)]
            let mut collector = collector;
            let mut buf = vec![0u8; frame_bytes];
            let mut index = 0usize;
            loop {
                if cancelled() {
                    bail!("cancelled");
                }
                match stdout.read_exact(&mut buf) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                }
                let img = ImgVec::new(buf.as_rgba().to_vec(), w as usize, h as usize);
                let pts = index as f64 / fps;
                if collector.add_frame_rgba(index, img, pts).is_err() {
                    break; // writer stopped (cancelled or failed); its result tells why
                }
                index += 1;
            }
            Ok(index)
            // collector dropped here -> writer finishes
        })();

        if read_result.is_err() {
            let _ = child.kill();
        }
        let _ = child.wait();
        let write_result = writer_thread.join().expect("writer thread panicked");
        let err_text = err_thread.join().unwrap_or_default();
        (read_result, write_result, err_text)
    });

    if cancelled() {
        bail!("cancelled");
    }
    let read_frames = read_result?;
    if read_frames == 0 {
        let msg = err_text.lines().last().unwrap_or("no frames decoded").to_string();
        bail!("フレームを読み込めませんでした: {msg}");
    }
    write_result.map_err(|e| anyhow!("GIF の書き込みに失敗しました: {e}"))
}

pub fn encode<W: Write + Send>(
    p: &GifParams,
    out: W,
    cancelled: &(dyn Fn() -> bool + Sync),
    on_frame: impl FnMut(usize) + Send,
) -> Result<usize> {
    if p.duration() <= 0.0 {
        bail!("区間が空です (empty range)");
    }
    encode_range(p, p.start, p.duration(), out, cancelled, on_frame)
}

/// Encodes a short sample from the middle of the selection and extrapolates.
pub fn estimate(p: &GifParams, cancelled: &(dyn Fn() -> bool + Sync)) -> Result<Estimate> {
    const SAMPLE_OUTPUT_SECS: f64 = 2.0;
    let duration = p.duration();
    if duration <= 0.0 {
        bail!("区間が空です (empty range)");
    }
    let sample_src = (SAMPLE_OUTPUT_SECS * p.speed()).min(duration);
    let sample_start = p.start + (duration - sample_src) / 2.0;

    let mut buf: Vec<u8> = Vec::new();
    let frames = encode_range(p, sample_start, sample_src, &mut buf, cancelled, |_| {})?;
    let total = p.total_frames();
    let exact = sample_src >= duration;
    let bytes = if exact {
        buf.len() as u64
    } else {
        ((buf.len() as f64 / frames.max(1) as f64) * total as f64) as u64
    };
    let (width, height) = p.output_size();
    Ok(Estimate { bytes, frames: total, width, height, exact })
}

/// End-to-end check against a real ffmpeg. Run with:
/// `GIF_MAKER_FFMPEG=path/to/ffmpeg cargo test -- --ignored`
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_video() -> String {
        let path = std::env::temp_dir().join("gif-maker-test-src.mp4");
        if !path.is_file() {
            let ok = ffmpeg::command()
                .args(["-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=30:duration=8"])
                .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-y"])
                .arg(&path)
                .status()
                .expect("run ffmpeg")
                .success();
            assert!(ok, "failed to create sample video");
        }
        path.to_string_lossy().into_owned()
    }

    fn params(path: String) -> GifParams {
        GifParams {
            path,
            start: 2.0,
            end: 5.0,
            src_width: 1280,
            src_height: 720,
            width: 320,
            fps: 10.0,
            quality: 80,
            loop_forever: true,
            speed: 1.0,
        }
    }

    #[test]
    #[ignore]
    fn encodes_real_video() {
        let p = params(sample_video());
        let info = crate::probe::probe(&p.path).unwrap();
        assert_eq!((info.width, info.height), (1280, 720));
        assert!((info.duration - 8.0).abs() < 0.1);

        let mut out = Vec::new();
        let frames = encode(&p, &mut out, &|| false, |_| {}).unwrap();
        assert_eq!(frames, 30);
        assert_eq!(&out[..6], b"GIF89a");
        // logical screen width/height
        assert_eq!(u16::from_le_bytes([out[6], out[7]]), 320);
        assert_eq!(u16::from_le_bytes([out[8], out[9]]), 180);

        let est = estimate(&p, &|| false).unwrap();
        assert_eq!(est.frames, 30);
        let ratio = est.bytes as f64 / out.len() as f64;
        eprintln!("actual {} bytes, estimate {} bytes (x{ratio:.2})", out.len(), est.bytes);
        assert!((0.3..3.0).contains(&ratio));
    }

    #[test]
    #[ignore]
    fn cancel_stops_encoding() {
        let p = GifParams { start: 0.0, end: 8.0, ..params(sample_video()) };
        let err = encode(&p, std::io::sink(), &|| true, |_| {}).unwrap_err();
        assert_eq!(err.to_string(), "cancelled");
    }
}
