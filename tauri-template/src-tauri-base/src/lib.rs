// src-tauri-base/src/lib.rs — Tauri app entry point.
//
// Startup sequence:
//   1. Splash window opens with visible:false; React invokes `splash_ready`
//      after the first CSS paint so the user never sees a transparent ghost.
//   2. Background thread runs the setup sequence while emitting
//      `splash://status` events that drive the splash terminal animation:
//        a. Spawn the PyInstaller sidecar (or Python dev-server fallback).
//        b. Check shared-drive reachability  → "Mounting shared drive".
//        c. Run the version check            → "Checking for updates".
//   3. The thread waits until at least 11 s have elapsed so the full
//      animation plays before the transition.
//   4. The splash closes and either the main window or the updater window
//      is shown, depending on the update check result.
//
// In debug builds the update / shared-drive check is skipped so local
// development works without a network drive.

mod sidecar;
mod splash;
mod updater;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Emitter, Listener, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

// ── Sidecar name ──────────────────────────────────────────────────────────

/// Name of the PyInstaller sidecar binary (without `.exe` extension).
///
/// Replace `${TOOL_SIDECAR_NAME}` with the actual sidecar name before use,
/// e.g. `"my-tool-backend"`.  This value must match the `sidecarName`
/// field in your `tauri.conf.json` and the PyInstaller spec `name`.
const SIDECAR_NAME: &str = "${TOOL_SIDECAR_NAME}";

// ── Backend state ─────────────────────────────────────────────────────────

/// Holds the backend base URL; updated once the sidecar starts.
struct BackendState {
    url: Mutex<String>,
}

// ── Update state ──────────────────────────────────────────────────────────

/// Holds the pending update information so the `start_update` Tauri command
/// can access the installer path after the user confirms in the UpdateModal.
struct UpdateState {
    latest: Mutex<Option<updater::LatestJson>>,
    update_path: Mutex<Option<PathBuf>>,
}

impl UpdateState {
    fn new() -> Self {
        Self {
            latest: Mutex::new(None),
            update_path: Mutex::new(None),
        }
    }
}

/// Tauri command: return the backend base URL to the webview.
#[tauri::command]
fn get_backend_url(state: tauri::State<BackendState>) -> String {
    state.url.lock().unwrap().clone()
}

/// Tauri command: list the immediate subdirectory names inside `path`.
///
/// Returns an empty list if the path does not exist or cannot be read.
/// Used by the frontend to detect when a user points the projects root at a
/// single project folder instead of the parent that contains all projects.
#[tauri::command]
fn peek_subfolders(path: String) -> Vec<String> {
    match std::fs::read_dir(&path) {
        Err(_) => vec![],
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
    }
}

const BACKEND_ADDR: &str = "127.0.0.1:8000";

/// Returns `true` when something is already listening on [`BACKEND_ADDR`].
fn is_backend_running() -> bool {
    std::net::TcpStream::connect_timeout(
        &BACKEND_ADDR.parse().expect("invalid socket address"),
        Duration::from_millis(300),
    )
    .is_ok()
}

/// Return a working Python executable path, preferring Miniconda.
///
/// Search order:
/// 1. **`CONDA_PREFIX`** — set when a conda environment is activated.
/// 2. **Well-known Miniconda / Anaconda install directories** under the
///    user's home folder.
/// 3. **`python` on `PATH`** — final fallback.
///
/// Each candidate is validated by running `<python> --version`.
fn find_python() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Active conda environment ($CONDA_PREFIX).
    if let Ok(prefix) = std::env::var("CONDA_PREFIX") {
        let prefix = PathBuf::from(prefix);
        if cfg!(windows) {
            candidates.push(prefix.join("python.exe"));
        } else {
            candidates.push(prefix.join("bin").join("python"));
        }
    }

    // 2. Well-known Miniconda / Anaconda install directories.
    if let Some(home) = home_dir() {
        let dir_names = ["miniconda3", "Miniconda3", "anaconda3", "Anaconda3"];
        for dir in &dir_names {
            if cfg!(windows) {
                candidates.push(home.join(dir).join("python.exe"));
            } else {
                candidates.push(home.join(dir).join("bin").join("python"));
            }
        }
    }

    // Try each candidate path and log what we're doing.
    for path in &candidates {
        println!("[tauri] Checking conda Python: {}", path.display());
        if path.is_file() {
            if Command::new(path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                println!("[tauri] Found conda Python: {}", path.display());
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }

    // 3. Fallback: `python` on PATH.
    println!("[tauri] No conda Python found; falling back to `python` on PATH");
    if Command::new("python")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        println!("[tauri] Found PATH Python: python");
        return Some("python".to_string());
    }

    None
}

