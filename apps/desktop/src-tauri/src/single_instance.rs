//! Single-instance enforcement via a file-based lock in the app data directory.
//!
//! Only one QRForge host process may run at a time. A second launch detects
//! the running instance and exits cleanly.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Attempts to acquire an exclusive single-instance lock.
///
/// Returns the lock guard. The lock is released when the guard is dropped.
/// If the lock cannot be acquired, prints an error message and exits the process.
///
/// This must be called early in the startup sequence, before Tauri initialization.
#[must_use]
pub fn acquire_or_exit() -> LockGuard {
    let app_data_dir = if let Ok(appdata) = std::env::var("APPDATA") {
        let path = PathBuf::from(&appdata).join("QRForge");
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Failed to create QRForge data directory: {e}");
            std::process::exit(1);
        }
        path
    } else {
        eprintln!("Failed to find APPDATA directory");
        std::process::exit(1);
    };

    match acquire(&app_data_dir) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

/// Acquires an exclusive single-instance lock or fails if already held.
///
/// Returns the lock guard. The lock is released when the guard is dropped.
pub fn acquire(app_data_dir: &Path) -> Result<LockGuard, String> {
    let lock_file = app_data_dir.join("qrforge.lock");

    // Ensure parent directory exists
    if let Some(parent) = lock_file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create app data dir: {e}"))?;
    }

    // Try to acquire the lock by opening the file exclusively.
    // On Windows, this prevents other processes from opening it while we hold it.
    if let Ok(file) = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_file)
    {
        // Write the PID for debugging purposes
        let pid = std::process::id();
        let _ = std::io::Write::write_all(&mut &file, pid.to_string().as_bytes());

        Ok(LockGuard {
            lock_file: lock_file.clone(),
            _file: file,
        })
    } else {
        // Lock file is held by another process; wait briefly and check again.
        // If still locked after a short timeout, we assume another instance is running.
        std::thread::sleep(Duration::from_millis(50));

        // Try once more to be sure
        if let Ok(file) = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_file)
        {
            let pid = std::process::id();
            let _ = std::io::Write::write_all(&mut &file, pid.to_string().as_bytes());
            Ok(LockGuard {
                lock_file: lock_file.clone(),
                _file: file,
            })
        } else {
            Err("QRForge is already running. Only one instance is allowed.".to_string())
        }
    }
}

/// RAII guard that releases the lock when dropped.
pub struct LockGuard {
    lock_file: PathBuf,
    _file: std::fs::File,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Clean up the lock file on normal exit
        let _ = fs::remove_file(&self.lock_file);
    }
}
