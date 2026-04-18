// src-tauri-base/src/splash.rs
//
// Splash screen integration.
//
// Provides:
//   - `emit_status`                    — emit a terminal status line to the splash window.
//   - `close_splash`                   — close the splash window (no-op if already closed).
//   - `SplashState`                    — managed Tauri state holding the first-run flag.
//   - `splash_is_first_run`            — Tauri command: returns true when this launch
//     follows an update or is the very first launch (drives full vs. short animation).
//   - `splash_ready`                   — Tauri command: called by the frontend after the
//     first CSS paint to show the window (avoids transparent-ghost pre-flash).
//   - `splash_first_launch_after_update` — internal helper: reads/writes the
//     splash-seen.json sentinel and returns true when the stored version differs
//     from the current binary version (i.e. a fresh install or just-updated launch).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

// ── Status event types ────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum StatusKind {
    Pending,
    Ok,
    Warn,
    Error,
}

#[derive(Serialize, Clone, Debug)]
pub struct StatusPayload {
    pub phase: String,
    pub message: String,
    pub kind: StatusKind,
}

// ── Managed state ─────────────────────────────────────────────────────────

/// Shared state held in Tauri's managed-state map.
pub struct SplashState {
    pub is_first_run:   Arc<AtomicBool>,
}

impl SplashState {
    pub fn new(is_first_run: bool) -> Self {
        Self {
            is_first_run:   Arc::new(AtomicBool::new(is_first_run)),
        }
    }

    pub fn first_run(&self) -> bool {
        self.is_first_run.load(Ordering::SeqCst)
    }
}

// ── Sentinel JSON schema ──────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
struct SplashSeen {
    last_seen_version: String,
}

// ── First-launch-after-update logic ──────────────────────────────────────

/// Returns `true` if this is the first launch after an install or update.
///
/// Reads `%APPDATA%\<app-identifier>\splash-seen.json` (created on first
/// run) and compares the stored version against `CARGO_PKG_VERSION`.
/// If they differ (or the file is absent) it writes the current version back
/// and returns `true` so the full 9.5 s animation plays.
/// On subsequent launches with the same version it returns `false`, triggering
/// the short (~3.2 s) mode.
pub fn splash_first_launch_after_update() -> bool {
    if let Ok(val) = std::env::var("SPLASH_FORCE_FRESH") {
        let v = val.to_ascii_lowercase();
        if v == "1" || v == "true" || v == "yes" {
            return true;
        }
    }

    let current = env!("CARGO_PKG_VERSION");

    let base_opt = {
        #[cfg(windows)]
        let v = std::env::var("APPDATA").ok();
        #[cfg(not(windows))]
        let v = std::env::var("HOME")
            .ok()
            .map(|h| format!("{h}/.local/share"));
        v
    };

    let base = match base_opt {
        Some(b) if !b.is_empty() => b,
        _ => return true,
    };

    // Use the Cargo package name as the app identifier directory.
    let sentinel_path = std::path::PathBuf::from(base)
        .join(env!("CARGO_PKG_NAME"))
        .join("splash-seen.json");

    let last_seen = sentinel_path
        .exists()
        .then(|| std::fs::read_to_string(&sentinel_path).ok())
        .flatten()
        .and_then(|s| serde_json::from_str::<SplashSeen>(&s).ok())
        .map(|ss| ss.last_seen_version);

    let is_new = last_seen.as_deref() != Some(current);

    if is_new {
        if let Some(parent) = sentinel_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let sentinel = SplashSeen {
            last_seen_version: current.to_string(),
        };
        if let Ok(json) = serde_json::to_string(&sentinel) {
            let _ = std::fs::write(&sentinel_path, json);
        }
    }

    is_new
}

// ── Tauri commands ────────────────────────────────────────────────────────

/// Returns `true` when the current launch is the first launch after an install
/// or update (i.e. the full ~9.5 s animation should play).
#[tauri::command]
pub fn splash_is_first_run(state: tauri::State<SplashState>) -> bool {
    state.first_run()
}

/// Called by the splash frontend once the first CSS paint has completed.
#[tauri::command]
pub fn splash_ready(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("splash") {
        let _ = win.show();
    }
}

/// Called by the splash frontend after the cross-fade animation completes.
#[tauri::command]
pub fn splash_fade_complete(app: tauri::AppHandle) {
    let app_for_ui = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(main_win) = app_for_ui.get_webview_window("main") {
            let _ = main_win.show();
        }
        close_splash(&app_for_ui);
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Emit a `splash://status` event to all windows (including the splash).
pub fn emit_status(app: &AppHandle, phase: &str, message: &str, kind: StatusKind) {
    let payload = StatusPayload {
        phase: phase.to_string(),
        message: message.to_string(),
        kind,
    };
    if let Err(e) = app.emit("splash://status", payload) {
        eprintln!("[splash] emit_status failed: {e}");
    }
}

/// Close the splash window. Silently ignores errors (e.g. already closed).
pub fn close_splash(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("splash") {
        if let Err(e) = win.close() {
            eprintln!("[splash] close_splash failed: {e}");
        }
    }
}