/// Cross-platform helper to obtain the user's home directory.
fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

/// Locate the repository's `backend/` directory containing `app.py`.
fn find_backend_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let anchored = manifest_dir.join("..").join("..").join("backend");
    if anchored.join("app.py").is_file() {
        if let Ok(abs) = anchored.canonicalize() {
            return Some(abs);
        }
    }

    for rel in ["../backend", "../../backend", "./backend"] {
        let p = PathBuf::from(rel);
        if p.join("app.py").is_file() {
            if let Ok(abs) = p.canonicalize() {
                return Some(abs);
            }
        }
    }

    None
}

// ── Update outcome (internal) ─────────────────────────────────────────────

#[allow(dead_code)]
enum UpdateOutcome {
    UpToDate,
    UpdateAvailable {
        latest: updater::LatestJson,
        update_path: PathBuf,
    },
    Offline {
        path: PathBuf,
    },
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_for_setup = child.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_backend_url,
            peek_subfolders,
            start_update,
            splash::splash_is_first_run,
            splash::splash_ready,
            splash::splash_fade_complete,
        ])
        .manage(BackendState {
            url: Mutex::new(String::from("http://127.0.0.1:8000")),
        })
        .manage(UpdateState::new())
        .manage(splash::SplashState::new(splash::splash_first_launch_after_update()))
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let child_arc = child_for_setup.clone();

            // Run the startup sequence in a background thread so the splash
            // window remains responsive (event loop keeps running).
            thread::spawn(move || {
                startup_sequence(app_handle, child_arc);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Cleanup: terminate the backend process when the app exits.
    let mut proc_opt = child.lock().unwrap().take();
    if let Some(ref mut proc) = proc_opt {
        println!("[tauri] Stopping backend (PID {})", proc.id());
        let _ = proc.kill();
        let _ = proc.wait();
    }
}

// ── Startup sequence ──────────────────────────────────────────────────────

/// Minimum time (ms) the splash must stay visible so the full animation
/// plays through to the fade-out phase.
const MIN_SPLASH_MS: u64 = 13_000;

/// Reduced minimum (ms) for subsequent launches with the same version.
/// Chosen so the user sees at most one hammer-strike cycle before the
/// window closes (the CSS `animation-iteration-count: 1` in `.splash-root.short-mode`
/// limits the hammer loop to a single swing on the frontend side).
/// NOTE: if the frontend `hammer-strike` keyframe duration (currently 1.6 s in
/// splash.css) changes, this constant should be updated to remain in sync.
const MIN_SPLASH_MS_SHORT: u64 = 3_200;

/// Extra hold (ms) for the offline error state before the dialog fires.
const OFFLINE_EXTRA_MS: u64 = 3_000;

fn startup_sequence(app: tauri::AppHandle, child_arc: Arc<Mutex<Option<Child>>>) {
    let start = Instant::now();

    // Read optional debug hold (dev only; no-op in production when unset).
    let hold_ms: u64 = match std::env::var("SPLASH_HOLD_MS") {
        Ok(val) => match val.parse::<u64>() {
            Ok(ms) => ms,
            Err(_) => {
                eprintln!(
                    "[splash] SPLASH_HOLD_MS is set to {:?} but is not a valid u64; defaulting to 0",
                    val
                );
                0
            }
        },
        Err(_) => 0,
    };
    if hold_ms > 0 {
        println!("[splash] Debug hold mode active: {} ms per phase", hold_ms);
    }

    // Brief pause to let the splash window finish its initial render.
    thread::sleep(Duration::from_millis(200));

    // ── 1. Backend ────────────────────────────────────────────────────────
    splash::emit_status(
        &app,
        "backend",
        "Starting backend service",
        splash::StatusKind::Pending,
    );
    if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
    let backend_url = do_spawn_backend(&child_arc);
    println!("[tauri] Backend URL: {backend_url}");
    splash::emit_status(&app, "backend", "Starting backend service", splash::StatusKind::Ok);
    if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }

    // Store backend URL in managed state.
    if let Some(state) = app.try_state::<BackendState>() {
        *state.url.lock().unwrap() = backend_url;
    }

    // ── 2. Update check ───────────────────────────────────────────────────
    let outcome: UpdateOutcome;

    #[cfg(not(debug_assertions))]
    {
        outcome = run_update_check_with_status(&app, hold_ms);
    }
    #[cfg(debug_assertions)]
    {
        // Dev mode: skip the actual network check; emit fake Ok statuses.
        splash::emit_status(&app, "mount", "Mounting shared drive", splash::StatusKind::Pending);
        if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
        splash::emit_status(&app, "mount", "Mounting shared drive", splash::StatusKind::Ok);
        if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
        splash::emit_status(&app, "updates", "Checking for updates", splash::StatusKind::Pending);
        if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
        splash::emit_status(&app, "updates", "Checking for updates", splash::StatusKind::Ok);
        if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
        outcome = UpdateOutcome::UpToDate;
    }

    // ── 3. Final status line ──────────────────────────────────────────────
    let extra_hold_ms: u64 = match &outcome {
        UpdateOutcome::Offline { .. } => {
            splash::emit_status(
                &app,
                "final",
                "Cannot reach shared drive",
                splash::StatusKind::Error,
            );
            OFFLINE_EXTRA_MS + hold_ms
        }
        UpdateOutcome::UpdateAvailable { .. } => {
            splash::emit_status(
                &app,
                "final",
                "Update detected, loading updater\u{2026}",
                splash::StatusKind::Warn,
            );
            hold_ms
        }
        UpdateOutcome::UpToDate => {
            splash::emit_status(&app, "final", "Ready", splash::StatusKind::Ok);
            // Small pause so the UI has a guaranteed beat to render the "Ready"
            // line before the startup_sequence proceeds to the MIN_SPLASH_MS wait.
            thread::sleep(Duration::from_millis(200));
            hold_ms
        }
    };

    // ── 4. Minimum display duration ───────────────────────────────────────
    // Pick the appropriate minimum based on whether this is a first/update run.
    let is_first = app
        .try_state::<splash::SplashState>()
        .map(|s| s.first_run())
        .unwrap_or(true);
    let min_ms = if is_first { MIN_SPLASH_MS } else { MIN_SPLASH_MS_SHORT };
    let target_ms = min_ms + extra_hold_ms;
    let elapsed = start.elapsed().as_millis() as u64;

    if elapsed < target_ms {
        let remaining = target_ms - elapsed;
        thread::sleep(Duration::from_millis(remaining));
    }

    // ── 5. Transition ─────────────────────────────────────────────────────
    match outcome {
        UpdateOutcome::Offline { path } => {
            splash::close_splash(&app);
            app.dialog()
                .message(format!(
                    "Cannot reach shared drive at `{}`.\n\
                     Connect to the network (Drive for Desktop must be \
                     running) and try again.",
                    path.display()
                ))
                .title("Connection Required")
                .kind(MessageDialogKind::Error)
                .blocking_show();
            app.exit(1);
        }

        UpdateOutcome::UpdateAvailable {
            latest,
            update_path,
        } => {
            updater::log_updater(&format!(
                "Update available: {} → {} (path: {})",
                env!("CARGO_PKG_VERSION"),
                latest.version,
                update_path.display(),
            ));

            // Store update info in managed state so `start_update` can
            // retrieve it when the user clicks Install Now in the UpdateModal.
            if let Some(update_state) = app.try_state::<UpdateState>() {
                *update_state.latest.lock().unwrap() = Some(latest.clone());
                *update_state.update_path.lock().unwrap() = Some(update_path.clone());
            }

            // 1. Register the 'updater_ready' listener BEFORE scheduling the
            //    window show so we never miss the event if React mounts fast.
            let (ready_tx, ready_rx) = mpsc::channel::<()>();
            let _ = app.once("updater_ready", move |_| {
                // If the receiver was dropped we've already timed out; ignore.
                let _ = ready_tx.send(());
            });

            // 2. Marshal all window show / hide operations onto the Tauri
            //    main thread.  Calling WebviewWindow::show() or close() from
            //    a background thread deadlocks the Windows event loop in
            //    release builds.
            let app_for_ui = app.clone();
            let _ = app.run_on_main_thread(move || {
                if let Some(updater_win) = app_for_ui.get_webview_window("updater") {
                    let _ = updater_win.show();
                }
                splash::close_splash(&app_for_ui);
            });

            // 3. Wait for the React bundle to mount and register its event
            //    listeners (up to 2 s; fall through immediately on timeout).
            let _ = ready_rx.recv_timeout(Duration::from_secs(2));

            // 4. Emit update_info now that React listeners are registered.
            //    The updater window will display the UpdateModal; the user
            //    must click Install Now which invokes `start_update`.
            let _ = app.emit(
                "update_info",
                serde_json::json!({
                    "version": latest.version,
                    "notes":   latest.notes,
                }),
            );

            // Startup sequence is done.  The UpdateModal in the updater
            // window calls `start_update` when the user confirms.
        }

        UpdateOutcome::UpToDate => {
            // Trigger the splash fade-out → main-window cross-fade.
            //
            // Sequence:
            //   1. Emit `splash://fade-now` so the splash JS holds the
            //      success state for FADE_HOLD_MS, then cross-fades the
            //      whole `.splash-root` to opacity 0 over FADE_DURATION_MS.
            //   2. The splash invokes `splash_fade_complete` from
            //      `transitionend`, which shows the main window and
            //      closes the splash atomically — no "brown background
            //      gap" between the two.
            //   3. As a safety net (in case the frontend never invokes the
            //      command — e.g. JS error, window minimized mid-fade), we
            //      sleep for the expected hold + fade duration and then
            //      perform the same show/close from Rust. Both paths are
            //      idempotent.
            //
            // The constants below must stay in sync with the matching
            // FADE_HOLD_MS / FADE_DURATION_MS in frontend/src/splash.jsx.
            const FADE_HOLD_MS:     u64 = 800;
            const FADE_DURATION_MS: u64 = 1000;
            const FADE_SAFETY_MS:   u64 = 400;

            if let Err(e) = app.emit("splash://fade-now", ()) {
                eprintln!("[splash] emit splash://fade-now failed: {e}");
            }

            thread::sleep(Duration::from_millis(
                FADE_HOLD_MS + FADE_DURATION_MS + FADE_SAFETY_MS,
            ));

            // Safety net: idempotent if the frontend already invoked
            // splash_fade_complete from `transitionend`.
            let app_for_ui = app.clone();
            let _ = app.run_on_main_thread(move || {
                if let Some(main_win) = app_for_ui.get_webview_window("main") {
                    let _ = main_win.show();
                }
                splash::close_splash(&app_for_ui);
            });
        }
    }
}

