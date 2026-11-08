//! Tauri commands exposed to the frontend.

use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use crate::encode::{self, Estimate, GifParams};
use crate::media;
use crate::probe::{self, VideoInfo};

#[derive(Default)]
pub struct AppState {
    encode_cancel: Mutex<Arc<AtomicBool>>,
    proxy_cancel: Mutex<Arc<AtomicBool>>,
    estimate_generation: AtomicU64,
}

type CmdResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn cache_dir(app: &AppHandle) -> CmdResult<PathBuf> {
    app.path().app_cache_dir().map_err(err)
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> CmdResult<T> + Send + 'static,
) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(err)?
}

/// Call `f` at most every `interval`, plus always for `force == true`.
struct Throttle {
    last: Option<Instant>,
    interval: Duration,
}

impl Throttle {
    fn new(ms: u64) -> Self {
        Self { last: None, interval: Duration::from_millis(ms) }
    }
    fn ready(&mut self, force: bool) -> bool {
        let now = Instant::now();
        if force || self.last.map_or(true, |t| now - t >= self.interval) {
            self.last = Some(now);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EncodeProgress {
    done: usize,
    total: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeResult {
    pub path: String,
    pub bytes: u64,
    pub frames: usize,
}

#[tauri::command]
pub async fn probe_video(app: AppHandle, path: String) -> CmdResult<VideoInfo> {
    let info = blocking({
        let path = path.clone();
        move || probe::probe(&path).map_err(err)
    })
    .await?;
    app.asset_protocol_scope().allow_file(&path).map_err(err)?;
    Ok(info)
}

#[tauri::command]
pub async fn generate_thumbnails(
    app: AppHandle,
    path: String,
    duration: f64,
    count: usize,
) -> CmdResult<Vec<String>> {
    let dir = cache_dir(&app)?.join("thumbs").join(media::cache_key(&path));
    blocking(move || {
        media::thumbnails(&path, duration, count, &dir)
            .map(|v| v.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
            .map_err(err)
    })
    .await
}

#[tauri::command]
pub async fn prepare_proxy(app: AppHandle, path: String, duration: f64) -> CmdResult<String> {
    let dest = cache_dir(&app)?.join("proxy").join(format!("{}.mp4", media::cache_key(&path)));
    let cancel = {
        let state = app.state::<AppState>();
        let mut slot = state.proxy_cancel.lock().unwrap();
        slot.store(true, Ordering::SeqCst); // stop any previous proxy job
        *slot = Arc::new(AtomicBool::new(false));
        slot.clone()
    };
    let emitter = app.clone();
    blocking(move || {
        let mut throttle = Throttle::new(200);
        media::proxy(&path, duration, &dest, |p| {
            if throttle.ready(p >= 1.0) {
                let _ = emitter.emit("proxy://progress", p);
            }
            !cancel.load(Ordering::SeqCst)
        })
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(err)
    })
    .await
}

#[tauri::command]
pub async fn estimate_size(app: AppHandle, params: GifParams) -> CmdResult<Option<Estimate>> {
    let generation = app.state::<AppState>().estimate_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let handle = app.clone();
    blocking(move || {
        let state = handle.state::<AppState>();
        let stale = || state.estimate_generation.load(Ordering::SeqCst) != generation;
        match encode::estimate(&params, &stale) {
            Ok(e) => Ok(Some(e)),
            Err(_) if stale() => Ok(None), // superseded by a newer request
            Err(e) => Err(err(e)),
        }
    })
    .await
}

#[tauri::command]
pub async fn encode_gif(app: AppHandle, params: GifParams) -> CmdResult<EncodeResult> {
    let out_dir = cache_dir(&app)?.join("output");
    let cancel = {
        let state = app.state::<AppState>();
        let mut slot = state.encode_cancel.lock().unwrap();
        slot.store(true, Ordering::SeqCst);
        *slot = Arc::new(AtomicBool::new(false));
        slot.clone()
    };
    let emitter = app.clone();
    blocking(move || {
        // Keep only the latest output around.
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&out_dir).map_err(err)?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let out_path = out_dir.join(format!("preview-{stamp}.gif"));

        let total = params.total_frames();
        let mut throttle = Throttle::new(100);
        let file = File::create(&out_path).map_err(err)?;
        let result = encode::encode(
            &params,
            BufWriter::new(file),
            &|| cancel.load(Ordering::SeqCst),
            |done| {
                if throttle.ready(done >= total) {
                    let _ = emitter.emit("encode://progress", EncodeProgress { done, total });
                }
            },
        );
        match result {
            Ok(frames) => {
                let bytes = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
                Ok(EncodeResult { path: out_path.to_string_lossy().into_owned(), bytes, frames })
            }
            Err(e) => {
                let _ = std::fs::remove_file(&out_path);
                Err(err(e))
            }
        }
    })
    .await
}

#[tauri::command]
pub fn cancel_encode(app: AppHandle) {
    app.state::<AppState>().encode_cancel.lock().unwrap().store(true, Ordering::SeqCst);
}

#[tauri::command]
pub async fn save_gif(src: String, dest: String) -> CmdResult<()> {
    blocking(move || std::fs::copy(&src, &dest).map(|_| ()).map_err(err)).await
}
