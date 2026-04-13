//! Shared helpers for integration tests.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Create a temp directory and run `git init` so commands that require a git
/// repo (like `nrs init`) work.
pub fn git_repo() -> TempDir {
    let tmp = TempDir::new().expect("create tempdir");
    let status = Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(tmp.path())
        .status()
        .expect("run git init");
    assert!(status.success(), "git init failed");
    tmp
}

/// Write a file at `dir/relative_path`, creating parent directories as needed.
pub fn write_file(dir: &Path, relative_path: &str, content: &str) {
    let full = dir.join(relative_path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&full, content).expect("write file");
}

/// Root of the fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contexts")
}

/// Read a fixture file by name from `tests/fixtures/contexts/`.
pub fn fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture '{}': {}", name, e))
}

/// Build a temp project from fixture files, run `nrs generate`, and return the
/// `TempDir`.
///
/// Each entry maps a fixture filename to the target path inside the project:
///
/// ```ignore
/// let tmp = project_from_fixtures(&[
///     ("project-with-orders.md", "project.context.md"),
///     ("orders-domain.md",       "src/orders/domain.context.md"),
/// ]);
/// ```
pub fn project_from_fixtures(fixtures: &[(&str, &str)]) -> TempDir {
    let tmp = TempDir::new().expect("create tempdir");
    for (fixture_name, target_path) in fixtures {
        let content = fixture(fixture_name);
        write_file(tmp.path(), target_path, &content);
    }
    // Generate so drift validator doesn't fire.
    let status = assert_cmd::Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();
    let _ = status; // suppress unused warning
    tmp
}
