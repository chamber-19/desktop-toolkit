// crates/desktop-toolkit/src/log.rs
//
// Shared file-based logging for framework modules.
//
// Writes to %LOCALAPPDATA%\<log_dir>\updater.log (release builds only).
// In debug builds, all log calls are no-ops.

/// Write a timestamped log line to `%LOCALAPPDATA%\<log_dir>\updater.log`.
///
/// No-op in debug builds. `log_dir` is typically the product/package name.
pub fn log_to_file(log_dir: &str, msg: &str) {
    #[cfg(not(debug_assertions))]
    {
        use std::fs;
        use std::io::Write;
        use std::path::PathBuf;
        use std::time::{SystemTime, UNIX_EPOCH};

        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let base = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("TEMP"))
            .unwrap_or_else(|_| String::from("C:\\Temp"));
        let log_path = PathBuf::from(base).join(log_dir).join("updater.log");

        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(f, "[{secs}] {msg}");
        }
    }

    #[cfg(debug_assertions)]
    {
        let _ = (log_dir, msg);
    }
}