// ── Update check with status emission ─────────────────────────────────────

#[cfg(not(debug_assertions))]
fn run_update_check_with_status(app: &tauri::AppHandle, hold_ms: u64) -> UpdateOutcome {
    // Step A: check drive reachability.
    splash::emit_status(app, "mount", "Mounting shared drive", splash::StatusKind::Pending);
    if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
    let update_path = updater::get_update_path();
    if !update_path.exists() {
        splash::emit_status(app, "mount", "Mounting shared drive", splash::StatusKind::Error);
        return UpdateOutcome::Offline { path: update_path };
    }
    splash::emit_status(app, "mount", "Mounting shared drive", splash::StatusKind::Ok);
    if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }

    // Step B: version comparison.
    splash::emit_status(app, "updates", "Checking for updates", splash::StatusKind::Pending);
    if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
    match updater::check_for_update() {
        updater::UpdateCheckResult::Offline { path } => {
            splash::emit_status(app, "updates", "Checking for updates", splash::StatusKind::Error);
            UpdateOutcome::Offline { path }
        }
        updater::UpdateCheckResult::UpdateAvailable {
            latest,
            update_path,
        } => {
            splash::emit_status(app, "updates", "Checking for updates", splash::StatusKind::Warn);
            if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
            UpdateOutcome::UpdateAvailable {
                latest,
                update_path,
            }
        }
        updater::UpdateCheckResult::UpToDate => {
            splash::emit_status(app, "updates", "Checking for updates", splash::StatusKind::Ok);
            if hold_ms > 0 { thread::sleep(Duration::from_millis(hold_ms)); }
            UpdateOutcome::UpToDate
        }
    }
}

