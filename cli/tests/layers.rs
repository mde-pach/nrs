//! Integration tests for `nrs claude layers`.

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn nrs() -> Command {
    Command::cargo_bin("nrs").unwrap()
}

#[test]
fn layers_lists_root_context() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();
    std::fs::write(tmp.path().join("nrs.context.md"), "# NRS").unwrap();

    nrs()
        .args(["claude", "layers", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("CLAUDE.md"))
        .stdout(contains("nrs"))
        .stdout(contains("project"));
}

#[test]
fn layers_lists_nested_context() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();
    let billing = tmp.path().join("src").join("billing");
    std::fs::create_dir_all(&billing).unwrap();
    std::fs::write(billing.join("domain.context.md"), "# Billing").unwrap();

    nrs()
        .args(["claude", "layers", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("CLAUDE.md — project"))
        .stdout(contains("src/billing/CLAUDE.md — domain"));
}

#[test]
fn layers_empty_project_silent() {
    let tmp = TempDir::new().unwrap();

    nrs()
        .args(["claude", "layers", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

#[test]
fn layers_lists_multiple_layers_in_one_directory() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("nrs.context.md"), "# NRS").unwrap();
    std::fs::write(tmp.path().join("corporate.context.md"), "# Corp").unwrap();
    std::fs::write(tmp.path().join("team.context.md"), "# Team").unwrap();
    std::fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();

    let output = nrs()
        .args(["claude", "layers", "--dir"])
        .arg(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Layer names appear in sort order (nrs → corporate → team → project) on the CLAUDE.md line
    let layer_line = stdout
        .lines()
        .find(|l| l.contains("CLAUDE.md"))
        .expect("layer line present");
    let nrs_idx = layer_line.find("nrs").unwrap();
    let corp_idx = layer_line.find("corporate").unwrap();
    let team_idx = layer_line.find("team").unwrap();
    let proj_idx = layer_line.find("project").unwrap();
    assert!(nrs_idx < corp_idx && corp_idx < team_idx && team_idx < proj_idx);
}

#[test]
fn layers_hook_mode_uses_cwd_from_stdin() {
    // The `--dir` flag is ignored in hook-mode; cwd from stdin wins.
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();

    let other = TempDir::new().unwrap();
    let input = serde_json::json!({ "cwd": tmp.path().to_str().unwrap() });

    let output = nrs()
        .args(["claude", "layers", "--hook-mode", "--dir"])
        .arg(other.path()) // --dir points to an empty project
        .write_stdin(serde_json::to_string(&input).unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("project"));
}

#[test]
fn layers_hook_mode_invalid_json_errors() {
    nrs()
        .args(["claude", "layers", "--hook-mode"])
        .write_stdin("not json at all")
        .assert()
        .failure()
        .stderr(contains("failed to parse hook JSON"));
}

#[test]
fn layers_hook_mode_outputs_json() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();

    let input = serde_json::json!({ "cwd": tmp.path().to_str().unwrap() });

    let output = nrs()
        .args(["claude", "layers", "--hook-mode"])
        .write_stdin(serde_json::to_string(&input).unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("CLAUDE.md"));
}
