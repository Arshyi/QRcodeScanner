//! Tauri resource/capability build script and build identity capture.

use std::{path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=QRFORGE_BUILD_COMMIT");
    for path in git_watch_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let commit = std::env::var("QRFORGE_BUILD_COMMIT")
        .ok()
        .or_else(git_head)
        .filter(|value| is_commit(value))
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=QRFORGE_BUILD_COMMIT={commit}");
    tauri_build::build();
}

fn git_head() -> Option<String> {
    git_output(&["rev-parse", "HEAD"])
}

fn git_watch_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for logical_path in ["HEAD", "packed-refs"] {
        if let Some(path) = git_path(logical_path) {
            paths.push(path);
        }
    }
    if let Some(reference) = git_output(&["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git_path(&reference)
    {
        paths.push(path);
    }
    paths.sort_unstable();
    paths.dedup();
    paths
}

fn git_path(logical_path: &str) -> Option<PathBuf> {
    git_output(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        logical_path,
    ])
    .map(PathBuf::from)
}

fn git_output(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
