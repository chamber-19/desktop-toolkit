// crates/desktop-toolkit-updater/src/main.rs
//
// desktop-toolkit-updater — standalone updater shim binary.
//
// This is a small, standalone process that handles the entire upgrade flow
// AFTER the parent app has exited, so NSIS can replace the parent's own exe
// without hitting a "file in use" lock.
//
// CLI:
//   desktop-toolkit-updater.exe
//     --installer        <path-to-installer.exe>
//     --installed-app-exe <path-to-the-app-being-updated.exe>
//     --version          <new-version-string>
//     [--sidecar-name    <sidecar-binary-name-without-.exe>]
//
// Sequence:
//   1. Parse CLI args.
//   2. Wait briefly for the parent app to fully exit.
//   3. Kill sidecar process if --sidecar-name is provided.
//   4. Spawn the NSIS installer and wait for it to complete (child.wait() is
//      safe here — this process is NOT the one being replaced).
//   5. On success, spawn the new app version detached.
//   6. Exit.
//
// Logs to: %LOCALAPPDATA%\desktop-toolkit-updater\updater.log

#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let installer_path = match get_arg(&args, "--installer") {
        Some(p) => p,
        None => {
            log_shim("shim: --installer argument is required");
            std::process::exit(1);
        }
    };
    let installed_app_exe = match get_arg(&args, "--installed-app-exe") {
        Some(p) => p,
        None => {
            log_shim("shim: --installed-app-exe argument is required");
            std::process::exit(1);
        }
    };
    let version = get_arg(&args, "--version").unwrap_or_else(|| "unknown".to_string());
    let sidecar_name = get_arg(&args, "--sidecar-name");

    log_shim(&format!(
        "shim: starting (version={version}, installer={installer_path:?})"
    ));

    // ── 1. Wait for parent to fully exit ─────────────────────────────────
    // The parent called app.exit(0) before spawning us; give Windows a moment
    // to fully release file handles on the parent exe.
    std::thread::sleep(Duration::from_millis(500));

    // ── 2. Kill sidecar (optional) ────────────────────────────────────────
    if let Some(ref sidecar) = sidecar_name {
        log_shim(&format!("shim: killing sidecar {sidecar}.exe"));
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", &format!("{sidecar}.exe"), "/T"])
                .status();
        }
        // Give Windows time to release file handles on the sidecar directory.
        std::thread::sleep(Duration::from_secs(1));
    }

    // ── 3. Spawn NSIS installer and wait ──────────────────────────────────
    let installer = PathBuf::from(&installer_path);
    if !installer.is_file() {
        log_shim(&format!(
            "shim: installer not found at {installer_path:?}"
        ));
        std::process::exit(1);
    }

    let mut cmd = Command::new(&installer);
    cmd.args(["/PASSIVE", "/NORESTART"]);

    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            log_shim(&format!("shim: installer launched (PID {pid})"));

            match child.wait() {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    log_shim(&format!("shim: installer exited with code {code}"));

                    if status.success() {
                        relaunch_app(&installed_app_exe, &version);
                    } else {
                        log_shim(&format!(
                            "shim: installer failed (exit code {code}), not relaunching"
                        ));
                    }
                }
                Err(e) => {
                    log_shim(&format!("shim: wait() failed: {e}"));
                }
            }
        }
        Err(e) => {
            log_shim(&format!("shim: failed to spawn installer: {e}"));
        }
    }

    log_shim("shim: done");
}

// ── Relaunch helper ───────────────────────────────────────────────────────

fn relaunch_app(installed_app_exe: &str, version: &str) {
    // Brief pause to let NSIS finish writing files before we open the new exe.
    std::thread::sleep(Duration::from_millis(500));

    let new_exe = PathBuf::from(installed_app_exe);
    log_shim(&format!("shim: relaunching {new_exe:?} (version {version})"));

    let mut relaunch = Command::new(&new_exe);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP — new app is independent.
        relaunch.creation_flags(0x0000_0008 | 0x0000_0200);
    }

    match relaunch.spawn() {
        Ok(child) => {
            log_shim(&format!("shim: relaunched (PID {})", child.id()));
        }
        Err(e) => {
            log_shim(&format!("shim: relaunch failed: {e}"));
        }
    }
}

// ── CLI arg parser ────────────────────────────────────────────────────────

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned();
        }
    }
    None
}

// ── File logger ───────────────────────────────────────────────────────────

fn log_shim(msg: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("TEMP"))
        .unwrap_or_else(|_| String::from("C:\\Temp"));
    let log_path = PathBuf::from(base)
        .join("desktop-toolkit-updater")
        .join("updater.log");

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{secs}] {msg}");
    }

    // Also print to stderr so the parent process can capture it during testing.
    eprintln!("{msg}");
}