// ── Backend spawning ──────────────────────────────────────────────────────

fn do_spawn_backend(child_arc: &Arc<Mutex<Option<Child>>>) -> String {
    // Production: try the PyInstaller sidecar first.
    if let Some(sidecar_path) = sidecar::find_sidecar_path(SIDECAR_NAME) {
        println!("[sidecar] Found sidecar at: {}", sidecar_path.display());
        match sidecar::spawn_sidecar(&sidecar_path) {
            Ok((proc, port)) => {
                *child_arc.lock().unwrap() = Some(proc);
                return format!("http://127.0.0.1:{port}");
            }
            Err(e) => {
                eprintln!("[sidecar] {e} -- falling back to Python dev server");
            }
        }
    }

    // Dev fallback: Python uvicorn on the fixed port 8000.
    spawn_python_backend(child_arc);
    String::from("http://127.0.0.1:8000")
}

// ── Python dev-server helpers ─────────────────────────────────────────────

fn spawn_python_backend(child_arc: &Arc<Mutex<Option<Child>>>) {
    if is_backend_running() {
        println!("[tauri] Backend already running on {BACKEND_ADDR}");
        return;
    }

    let python = match find_python() {
        Some(p) => p,
        None => {
            eprintln!(
                "[tauri] Python not found. \
                 Start the backend manually: cd backend && python -m uvicorn app:app --port 8000"
            );
            return;
        }
    };

    let backend_dir = match find_backend_dir() {
        Some(d) => d,
        None => {
            eprintln!("[tauri] Could not find backend/app.py. Start the backend manually.");
            return;
        }
    };

    println!("[tauri] Starting Python backend on port 8000");

    match Command::new(&python)
        .args([
            "-m",
            "uvicorn",
            "app:app",
            "--host",
            "127.0.0.1",
            "--port",
            "8000",
            "--reload",
        ])
        .current_dir(&backend_dir)
        .spawn()
    {
        Ok(c) => {
            let pid = c.id();
            println!("[tauri] Python backend spawned (PID {pid})");
            *child_arc.lock().unwrap() = Some(c);

            let check = child_arc.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(2));
                if let Ok(mut guard) = check.lock() {
                    if let Some(ref mut proc) = *guard {
                        if let Ok(Some(status)) = proc.try_wait() {
                            eprintln!(
                                "[tauri] Backend (PID {pid}) exited early: {status}. \
                                 Run: cd backend && pip install -r requirements.txt"
                            );
                        }
                    }
                }
            });
        }
        Err(e) => {
            eprintln!("[tauri] Failed to spawn Python backend: {e}");
        }
    }
}

