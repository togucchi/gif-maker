//! Locating and launching the bundled FFmpeg sidecar.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const EXE: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };

/// Tauri copies `externalBin` entries next to the main executable (without the
/// target-triple suffix) both in `tauri dev` and in bundles. Fall back to PATH.
pub fn path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        if let Some(p) = std::env::var_os("GIF_MAKER_FFMPEG") {
            return PathBuf::from(p);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let candidate = dir.join(EXE);
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
        PathBuf::from(EXE)
    })
}

/// A `Command` for ffmpeg that never pops up a console window on Windows.
pub fn command() -> Command {
    let mut cmd = Command::new(path());
    cmd.arg("-hide_banner").arg("-nostdin");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Format seconds for ffmpeg's `-ss` / `-t` options.
pub fn secs(v: f64) -> String {
    format!("{:.3}", v.max(0.0))
}
