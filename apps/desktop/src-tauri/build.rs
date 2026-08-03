//! Tauri resource/capability build script and build identity capture.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=QRFORGE_BUILD_COMMIT");
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    let commit = std::env::var("QRFORGE_BUILD_COMMIT")
        .ok()
        .or_else(git_head)
        .filter(|value| is_commit(value))
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=QRFORGE_BUILD_COMMIT={commit}");
    tauri_build::build();
}

fn git_head() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