// ── start_update command ──────────────────────────────────────────────────

/// Tauri command invoked by the `UpdateModal` frontend component when the
/// user clicks **Install Now**.
///
/// v2.1.0 sequence:
///   1. Read the pending installer name and shared-drive path from
///      [`UpdateState`] (populated by `startup_sequence` on update detect).
///   2. Copy the installer from the shared drive to `%TEMP%`, emitting
///      `update_progress` and `update_status` events.
///   3. Locate `desktop-toolkit-updater.exe` next to the running app exe.
///   4. Spawn the shim DETACHED with `--installer`, `--installed-app-exe`,
///      `--version`, and `--sidecar-name` args.
///   5. Call `app.exit(0)` — releasing the file lock on the parent exe so
///      NSIS can replace it cleanly.
///
/// The shim takes over from here: it waits for NSIS, then relaunches the
/// new version. This process is NOT blocked on `child.wait()`.
#[tauri::command]
fn start_update(app: tauri::AppHandle, state: tauri::State<UpdateState>) {
    let latest = state.latest.lock().unwrap().clone();
    let update_path = state.update_path.lock().unwrap().clone();

    let (latest, update_path) = match (latest, update_path) {
        (Some(l), Some(p)) => (l, p),
        _ => {
            updater::log_updater("start_update: no pending update in state");
            return;
        }
    };

    let app_clone = app.clone();
    thread::spawn(move || {
        // ── 1. Status: preparing ──────────────────────────────────────────
        let _ = app_clone.emit(
            "update_status",
            serde_json::json!({ "message": "Preparing update…" }),
        );

        // ── 2. Copy installer from shared drive to %TEMP% ─────────────────
        updater::log_updater(&format!(
            "start_update: copying {} from shared drive",
            latest.installer,
        ));
        let dest_path =
            match updater::copy_installer_with_progress(&update_path, &latest.installer, &app_clone) {
                Ok(p) => p,
                Err(e) => {
                    updater::log_updater(&format!("start_update: copy failed: {e}"));
                    app_clone.exit(1);
                    return;
                }
            };

        let _ = app_clone.emit(
            "update_status",
            serde_json::json!({ "message": "Closing application…" }),
        );

        // ── 3. Locate the updater shim ────────────────────────────────────
        let updater_exe = match std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("desktop-toolkit-updater.exe")))
        {
            Some(p) => p,
            None => {
                updater::log_updater("start_update: cannot determine updater exe path");
                app_clone.exit(1);
                return;
            }
        };

        let installed_app_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                updater::log_updater(&format!("start_update: cannot determine current exe: {e}"));
                app_clone.exit(1);
                return;
            }
        };

        updater::log_updater(&format!("start_update: spawning shim {:?}", updater_exe));

        // ── 4. Spawn updater shim DETACHED ────────────────────────────────
        let mut cmd = Command::new(&updater_exe);
        cmd.arg("--installer")
            .arg(&dest_path)
            .arg("--installed-app-exe")
            .arg(&installed_app_exe)
            .arg("--version")
            .arg(&latest.version)
            .arg("--sidecar-name")
            .arg(SIDECAR_NAME);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP — shim survives parent exit.
            cmd.creation_flags(0x0000_0008 | 0x0000_0200);
        }

        match cmd.spawn() {
            Ok(child) => {
                updater::log_updater(&format!(
                    "start_update: shim launched (PID {}), exiting parent",
                    child.id()
                ));
            }
            Err(e) => {
                updater::log_updater(&format!(
                    "start_update: failed to launch shim: {e}"
                ));
                app_clone.exit(1);
                return;
            }
        }

        // ── 5. Exit parent — releases file lock on the app exe ────────────
        // Brief sleep so the OS finishes handing off to the shim before we
        // release the process handles.
        thread::sleep(Duration::from_millis(200));
        app_clone.exit(0);
    });
}
