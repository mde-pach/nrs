//! Integration tests for `nrs init`.

mod common;

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn init_creates_nrs_context_and_pre_commit_hook() {
    let tmp = common::git_repo();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("created nrs.context.md"))
        .stdout(contains("created pre-commit hook"));

    let nrs_context = tmp.path().join("nrs.context.md");
    assert!(nrs_context.exists(), "nrs.context.md should be created");
    let body = std::fs::read_to_string(&nrs_context).unwrap();
    assert!(body.contains("NRS"), "template content should be written");

    let hook = tmp.path().join(".git/hooks/pre-commit");
    assert!(hook.exists(), "pre-commit hook should be created");
    let hook_body = std::fs::read_to_string(&hook).unwrap();
    assert!(hook_body.contains("nrs generate all"));
    assert!(hook_body.contains("nrs validate"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "hook should be executable");
    }
}

#[test]
fn init_is_idempotent() {
    let tmp = common::git_repo();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("nrs.context.md already exists"))
        .stdout(contains("already contains NRS commands"));
}

#[test]
fn init_appends_to_existing_pre_commit_hook() {
    let tmp = common::git_repo();
    let hook = tmp.path().join(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(&hook, "#!/bin/sh\necho existing\n").unwrap();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("appended NRS commands"));

    let body = std::fs::read_to_string(&hook).unwrap();
    assert!(body.contains("echo existing"), "preserves existing hook");
    assert!(body.contains("nrs generate all"), "adds NRS commands");
}

#[test]
fn init_fails_outside_git_repo() {
    let tmp = tempfile::TempDir::new().unwrap();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(contains("not a git repository"));
}

#[test]
fn init_fails_on_missing_directory() {
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir", "/tmp/nrs-does-not-exist-xyz-12345"])
        .assert()
        .failure()
        .stderr(contains("directory not found"));
}
